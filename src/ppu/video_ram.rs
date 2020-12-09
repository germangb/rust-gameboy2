use crate::dev::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VideoRam {
    data: Box<[u8; 0x2000]>,
}

impl Default for VideoRam {
    fn default() -> Self {
        Self {
            data: Box::new([0; 0x2000]),
        }
    }
}

impl Device for VideoRam {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9fff => self.data[address as usize - 0x800],
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x8000..=0x9fff => self.data[address as usize - 0x800] = data,
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ppu::video_ram::VideoRam;

    #[test]
    fn video_ram() {
        let mut vram = VideoRam::default();
    }
}
