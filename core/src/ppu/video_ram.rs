use crate::dev::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const OFFSET: usize = 0x8000;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VideoRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
}

impl Default for VideoRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x2000].into_boxed_slice(),
        }
    }
}

impl Device for VideoRAM {
    fn debug_name() -> &'static str {
        "Video RAM"
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9fff => self.data[address as usize - OFFSET],
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x8000..=0x9fff => self.data[address as usize - OFFSET] = data,
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use super::VideoRAM;
    use crate::dev::Device;

    #[test]
    fn video_ram() {
        let mut vram = VideoRAM::default();

        vram.write(0x8000, 1);
        vram.write(0x9fff, 2);

        assert_eq!([1, 2], [vram.read(0x8000), vram.read(0x9fff)]);
    }
}
