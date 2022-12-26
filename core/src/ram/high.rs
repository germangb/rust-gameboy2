use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const OFFSET: usize = 0xff80;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HighRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
}

impl Default for HighRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x7f].into_boxed_slice(),
        }
    }
}

impl Device for HighRAM {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff80..=0xfffe => Ok(self.data[address as usize - OFFSET]),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff80..=0xfffe => self.data[address as usize - OFFSET] = data,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, LR35902};

    #[test]
    fn high_ram() {
        let mut emu = LR35902::new(NoCartridge);

        emu.write(0xff80, 1).unwrap();
        emu.write(0xfffe, 2).unwrap();

        assert_eq!(
            [1, 2],
            [
                emu.high_ram.read(0xff80).unwrap(),
                emu.high_ram.read(0xfffe).unwrap()
            ]
        );
    }
}
