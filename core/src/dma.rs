use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OAMDMA {}

impl Device for OAMDMA {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff46 => Ok(0x00),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        Err(WriteError::UnknownAddr(address, data))
    }
}

#[cfg(feature = "cgb")]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VRAMDMA {
    pub hdma1: u8,
    pub hdma2: u8,
    pub hdma3: u8,
    pub hdma4: u8,
}

#[cfg(feature = "cgb")]
impl Device for VRAMDMA {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff51..=0xff54 => Ok(0xff),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff51 => self.hdma1 = data,
                0xff52 => self.hdma2 = data,
                0xff53 => self.hdma3 = data,
                0xff54 => self.hdma4 = data,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {}
