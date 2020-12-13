use crate::{
    device::{invalid_read, invalid_write, Address, Device},
    irq::Request,
    ppu::{
        io::{lcdc::LCDC, stat::STAT, Palette, Scroll, Window},
        lcd::{Display, DisplaySerde, Pixel},
        oam::{Entry, Flags, OAM},
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
    oam: OAM,
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

        if !self.lcdc.obj_enable() {
            return;
        }

        for i in 0..40 {
            let Entry { y, x, tile, flags } = self.oam.table()[i];
            let y = y.wrapping_sub(16);
            let x = x.wrapping_sub(8);
            let size = if self.lcdc.obj_size() { 16 } else { 8 };

            if ly >= y && ly < y + size {
                for dot in x..x + 8 {
                    let mut row = (ly - y) as u16;
                    let mut col = (dot - x) as u16;
                    if flags.contains(Flags::Y_FLIP) {
                        row = 7 - row;
                    }
                    if !flags.contains(Flags::X_FLIP) {
                        col = 7 - col;
                    }

                    let pal = if flags.contains(Flags::PAL_NUMBER) {
                        self.palette.obp1
                    } else {
                        self.palette.obp0
                    };

                    if let Some(color) = self.decode_tile(row, col, tile, 0x8000, pal, true) {
                        if !flags.contains(Flags::OBJ_TO_BG_PRIORITY) {
                            self.display[display_offset + dot as usize] = color;
                        }
                    }
                }
            }
        }
    }

    // decode bg and window pixel
    fn decode_map(&self, row: u8, col: u8, map: u16) -> Pixel {
        // map pixel
        let row = row as u16;
        let col = col as u16;
        // tile pixel
        let prow = row % 8;
        let pcol = 7 - col % 8;
        // tile index
        let tile_index_address = map + 32 * (row / 8) + (col / 8);
        let tile_index = self.read(tile_index_address);

        self.decode_tile(
            prow,
            pcol,
            tile_index,
            self.lcdc.bg_window_data_select(),
            self.palette.bgp,
            false,
        )
        .unwrap()
    }

    // decode tile pixel color (None if reansparent)
    fn decode_tile(
        &self,
        prow: u16,
        pcol: u16,
        index: u8,
        data_select: u16,
        pal: u8,
        opacity: bool,
    ) -> Option<Pixel> {
        // tile data (each row takes 2 bytes so a full 8x8 tile consists of 16 bytes)
        let data_address = if data_select == 0x8800 {
            let index: isize = 128 + unsafe { std::mem::transmute::<_, i8>(index) } as isize;
            data_select + (index as u16 * 16) + (prow * 2)
        } else {
            data_select + (index as u16 * 16) + (prow * 2)
        };
        //let tile_data_address = tile_data_select + (tile_index * 16) + (prow * 2);
        let tile_data_lo = (self.read(data_address) >> pcol) & 1;
        let tile_data_hi = (self.read(data_address + 1) >> pcol) & 1;
        // decode color
        let pal_index = (tile_data_hi << 1) | tile_data_lo;
        if opacity && pal_index == 0 {
            return None;
        }
        let color_index = (pal >> (2 * pal_index)) & 0b11;
        Some(lcd::PALETTE[color_index as usize])
    }
}

impl Update for PPU {
    fn update(&mut self, step: &EmulationStep, request: &mut Request) {
        let line = self.stat.ly();

        #[cfg(fixme)]
        {
            for _ in 0..step.clock_ticks / 4 {
                self.stat.update(4, request);
            }
            self.stat.update(step.clock_ticks % 4, request);
        }
        self.stat.update(step.clock_ticks, request);

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
            0xfe00..=0xfe9f => self.oam.read(address),
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
            0xfe00..=0xfe9f => self.oam.write(address, data),
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
