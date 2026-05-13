use std::{collections::HashMap, time::Duration};

use data_url::DataUrl;
use image::load_from_memory_with_format;
use mirajazz::{device::Device, error::MirajazzError, state::DeviceStateUpdate};
use openaction::{device_plugin::*, global_events::SetImageEvent};
use tokio::{sync::mpsc, time::timeout};
use tokio_util::sync::CancellationToken;

use crate::{
    BUTTONS, DEVICES, FLUSH_CHANNELS, SUSPENSION_CHANNELS, TOKENS,
    inputs::opendeck_to_device,
    mappings::{COL_COUNT, CandidateDevice, ENCODER_COUNT, IMAGE_FORMAT, KEY_COUNT, ROW_COUNT},
};

const IDLE_TIME: Duration = Duration::from_secs(60 * 60);
const FLUSH_DEBOUNCE_TIME: Duration = Duration::from_millis(70);

/// Initializes a device and listens for events
pub async fn device_task(candidate: CandidateDevice, token: CancellationToken) {
    log::info!("Running device task for {:?}", candidate);

    // Wrap in a closure so we can use `?` operator
    let device = async {
        let device = connect(&candidate).await?;

        device.set_brightness(50).await?;
        device.clear_all_button_images().await?;
        device.flush().await?;

        Ok(device)
    }
    .await;

    let device: Device = match device {
        Ok(device) => device,
        Err(err) => {
            handle_error(&candidate.id, err).await;

            log::error!(
                "Had error during device init, finishing device task: {:?}",
                candidate
            );

            return;
        }
    };

    log::debug!("Registering device {}", candidate.id);
    register_device(
        candidate.id.clone(),
        candidate.kind.human_name(),
        ROW_COUNT as u8,
        COL_COUNT as u8,
        ENCODER_COUNT as u8,
        0,
    )
    .await
    .unwrap();

    DEVICES.write().await.insert(candidate.id.clone(), device);

    BUTTONS
        .write()
        .await
        .insert(candidate.id.clone(), HashMap::new());

    let (suspension_tx, suspension_rx) = mpsc::channel::<()>(10);

    SUSPENSION_CHANNELS
        .write()
        .await
        .insert(candidate.id.clone(), suspension_tx);

    let (flush_tx, flush_rx) = mpsc::channel::<()>(10);

    FLUSH_CHANNELS
        .write()
        .await
        .insert(candidate.id.clone(), flush_tx);

    tokio::select! {
        _ = device_suspension_task(&candidate, suspension_rx) => {},
        _ = device_flush_debouncer_task(&candidate, flush_rx) => {},
        _ = device_events_task(&candidate) => {},
        _ = token.cancelled() => {}
    };

    log::info!("Shutting down device {:?}", candidate);

    SUSPENSION_CHANNELS.write().await.remove(&candidate.id);

    if let Some(device) = DEVICES.read().await.get(&candidate.id) {
        device.shutdown().await.ok();
    }

    log::info!("Device task finished for {:?}", candidate);
}

/// Handles errors, returning true if should continue, returning false if an error is fatal
pub async fn handle_error(id: &String, err: MirajazzError) -> bool {
    log::error!("Device {} error: {}", id, err);

    // Some errors are not critical and can be ignored without sending disconnected event
    if matches!(err, MirajazzError::ImageError(_) | MirajazzError::BadData) {
        return true;
    }

    log::debug!("Deregistering device {}", id);
    unregister_device(id.clone()).await.unwrap();

    log::debug!("Cancelling tasks for device {}", id);
    if let Some(token) = TOKENS.read().await.get(id) {
        token.cancel();
    }

    log::debug!("Removing device {} from the list", id);
    DEVICES.write().await.remove(id);

    log::debug!("Removing buttons for device {}", id);
    BUTTONS.write().await.remove(id);

    log::debug!("Finished clean-up for {}", id);

    false
}

pub async fn connect(candidate: &CandidateDevice) -> Result<Device, MirajazzError> {
    let result = Device::connect(&candidate.dev, 1, KEY_COUNT, ENCODER_COUNT)
        .await
        .map(|d| d.with_supports_both_keypress_states(true))
        .map(|d| d.with_supports_both_encoder_states(true));

    match result {
        Ok(device) => Ok(device),
        Err(e) => {
            log::error!("Error while connecting to device: {e}");

            Err(e)
        }
    }
}

/// Handles the idle time of the device
async fn device_suspension_task(candidate: &CandidateDevice, mut rx: mpsc::Receiver<()>) {
    log::info!("Starting device suspension task");

    let mut sleeping = false;
    while !rx.is_closed() {
        if timeout(IDLE_TIME, rx.recv()).await.is_ok() {
            if sleeping {
                log::info!("Device {} is now awake", candidate.id);
                sleeping = false;
            }
        } else if !sleeping {
            log::info!("Putting device {} to sleep", candidate.id);

            if let Some(device) = DEVICES.read().await.get(&candidate.id)
                && let Err(err) = device.sleep().await
            {
                log::error!("Failed to put device {} to sleep: {}", candidate.id, err);
            }
            sleeping = true;
        }
    }
}

/// Debounces flush events
async fn device_flush_debouncer_task(candidate: &CandidateDevice, mut rx: mpsc::Receiver<()>) {
    log::info!("Starting device flush debouncer task");

    while let Some(()) = rx.recv().await {
        log::info!("First flush event received");

        let _ = timeout(FLUSH_DEBOUNCE_TIME, async {
            while rx.recv().await.is_some() {}
        })
        .await;

        log::info!("Sending flush event for {}", candidate.id);
        if let Some(device) = DEVICES.read().await.get(&candidate.id)
            && let Err(err) = device.flush().await
        {
            log::error!(
                "Failed to send flush event for device {}: {}",
                candidate.id,
                err
            );
        }
    }
}

/// Handles events from device to OpenDeck
async fn device_events_task(candidate: &CandidateDevice) -> Result<(), MirajazzError> {
    log::info!("Connecting to {} for incoming events", candidate.id);

    let devices_lock = DEVICES.read().await;
    let reader = match devices_lock.get(&candidate.id) {
        Some(device) => device.get_reader(crate::inputs::process_input),
        None => return Ok(()),
    };
    drop(devices_lock);

    log::info!("Connected to {} for incoming events", candidate.id);

    log::info!("Reader is ready for {}", candidate.id);

    loop {
        log::info!("Reading updates...");

        let updates = match reader.read(None).await {
            Ok(updates) => updates,
            Err(e) => {
                if !handle_error(&candidate.id, e).await {
                    break;
                }

                continue;
            }
        };

        if !updates.is_empty()
            && let Some(tx) = SUSPENSION_CHANNELS.read().await.get(&candidate.id)
        {
            let _ = tx.send(()).await;
        }

        for update in updates {
            log::debug!("New update: {:#?}", update);

            let id = candidate.id.clone();

            match update {
                DeviceStateUpdate::ButtonDown(key) => key_down(id, key).await.unwrap(),
                DeviceStateUpdate::ButtonUp(key) => key_up(id, key).await.unwrap(),
                DeviceStateUpdate::EncoderDown(encoder) => {
                    encoder_down(id, encoder).await.unwrap();
                }
                DeviceStateUpdate::EncoderUp(encoder) => {
                    encoder_up(id, encoder).await.unwrap();
                }
                DeviceStateUpdate::EncoderTwist(encoder, val) => {
                    encoder_change(id, encoder, val as i16).await.unwrap();
                }
            }
        }
    }

    Ok(())
}

/// Handles different combinations of "set image" event, including clearing the specific buttons and whole device
pub async fn handle_set_image(device: &Device, evt: SetImageEvent) -> Result<(), MirajazzError> {
    match (evt.position, evt.image) {
        (Some(position), Some(image)) => {
            log::info!("Setting image for button {}", position);

            // OpenDeck sends image as a data url, so parse it using a library
            let url = DataUrl::process(image.as_str()).unwrap(); // Isn't expected to fail, so unwrap it is
            let (body, _fragment) = url.decode_to_vec().unwrap(); // Same here

            // Allow only image/jpeg mime for now
            if url.mime_type().subtype != "jpeg" {
                log::error!("Incorrect mime type: {}", url.mime_type());

                return Ok(()); // Not a fatal error, enough to just log it
            }

            let hash = blake3::hash(body.as_slice());

            if let Some(m) = BUTTONS.read().await.get(&evt.device)
                && let Some(old_hash) = m.get(&position)
                && hash == *old_hash
            {
                log::info!(
                    "Image for button {} is the same as before, skipping",
                    position
                );
                return Ok(());
            }

            let image = load_from_memory_with_format(body.as_slice(), image::ImageFormat::Jpeg)?;
            device
                .set_button_image(opendeck_to_device(position), IMAGE_FORMAT, image.clone())
                .await?;
            device_flush(&evt.device, device).await?;

            BUTTONS
                .write()
                .await
                .entry(evt.device)
                .or_insert(HashMap::new())
                .insert(position, hash.into());
        }
        (Some(position), None) => {
            if BUTTONS
                .read()
                .await
                .get(&evt.device)
                .is_none_or(|d| d.get(&position).is_none())
            {
                log::info!("Image for button {} is already empty, skipping", position);
                return Ok(());
            }
            device
                .clear_button_image(opendeck_to_device(position))
                .await?;
            device_flush(&evt.device, device).await?;

            if let Some(buttons) = BUTTONS.write().await.get_mut(&evt.device) {
                buttons.remove(&position);
            }
        }
        (None, None) => {
            device.flush().await?; // Manually flush to display the pressed button

            if BUTTONS.read().await.get(&evt.device).is_none() {
                log::info!("Buttons are already empty, skipping");
                return Ok(());
            }
            device.clear_all_button_images().await?;
            device_flush(&evt.device, device).await?;

            BUTTONS.write().await.remove(&evt.device);
        }
        _ => {}
    }

    Ok(())
}

async fn device_flush(device_id: &str, device: &Device) -> Result<(), MirajazzError> {
    match FLUSH_CHANNELS.read().await.get(device_id) {
        Some(tx) if tx.send(()).await.is_err() => device.flush().await,
        _ => Ok(()),
    }
}
