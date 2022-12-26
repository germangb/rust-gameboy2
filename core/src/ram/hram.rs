use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
}

impl Default for HRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x7f].into_boxed_slice(),
        }
    }
}

impl Device for HRAM {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff80..=0xfffe => Ok(self.data[address as usize - 0xff80]),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff80..=0xfffe => self.data[address as usize - 0xff80] = data,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {}
