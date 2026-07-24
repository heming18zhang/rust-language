use core::sync::atomic::AtomicU8;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum LedMode {
    Off = 0,
    On = 1,
    Blink = 2,
}

impl LedMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => LedMode::Off,
            1 => LedMode::On,
            2 => LedMode::Blink,
            _ => LedMode::Off,
        }
    }
}

pub static LED_MODE: AtomicU8 =
    AtomicU8::new(LedMode::Blink as u8);