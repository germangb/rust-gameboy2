use crate::device::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const OFFSET: usize = 0xc000;
const SIZE: usize = 0x2000;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

    fn read(&self, address: u16) -> u8 {
        match address {
            0xc000..=0xdfff => self.data[address as usize - OFFSET],
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xc000..=0xdfff => self.data[address as usize - OFFSET] = data,
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn work_ram() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xc000, 1);
        emu.write(0xcfff, 2);

        assert_eq!(
            [1, 2],
            [emu.work_ram.read(0xc000), emu.work_ram.read(0xcfff)]
        );
    }
}
