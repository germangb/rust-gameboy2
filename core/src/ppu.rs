use crate::{
    dev::{invalid_read, invalid_write, Address, Device},
    ppu::{
        io::{lcdc::LCDC, stat::STAT},
        lcd::LcdBuffer,
        oam::OAMTable,
        video_ram::VideoRAM,
    },
    EmulationStep, Update,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod io;
pub mod lcd;
mod oam;
mod video_ram;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PPU {
    #[cfg_attr(feature = "serde", serde(skip))]
    buffer: Box<LcdBuffer>,
    oam_table: OAMTable,
    video_ram: VideoRAM,
    lcdc: LCDC,
    stat: STAT,
}

impl PPU {
    /// Returns current state of the display.
    pub fn display(&self) -> &[lcd::Pixel; lcd::WIDTH * lcd::HEIGHT] {
        &self.buffer.0
    }
}

impl Update for PPU {
    fn update(&mut self, step: &EmulationStep) {
        todo!()
    }
}

impl Device for PPU {
    fn debug_name() -> &'static str {
        "Pixel Processing Unit"
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9fff => self.video_ram.read(address),
            0xfe00..=0xfe9f => self.oam_table.read(address),
            0xff40 => self.lcdc.read(address),
            0xff41 => self.stat.read(address),
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x8000..=0x9fff => self.video_ram.write(address, data),
            0xfe00..=0xfe9f => self.oam_table.write(address, data),
            0xff40 => self.lcdc.write(address, data),
            0xff41 => self.stat.write(address, data),
            _ => invalid_write(address),
        }
    }
}
