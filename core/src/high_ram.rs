use crate::device::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const OFFSET: usize = 0xff80;

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
    fn debug_name() -> &'static str {
        "High Ram"
    }

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
    use super::HighRAM;
    use crate::device::Device;

    #[test]
    fn high_ram() {
        let mut hram = HighRAM::default();

        hram.write(0xff80, 1);
        hram.write(0xfffe, 2);

        assert_eq!([1, 2], [hram.read(0xff80), hram.read(0xfffe)]);
    }
}
