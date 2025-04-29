use mirajazz::types::{ImageFormat, ImageMirroring, ImageMode, ImageRotation};

pub const ROW_COUNT: usize = 3;
pub const COL_COUNT: usize = 6;
pub const KEY_COUNT: usize = ROW_COUNT * COL_COUNT;

pub const IMAGE_FORMAT: ImageFormat = ImageFormat {
    mode: ImageMode::JPEG,
    size: (85, 85),
    rotation: ImageRotation::Rot90,
    mirror: ImageMirroring::Both,
};

#[derive(Debug, Clone)]
pub enum Kind {
    HSV293S,
    AKP153,
    AKP153E,
    AKP153R,
}

pub const AJAZZ_VID: u16 = 0x0300;
pub const MIRABOX_VID: u16 = 0x5548;

pub const HSV293S_PID: u16 = 0x6670;

pub const AKP153_PID: u16 = 0x6674;
pub const AKP153E_PID: u16 = 0x1010;
pub const AKP153R_PID: u16 = 0x1020;

impl Kind {
    /// Matches devices VID+PID pairs to correct kinds
    pub fn from_vid_pid(vid: u16, pid: u16) -> Option<Self> {
        match vid {
            AJAZZ_VID => match pid {
                AKP153_PID => Some(Kind::AKP153),
                AKP153E_PID => Some(Kind::AKP153E),
                AKP153R_PID => Some(Kind::AKP153R),
                _ => None,
            },

            MIRABOX_VID => match pid {
                HSV293S_PID => Some(Kind::HSV293S),
                _ => None,
            },

            _ => None,
        }
    }

    /// Returns true for devices that emitting two events per key press, instead of one
    /// Currently none of the devices from this family support that
    pub fn supports_both_states(&self) -> bool {
        false
    }

    /// There is no point relying on manufacturer/device names reported by the USB stack,
    /// so we return custom names for all the kinds of devices
    pub fn human_name(&self) -> String {
        match &self {
            Self::AKP153 => "Ajazz AKP153",
            Self::AKP153E => "Ajazz AKP153E",
            Self::AKP153R => "Ajazz AKP153R",
            Self::HSV293S => "Mirabox HSV293S",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct CandidateDevice {
    pub id: String,
    pub vid: u16,
    pub pid: u16,
    pub serial: String,
    pub kind: Kind,
}
