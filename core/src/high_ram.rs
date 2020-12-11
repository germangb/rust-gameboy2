use crate::device::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const OFFSET: usize = 0xff80;
const SIZE: usize = 0x27;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
    const DEBUG_NAME: &'static str = "High Ram";

    fn read(&self, address: u16) -> u8 {
        match address {
            0xff80..=0xfffe => self.data[address as usize - OFFSET],
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff80..=0xfffe => self.data[address as usize - OFFSET] = data,
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn high_ram() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff80, 1);
        emu.write(0xfffe, 2);

        assert_eq!(
            [1, 2],
            [emu.high_ram.read(0xff80), emu.high_ram.read(0xfffe)]
        );
    }
}
