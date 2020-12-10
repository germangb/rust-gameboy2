use crate::device::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Irq {
    ie: u8,
    fi: u8,
}

impl Device for Irq {
    fn debug_name() -> &'static str {
        "IRQ"
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0xffff => self.ie,
            0xff0f => self.fi,
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xffff => self.ie = data,
            0xff0f => self.fi = data,
            _ => invalid_write(address),
        }
    }
}
