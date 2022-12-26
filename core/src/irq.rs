use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Flags: u8 {
        const VBLANK   = 0x01;
        const LCD_STAT = 0x02;
        const TIMER    = 0x04;
        const SERIAL   = 0x08;
        const JOYPAD   = 0x10;
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IRQ {
    pub fi: Flags,
    pub ie: Flags,
}

impl Device for IRQ {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff0f => Ok(self.fi.bits),
                0xffff => Ok(self.ie.bits),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff0f => self.fi = Flags::from_bits(data).ok_or(WriteError::InvalidData(address, data))?,
                0xffff => self.ie = Flags::from_bits(data).ok_or(WriteError::InvalidData(address, data))?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {}
