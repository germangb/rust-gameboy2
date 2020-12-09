use crate::{
    dev::{invalid_read, invalid_write, Address, Device},
    ppu::{lcd::LcdBuffer, oam::OamTable, video_ram::VideoRam},
    EmulationStep, Update,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod io;
pub mod lcd;
mod oam;
mod video_ram;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PPU {
    #[cfg_attr(feature = "serde", serde(skip))]
    buffer: Box<LcdBuffer>,
    oam_table: OamTable,
    video_ram: VideoRam,
}

impl PPU {
    pub fn buffer(&self) -> &LcdBuffer {
        &self.buffer
    }

    /// Perform OAM DMA transfer
    pub fn oam_dma_transfer(&mut self, from: Address) {
        let src = from..=from | 0x9f;
        let dst = 0xfe00..0xfe9f;

        for (src, dst) in src.zip(dst) {
            let data = self.read(src);
            self.write(dst, data);
        }
    }
}

impl Update for PPU {
    fn update(&mut self, step: &EmulationStep) {
        todo!()
    }
}

impl Device for PPU {
    fn debug_name() -> Option<&'static str> {
        Some("Pixel Processing Unit")
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9fff => self.video_ram.read(address),
            0xfe00..=0xfe9f => self.oam_table.read(address),
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x8000..=0x9fff => self.video_ram.write(address, data),
            0xfe00..=0xfe9f => self.oam_table.write(address, data),
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn oam_dma_transfer() {
        todo!()
    }
}
