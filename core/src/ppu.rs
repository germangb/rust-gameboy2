use crate::{
    device::{invalid_read, invalid_write, Address, Device},
    irq::Request,
    ppu::{
        io::{lcdc::LCDC, stat::STAT, Palette, Scroll, Window},
        lcd::{Display, DisplaySerde, Pixel},
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

    fn clear_display(&mut self) {
        self.display.iter_mut().for_each(|p| *p = lcd::PALETTE[0]);
    }

    // render the given line to the display buffer
    fn draw_scanline(&mut self, ly: u8) {
        let display_offset = lcd::WIDTH * (ly as usize);
        let Scroll { scy, scx } = self.scroll;
        let Window { wy, wx } = self.window;

        for dot in 0..lcd::WIDTH {
            let row = scy.wrapping_add(ly);
            let col = scx.wrapping_add(dot as u8);

            let color = if self.lcdc.window_enable() {
                // WX - Window X Position minus 7
                let dot = (dot + 7) as u8;

                if ly >= wy && dot >= wx {
                    let row = ly - wy;
                    let col = dot - wx;

                    self.decode_map(row, col, self.lcdc.window_map_select())
                } else {
                    self.decode_map(row, col, self.lcdc.bg_map_select())
                }
            } else {
                self.decode_map(row, col, self.lcdc.bg_map_select())
            };

            self.display[display_offset + dot as usize] = color;
        }
    }

    // decode bg and window pixel
    fn decode_map(&mut self, row: u8, col: u8, map: u16) -> Pixel {
        // map pixel
        let row = row as u16;
        let col = col as u16;
        // tile pixel
        let prow = row % 8;
        let pcol = 7 - col % 8;
        // tile index
        let tile_index_address = map + 32 * (row / 8) + (col / 8);
        let tile_index = self.read(tile_index_address);
        // tile data (each row takes 2 bytes so a full 8x8 tile consists of 16 bytes)
        let tile_data_select = self.lcdc.bg_window_data_select();
        let tile_data_address = if tile_data_select == 0x8800 {
            let tile_index: isize =
                128 + unsafe { std::mem::transmute::<_, i8>(tile_index) } as isize;
            tile_data_select + (tile_index as u16 * 16) + (prow * 2)
        } else {
            tile_data_select + (tile_index as u16 * 16) + (prow * 2)
        };
        //let tile_data_address = tile_data_select + (tile_index * 16) + (prow * 2);
        let tile_data_lo = (self.read(tile_data_address) >> pcol) & 1;
        let tile_data_hi = (self.read(tile_data_address + 1) >> pcol) & 1;
        // decode color
        let pal_index = (tile_data_hi << 1) | tile_data_lo;
        let color_index = (self.palette.bgp >> (2 * pal_index)) & 0b11;
        lcd::PALETTE[color_index as usize]
    }
}

impl Update for PPU {
    fn update(&mut self, step: &EmulationStep, request: &mut Request) {
        let line = self.stat.ly();

        self.stat.update(step, request);

        if line < 144 && line != self.stat.ly() {
            self.draw_scanline(line);
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

                if !self.lcdc.lcd_on() {
                    self.clear_display()
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
