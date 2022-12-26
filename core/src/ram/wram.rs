use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
    svbk: u8,
}

impl Default for WRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x1000 * 8].into_boxed_slice(),
            svbk: 0x00,
        }
    }
}

impl WRAM {
    fn bank(&self) -> usize {
        (self.svbk & 0x7) as usize
    }

    fn bank_addr(&self, address: u16) -> usize {
        let bank = self.bank().max(1);
        (address as usize) - 0xd000 + (bank * 0x1000)
    }
}

impl Device for WRAM {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xc000..=0xcfff => Ok(self.data[(address as usize) - 0xc000]),
                0xd000..=0xdfff => Ok(self.data[self.bank_addr(address)]),
                0xe000..=0xfdff => self.read(address - 0xe000 + 0xc000),
                0xff70 => Ok(self.svbk),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xc000..=0xcfff => self.data[(address as usize) - 0xc000] = data,
                0xd000..=0xdfff => self.data[self.bank_addr(address)] = data,
                0xe000..=0xfdff => self.write(address - 0xe000 + 0xc000, data)?,
                0xff70 => self.svbk = data,
            }
        }
        Ok(())
    }

    #[cfg(todo)]
    fn read_exact(&self, address: u16, buf: &mut [u8]) -> Result<(), ReadError> {
        match address {
            #[cfg(not(feature = "cgb"))]
            0xc000..=0xdfff if (address as usize) + buf.len() <= 0xdfff => {
                let offset = address as usize - 0xc000;
                buf.copy_from_slice(&self.data[offset..(offset + buf.len())]);
                Ok(())
            }
            #[cfg(feature = "cgb")]
            0xc000..=0xcfff if (address as usize) + buf.len() <= 0xcfff => {
                let offset = address as usize - 0xc000;
                buf.copy_from_slice(&self.data[offset..(offset + buf.len())]);
                Ok(())
            }
            #[cfg(feature = "cgb")]
            0xd000..=0xdfff if (address as usize) + buf.len() <= 0xdfff => {
                let offset = self.bank_addr(address);
                buf.copy_from_slice(&self.data[offset..(offset + buf.len())]);
                Ok(())
            }
            #[cfg(feature = "cgb")]
            0xc000..=0xdfff if (address as usize) + buf.len() <= 0xdfff => {
                // TODO read using two memcpy
                self.read_exact_fallback(address, buf)
            }
            _ => self.read_exact_fallback(address, buf),
        }
    }
}

#[cfg(test)]
mod test {}
