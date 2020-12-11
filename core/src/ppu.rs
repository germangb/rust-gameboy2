use crate::{
    device::{invalid_read, invalid_write, Address, Device},
    irq::Request,
    ppu::{
        io::{lcdc::LCDC, stat::STAT, Palette, Scroll, Window},
        lcd::{Display, DisplaySerde},
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
    display: Box<DisplaySerde>,
    oam_table: OAMTable,
    video_ram: VideoRAM,
    lcdc: LCDC,
    stat: STAT,
    scroll: Scroll,
    window: Window,
    palette: Palette,
}

impl PPU {
    pub fn display(&self) -> &Display {
        &self.display
    }

    fn clear(&mut self) {
        self.display.iter_mut().for_each(|p| *p = 0);
    }

    // render the given line to the display buffer
    fn draw_line(&mut self, ly: u8) {
        let row = ly.wrapping_add(self.scroll.scy);

        let offset = lcd::WIDTH * (ly as usize);
        for pixel in &mut self.display[offset..offset + lcd::WIDTH] {
            if row > 143 {
                *pixel = 0x0;
            } else {
                *pixel = 0xff00ff;
            }
        }
    }
}

impl Update for PPU {
    fn update(&mut self, step: &EmulationStep, request: &mut Request) {
        let line = self.stat.ly();

        self.stat.update(step, request);

        if line < 144 && line != self.stat.ly() {
            self.draw_line(line);
        }
    }
}

impl Device for PPU {
    const DEBUG_NAME: &'static str = "Pixel Processing Unit";

    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9fff => self.video_ram.read(address),
            0xfe00..=0xfe9f => self.oam_table.read(address),
            0xff40 => self.lcdc.read(address),
            0xff41 => self.stat.read(address),
            0xff42..=0xff43 => self.scroll.read(address),
            0xff44..=0xff45 => self.stat.read(address),
            0xff47..=0xff49 => self.palette.read(address),
            0xff4a..=0xff4b => self.window.read(address),
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x8000..=0x9fff => self.video_ram.write(address, data),
            0xfe00..=0xfe9f => self.oam_table.write(address, data),
            0xff40 => {
                self.lcdc.write(address, data);

                // turn off display
                if !self.lcdc.lcd_on() {
                    self.clear()
                }
            }
            0xff41 => self.stat.write(address, data),
            0xff42..=0xff43 => self.scroll.write(address, data),
            0xff44..=0xff45 => self.stat.write(address, data),
            0xff47..=0xff49 => self.palette.write(address, data),
            0xff4a..=0xff4b => self.window.write(address, data),
            _ => invalid_write(address),
        }
    }
}
