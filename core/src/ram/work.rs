use crate::{
    device::{Device, Result},
    error::Error,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const OFFSET: usize = 0xc000;
const SIZE: usize = 0x2000;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
}

impl Default for WorkRAM {
    fn default() -> Self {
        Self {
            data: vec![0; SIZE].into_boxed_slice(),
        }
    }
}

impl Device for WorkRAM {
    const DEBUG_NAME: &'static str = "Work RAM";

    fn read(&self, address: u16) -> Result<u8> {
        match address {
            0xc000..=0xdfff => Ok(self.data[address as usize - OFFSET]),
            _ => Err(Error::InvalidAddr(address)),
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        match address {
            0xc000..=0xdfff => self.data[address as usize - OFFSET] = data,
            _ => return Err(Error::InvalidAddr(address)),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn work_ram() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xc000, 1).unwrap();
        emu.write(0xcfff, 2).unwrap();

        assert_eq!(
            [1, 2],
            [
                emu.work_ram.read(0xc000).unwrap(),
                emu.work_ram.read(0xcfff).unwrap()
            ]
        );
    }
}
