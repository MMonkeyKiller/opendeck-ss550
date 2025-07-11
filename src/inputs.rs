use mirajazz::{error::MirajazzError, types::DeviceInput};

use crate::mappings::KEY_COUNT;

pub fn process_input(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::debug!("Processing input: {}, {}", input, state);

    match input as usize {
        (0..=KEY_COUNT) => read_button_press(input, state),
        _ => Err(MirajazzError::BadData),
    }
}

fn read_button_states(states: &[u8]) -> Vec<bool> {
    let mut bools = vec![];

    for i in 0..KEY_COUNT {
        bools.push(states[i + 1] != 0);
    }

    bools
}

/// Converts opendeck key index to device key index
pub fn opendeck_to_device(key: u8) -> u8 {
    if key < KEY_COUNT as u8 {
        [10, 11, 12, 13, 14, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4][key as usize]
    } else {
        key
    }
}

/// Converts device key index to opendeck key index
pub fn device_to_opendeck(key: usize) -> usize {
    key - 1 // We have to subtract 1 from key index reported by device, because list is shifted by 1
}

fn read_button_press(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; KEY_COUNT + 1]);

    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(
            &button_states,
        )));
    }

    let pressed_index: usize = device_to_opendeck(input as usize);

    // `device_to_opendeck` is 0-based, so add 1
    // I'll probably have to refactor all of this off-by-one stuff in this file, but that's a future me problem
    button_states[pressed_index + 1] = state;

    Ok(DeviceInput::ButtonStateChange(read_button_states(
        &button_states,
    )))
}
