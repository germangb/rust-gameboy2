use crate::{
    device::{Device, Result},
    error::Error,
    irq,
    ppu::{
        io::{lcdc::LCDC, stat::STAT, Palette, Scroll, Window},
        lcd::{Display, DisplaySerde, Pixel},
        oam::{Entry, Flags, OAM},
    },
    ram::VideoRAM,
    Update,
};
use log::{info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "debug")]
mod debug;
mod io;
pub mod lcd;
mod oam;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
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
    #[cfg(feature = "debug")]
    debug_overlays: bool,
}

impl PPU {
    pub fn display(&self) -> &Display {
        &self.display
    }

    fn clear_display(&mut self) {
        self.display.iter_mut().for_each(|p| *p = lcd::PALETTE[0]);
    }

    #[cfg(not(feature = "debug"))]
    pub fn set_debug_overlays(&mut self, _: bool) {
        warn!("Emulator is not built with debug overlays.");
    }

    #[cfg(feature = "debug")]
    pub fn set_debug_overlays(&mut self, b: bool) {
        self.debug_overlays = b;
    }

    pub fn finish(&mut self) {
        #[cfg(feature = "debug")]
        if self.debug_overlays {
            self.draw_debug_overlays();
        }
    }

    #[cfg(feature = "debug")]
    fn draw_debug_overlays(&mut self) {
        use embedded_graphics::{
            fonts::Text,
            pixelcolor::Rgb888,
            prelude::*,
            primitives::Line,
            style::{PrimitiveStyle, TextStyle},
        };
        use embedded_picofont::FontPico;

        // draw LYC
        let lyc = self.stat.lyc() as _;
        Line::new(Point::new(0, lyc), Point::new(lcd::WIDTH as _, lyc))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
            .draw(self.display.as_mut())
            .unwrap();

        // text
        let draw = &[
            format!("SCY={} SCX={}", self.scroll.scy, self.scroll.scx),
            format!("WY={} WX={}", self.window.wy, self.window.wx),
            format!(
                "OBP0={:08b} OBP1={:08b} BGP={:08b}",
                self.palette.obp0, self.palette.obp1, self.palette.bgp
            ),
            format!("LCDC={:08b}", self.lcdc.bits()),
            format!(
                "STAT={:08b} LY={} LYC={}",
                self.stat.stat(),
                self.stat.ly(),
                self.stat.lyc(),
            ),
        ];

        for (y, item) in draw.iter().enumerate() {
            Text::new(
                &item,
                Point::new(0, lcd::HEIGHT as i32 - 7 - (y as i32) * 7),
            )
            .into_styled(TextStyle::new(FontPico, Rgb888::WHITE))
            .draw(self.display.as_mut())
            .unwrap();
        }
    }

    // render the given line to the display buffer
    fn draw_scanline(&mut self, ly: u8) -> Result<()> {
        let display_offset = lcd::WIDTH * (ly as usize);
        let Scroll { scy, scx } = self.scroll;
        let Window { wy, wx } = self.window;

        for dot in 0..lcd::WIDTH {
            let row = scy.wrapping_add(ly) as u16;
            let col = scx.wrapping_add(dot as u8) as u16;

            let color = if self.lcdc.window_enable() {
                // WX - Window X Position minus 7
                let dot = (dot + 7) as u8;

                if ly >= wy && dot >= wx {
                    let row = (ly - wy) as u16;
                    let col = (dot - wx) as u16;

                    let mut color = self.decode_bg_win(row, col, self.lcdc.window_map_select())?;

                    #[cfg(feature = "debug")]
                    if self.debug_overlays {
                        color = debug::mix(color, 0x00ffff, 0.5);
                    }

                    color
                } else {
                    self.decode_bg_win(row, col, self.lcdc.bg_map_select())?
                }
            } else {
                self.decode_bg_win(row, col, self.lcdc.bg_map_select())?
            };

            self.display[display_offset + dot as usize] = color;
        }

        if !self.lcdc.obj_enable() {
            return Ok(());
        }

        let ly = ly as i16;

        for i in 0..40 {
            let Entry { y, x, tile, flags } = self.oam.table()[i];
            let y = (y as i16) - 16;
            let x = (x as i16) - 8;
            let size = if self.lcdc.obj_size() { 16 } else { 8 };

            if ly >= y && ly < y + size {
                for dot in x.max(0)..(x + 8).min(lcd::WIDTH as _) {
                    let mut row = ly - y;
                    let mut col = dot - x;
                    if flags.contains(Flags::Y_FLIP) {
                        row = size - row - 1;
                    }
                    if !flags.contains(Flags::X_FLIP) {
                        col = 7 - col;
                    }

                    let pal = if flags.contains(Flags::PAL_NUMBER) {
                        self.palette.obp1
                    } else {
                        self.palette.obp0
                    };

                    let offset = display_offset + dot as usize;

                    if !flags.contains(Flags::OBJ_TO_BG_PRIORITY)
                        || self.display[offset] == lcd::PALETTE[0]
                    {
                        let mut color = self
                            .decode_tile(row as u16, col as u16, tile, 0x8000, pal, true)?
                            .unwrap_or(self.display[offset]);

                        #[cfg(feature = "debug")]
                        if self.debug_overlays {
                            color = if self.lcdc.obj_size() {
                                debug::mix(color, debug::DEBUG_OBJ_DOUBLE, 0.5)
                            } else {
                                debug::mix(color, debug::DEBUG_OBJ, 0.5)
                            };
                        };

                        self.display[offset] = color;
                    }
                }
            }
        }

        Ok(())
    }

    // decode bg and window pixel
    fn decode_bg_win(&self, row: u16, col: u16, map: u16) -> Result<Pixel> {
        // tile pixel
        let prow = row % 8;
        let pcol = 7 - col % 8;
        // tile index
        let tile_index_address = map + 32 * (row / 8) + (col / 8);
        let tile_index = self.read(tile_index_address)?;

        Ok(self
            .decode_tile(
                prow,
                pcol,
                tile_index,
                self.lcdc.bg_window_data_select(),
                self.palette.bgp,
                false,
            )?
            .unwrap())
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
    ) -> Result<Option<Pixel>> {
        // tile data (each row takes 2 bytes so a full 8x8 tile consists of 16 bytes)
        let tile_data_offset = decode_tile_data_offset(index, data_select);
        let data_address = data_select + tile_data_offset * 16 + (prow * 2);
        let tile_data_lo = (self.read(data_address)? >> pcol) & 1;
        let tile_data_hi = (self.read(data_address + 1)? >> pcol) & 1;
        // decode color
        let pal_index = (tile_data_hi << 1) | tile_data_lo;
        if opacity && pal_index == 0 {
            return Ok(None);
        }
        let color_index = (pal >> (2 * pal_index)) & 0b11;
        Ok(Some(lcd::PALETTE[color_index as usize]))
    }
}

// decode tile index
fn decode_tile_data_offset(index: u8, data_select: u16) -> u16 {
    if data_select == 0x8800 {
        let index: i32 = 128 + unsafe { std::mem::transmute::<_, i8>(index) } as i32;
        index as u16
    } else {
        index as u16
    }
}

impl Update for PPU {
    fn update(&mut self, ticks: u64, flags: &mut irq::Flags) {
        let line = self.stat.ly();

        if self.stat.update(ticks, flags) {
            self.draw_scanline(line).unwrap();
        }
    }
}

impl Device for PPU {
    const DEBUG_NAME: &'static str = "Pixel Processing Unit";

    fn read(&self, address: u16) -> Result<u8> {
        match address {
            0x8000..=0x9fff => self.video_ram.read(address),
            0xfe00..=0xfe9f => self.oam.read(address),
            0xff40 => self.lcdc.read(address),
            0xff41 => self.stat.read(address),
            0xff42..=0xff43 => self.scroll.read(address),
            0xff44..=0xff45 => self.stat.read(address),
            0xff47..=0xff49 => self.palette.read(address),
            0xff4a..=0xff4b => self.window.read(address),
            _ => Err(Error::InvalidAddr(address)),
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        match address {
            0x8000..=0x9fff => self.video_ram.write(address, data),
            0xfe00..=0xfe9f => self.oam.write(address, data),
            0xff40 => {
                self.lcdc.write(address, data)?;

                if !self.lcdc.lcd_on() {
                    info!("Turn off LCD display");

                    self.clear_display()
                }

                Ok(())
            }
            0xff41 => self.stat.write(address, data),
            0xff42..=0xff43 => self.scroll.write(address, data),
            0xff44..=0xff45 => self.stat.write(address, data),
            0xff47..=0xff49 => self.palette.write(address, data),
            0xff4a..=0xff4b => self.window.write(address, data),
            _ => return Err(Error::InvalidAddr(address)),
        }
    }
}
