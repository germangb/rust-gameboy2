#[cfg(feature = "cgb")]
use crate::ppu::io::ColorPalette as ColorPaletteIO;
#[cfg(feature = "cgb")]
use crate::ram::vram::Attributes;
use crate::{
    device::Device,
    error::{ReadError, WriteError},
    irq,
    ppu::{
        io::{Palette, Scroll, Window, LCDC, STAT},
        lcd::{BoolLineBuffer, ColorId, ColorLineBuffer, LineBuffer},
        oam::{Entry, Flags, OAM},
    },
    ram::vram::VRAM,
    Update,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::Range;
#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "lcd_debug_overlay")]
mod debug;
mod io;
mod lcd;
mod oam;

pub type Color = [u8; 4];
pub type ColorPalette = [Color; 4];

/// Height (or number of rows) in the LCD display.
pub const LCD_HEIGHT: usize = 144;

/// Width (or number of columns) in the LCD display.
pub const LCD_WIDTH: usize = 160;

/// Palette of puke-greens used for color.
pub const PALETTE: [Color; 4] = [
    lcd::color(0x7e, 0x84, 0x16),
    lcd::color(0x57, 0x7b, 0x46),
    lcd::color(0x38, 0x5d, 0x49),
    lcd::color(0x2e, 0x46, 0x3d),
];

/// A trait for types to drive output of the PPU.
pub trait LCD {
    fn output_line(&mut self, ly: u8, data: &[Color; LCD_WIDTH]);
}

#[cfg(feature = "lcd_debug_overlay")]
bitflags::bitflags! {
    /// LCD Debug overlay flags.
    #[rustfmt::skip]
    #[cfg_attr(feature = "wasm-bindgen", wasm_bindgen)]
    #[derive(Default)]
    pub struct LCDDebugOverlay: u8 {
        const TILEMAP  = 0b0001;
        const WINDOW   = 0b0010;
        const SPRITES  = 0b0100;
        const LYC      = 0b1000;
    }
}

impl LCD for () {
    fn output_line(&mut self, _ly: u8, _data: &[Color; LCD_WIDTH]) {}
}

#[derive(Debug, Default)]
pub struct PPU<O: LCD> {
    output: O,
    line: Box<LineBuffer>,
    color_line: Box<ColorLineBuffer>,
    oam: OAM,
    video_ram: VRAM,
    lcdc: LCDC,
    stat: STAT,
    scroll: Scroll,
    window: Window,
    palette: Palette,
    #[cfg(feature = "cgb")]
    color_palette: ColorPaletteIO,
    #[cfg(feature = "lcd_debug_overlay")]
    pub lcd_debug_overlay: LCDDebugOverlay,
}

impl<O: LCD> PPU<O> {
    pub(super) fn new(output: O) -> Self {
        Self {
            output,
            line: Box::new(Default::default()),
            color_line: Box::new(Default::default()),
            oam: Default::default(),
            video_ram: Default::default(),
            lcdc: Default::default(),
            stat: Default::default(),
            scroll: Default::default(),
            window: Default::default(),
            palette: Default::default(),
            #[cfg(feature = "cgb")]
            color_palette: Default::default(),
            #[cfg(feature = "lcd_debug_overlay")]
            lcd_debug_overlay: LCDDebugOverlay::empty(),
        }
    }

    pub fn output(&self) -> &O {
        &self.output
    }

    pub fn output_mut(&mut self) -> &mut O {
        &mut self.output
    }

    #[cfg(not(feature = "cgb"))]
    pub fn bgp(&self) -> &ColorPalette {
        self.palette.bgp()
    }

    #[cfg(not(feature = "cgb"))]
    pub fn obp0(&self) -> &ColorPalette {
        self.palette.obp0()
    }

    #[cfg(not(feature = "cgb"))]
    pub fn obp1(&self) -> &ColorPalette {
        self.palette.obp1()
    }

    #[cfg(feature = "cgb")]
    pub fn bgp(&self) -> &[ColorPalette; 8] {
        self.color_palette.bgp()
    }

    #[cfg(feature = "cgb")]
    pub fn obp(&self) -> &[ColorPalette; 8] {
        self.color_palette.obp()
    }

    pub(crate) fn vram(&self) -> &VRAM {
        &self.video_ram
    }

    fn clear_display(&mut self) {
        #[cfg(not(feature = "cgb"))]
        let color = PALETTE[0];
        #[cfg(feature = "cgb")]
        let color = lcd::color(0xff, 0xff, 0xff);
        let line = [color; LCD_WIDTH];
        for y in 0..LCD_HEIGHT {
            self.output.output_line(y as u8, &line);
        }
    }

    fn draw_scanline_obj(&mut self, ly: u8) {
        // draw sprites
        // TODO in gbc mode, BG tiles may take priority over sprites
        if self.lcdc.obj_enable() {
            self.draw_sprites(ly);
        }
    }

    fn draw_scanline(&mut self, ly: u8, dots: Range<usize>) {
        // When Bit 0 is cleared, both background and window become blank (white), and
        // the Window Display Bit is ignored in that case. Only Sprites may still be
        // displayed (if enabled in Bit 1).
        #[cfg(not(feature = "cgb"))]
        if self.lcdc.bg_window_priority() {
            self.draw_bg_win(ly, dots);
        }
        #[cfg(feature = "cgb")]
        self.draw_bg_win(ly, dots);
    }

    fn draw_bg_win(&mut self, ly: u8, dots: Range<usize>) {
        let Scroll { scy, scx } = self.scroll;
        let Window { wy, wx } = self.window;

        for dot in dots {
            let row = scy.wrapping_add(ly) as u16;
            let col = scx.wrapping_add(dot as u8) as u16;

            let (color, color_id, attributes) = if self.lcdc.window_enable() {
                // WX - Window X Position minus 7
                let dot = (dot + 7) as u8;

                if ly >= wy && dot >= wx {
                    let row = (ly - wy) as u16;
                    let col = (dot - wx) as u16;
                    #[allow(unused_mut)]
                    let (mut color, color_id, attributes) =
                        self.decode_bg_win(row, col, self.lcdc.window_map_select());

                    #[cfg(feature = "lcd_debug_overlay")]
                    if self.lcd_debug_overlay.contains(LCDDebugOverlay::WINDOW) {
                        color = debug::mix(color, lcd::color(0x00, 0xff, 0xff), 0.25);
                    }

                    (color, color_id, attributes)
                } else {
                    self.decode_bg_win(row, col, self.lcdc.bg_map_select())
                }
            } else {
                self.decode_bg_win(row, col, self.lcdc.bg_map_select())
            };

            self.draw_pixel(dot, color, color_id, attributes);
        }
    }

    fn draw_sprites(&mut self, ly: u8) {
        let ly = ly as i16;

        // In CGB mode, the first sprite in OAM ($FE00-$FE03) has the highest priority,
        // and so on. In Non-CGB mode, the smaller the X coordinate, the higher the
        // priority. The tie breaker (same X coordinates) is the same priority as in CGB
        // mode.
        #[cfg(feature = "cgb")]
        let oam_idx_range = (0..40).rev();
        #[cfg(not(feature = "cgb"))]
        let oam_idx_range = 0..40;

        for oam_idx in oam_idx_range {
            let Entry { y, x, index, flags } = self.oam.table()[oam_idx];
            let y = (y as i16) - 16;
            let x = (x as i16) - 8;
            let size = if self.lcdc.obj_size() { 16 } else { 8 };

            if ly >= y && ly < y + size {
                for dot in x.max(0)..(x + 8).min(LCD_WIDTH as _) {
                    // current BG | WINDOW color
                    let (color, color_id) = self.pixel(dot as _);

                    #[cfg(feature = "cgb")]
                    if color_id == 4 {
                        continue;
                    }
                    if flags.contains(Flags::OBJ_TO_BG_PRIORITY) && color_id != 0 {
                        continue;
                    }

                    let mut row = ly - y;
                    let mut col = dot - x;
                    if flags.contains(Flags::Y_FLIP) {
                        row = size - row - 1;
                    }
                    if !flags.contains(Flags::X_FLIP) {
                        col = 7 - col;
                    }

                    #[cfg(not(feature = "cgb"))]
                    let bank = 0;
                    #[cfg(not(feature = "cgb"))]
                    let color_palette = if flags.contains(Flags::PAL_NUMBER) {
                        self.palette.obp1()
                    } else {
                        self.palette.obp0()
                    };
                    #[cfg(feature = "cgb")]
                    let color_palette = &self.color_palette.obp()[flags.palette()];
                    #[cfg(feature = "cgb")]
                    let bank = flags.bank();

                    #[allow(unused_mut)]
                    let (mut color, _) = self
                        .decode_tile(
                            row as u16,
                            col as u16,
                            index,
                            bank,
                            0x8000,
                            color_palette,
                            true,
                        )
                        .unwrap_or((color, 0));

                    #[cfg(feature = "lcd_debug_overlay")]
                    if self.lcd_debug_overlay.contains(LCDDebugOverlay::SPRITES) {
                        color = debug::mix(
                            color,
                            if self.lcdc.obj_size() {
                                debug::DEBUG_OBJ_DOUBLE
                            } else {
                                debug::DEBUG_OBJ
                            },
                            0.25,
                        );
                    };

                    self.draw_pixel(dot as _, color, 0, crate::ram::vram::Attributes::empty());
                }
            }
        }
    }

    fn pixel(&self, dot: usize) -> (Color, ColorId) {
        (self.color_line[dot], self.line[dot])
    }

    fn draw_pixel(
        &mut self,
        dot: usize,
        color: Color,
        color_id: ColorId,
        attributes: crate::ram::vram::Attributes,
    ) {
        self.line[dot] = color_id;
        self.color_line[dot] = color;
    }

    // decode bg and window pixel
    fn decode_bg_win(
        &self,
        row: u16,
        col: u16,
        map: u16,
    ) -> (Color, ColorId, crate::ram::vram::Attributes) {
        // tile pixel
        let pixel_row = row % 8;
        let pixel_col = 7 - (col % 8);
        // transformed tile pixel
        #[allow(unused_mut)]
        let mut tile_pixel_row = pixel_row;
        #[allow(unused_mut)]
        let mut tile_pixel_col = pixel_col;
        // tile index
        let tile_index_address = map + 32 * (row / 8) + (col / 8);
        let tile_index = self.video_ram.data(0, tile_index_address);

        #[cfg(feature = "cgb")]
        let attributes = self.video_ram.attributes(tile_index_address);
        #[cfg(feature = "cgb")]
        if attributes.contains(Attributes::HORIZONTAL_FLIP) {
            tile_pixel_col = 7 - tile_pixel_col;
        }
        #[cfg(feature = "cgb")]
        if attributes.contains(Attributes::VERTICAL_FLIP) {
            tile_pixel_row = 7 - tile_pixel_row;
        }

        #[cfg(not(feature = "cgb"))]
        let color_palette = self.palette.bgp();
        #[cfg(not(feature = "cgb"))]
        let bank = 0;
        #[cfg(feature = "cgb")]
        let color_palette = &self.color_palette.bgp()[attributes.palette()];
        #[cfg(feature = "cgb")]
        let bank = attributes.bank();

        #[allow(unused_mut)]
        let (mut color, mut color_id) = self
            .decode_tile(
                tile_pixel_row,
                tile_pixel_col,
                tile_index,
                bank,
                self.lcdc.bg_window_data_select(),
                &color_palette,
                false,
            )
            .unwrap();

        // When Bit 7 is set, the corresponding BG tile will have priority above all
        // OBJs (regardless of the priority bits in OAM memory). There's also a Master
        // Priority flag in LCDC register Bit 0 which overrides all other priority bits
        // when cleared.
        #[cfg(feature = "cgb")]
        if attributes.contains(Attributes::BG_OAM_PRIPRITY) && color_id != 0 {
            color_id = 4;
            #[cfg(feature = "lcd_debug_overlay")]
            if self.lcd_debug_overlay.contains(LCDDebugOverlay::TILEMAP) {
                color = debug::mix(color, lcd::color(0xff, 0x00, 0xff), 0.25);
            }
        }

        #[cfg(feature = "lcd_debug_overlay")]
        if self.lcd_debug_overlay.contains(LCDDebugOverlay::TILEMAP)
            && (pixel_row == 0 || pixel_col == 0)
        {
            color = debug::mix(color, lcd::color(0x00, 0x00, 0x00), 0.25);
        }

        #[cfg(feature = "cgb")]
        return (color, color_id, attributes);
        #[cfg(not(feature = "cgb"))]
        return (color, color_id, crate::ram::vram::Attributes::empty());
    }

    // decode tile pixel color (None if transparent)
    fn decode_tile(
        &self,
        pixel_row: u16,
        pixel_col: u16,
        tile_index: u8,
        bank: usize,
        data_select: u16,
        color_palette: &[Color; 4],
        opacity: bool,
    ) -> Option<(Color, ColorId)> {
        // tile data (each row takes 2 bytes so a full 8x8 tile consists of 16 bytes)
        let tile_data_offset = decode_tile_data_offset(tile_index, data_select);
        let data_address = data_select + tile_data_offset * 16 + (pixel_row * 2);
        let tile_data_lo = (self.video_ram.data(bank, data_address) >> pixel_col) & 1;
        let tile_data_hi = (self.video_ram.data(bank, data_address + 1) >> pixel_col) & 1;
        // decode color
        let pal_index = (tile_data_hi << 1) | tile_data_lo;
        if opacity && pal_index == 0 {
            return None;
        }
        let color_id = pal_index;
        Some((color_palette[pal_index as usize], color_id as _))
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

impl<O: LCD> Update for PPU<O> {
    fn update(&mut self, ticks: u64, flags: &mut irq::Flags) {
        let ly = self.stat.ly();
        let mode = self.stat.mode();
        let dots = self.stat.dots();

        // TODO(german) should not update LCD state if LCD is off...
        self.stat.update(ticks, flags);

        // if we transition LCD_TRANSTER -> HBALNK, draw the rest of the scanline
        if matches!(
            (mode, self.stat.mode()),
            (io::stat::Mode::PIXEL0, io::stat::Mode::HBLANK),
        ) {
            self.draw_scanline(ly, 0..LCD_WIDTH);
            self.draw_scanline_obj(ly);
            #[cfg(feature = "lcd_debug_overlay")]
            if self.lcd_debug_overlay.contains(LCDDebugOverlay::LYC)
                && self.stat.lyc_hist[ly as usize]
            {
                for i in (0..LCD_WIDTH).filter(|i| *i % 2 == (ly % 2) as usize) {
                    self.color_line[i] =
                        debug::mix(self.color_line[i], lcd::color(0, 0xff, 0), 0.75);
                }
            }
            self.output.output_line(ly, &self.color_line.as_ref().0);
        }
    }
}

impl<O: LCD> Device for PPU<O> {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0x8000..=0x9fff => self.video_ram.read(address),
                0xfe00..=0xfe9f => self.oam.read(address),
                0xff40 => self.lcdc.read(address),
                0xff41 => self.stat.read(address),
                0xff42..=0xff43 => self.scroll.read(address),
                0xff44..=0xff45 => self.stat.read(address),
                0xff47..=0xff49 => self.palette.read(address),
                0xff4a..=0xff4b => self.window.read(address),
                0xff4f => self.video_ram.read(address),
                #[cfg(feature = "cgb")]
                0xff68..=0xff6b => self.color_palette.read(address),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0x8000..=0x9fff => self.video_ram.write(address, data),
                0xfe00..=0xfe9f => self.oam.write(address, data),
                0xff40 => {
                    self.lcdc.write(address, data)?;
                    if !self.lcdc.lcd_on() {
                        self.clear_display();
                        self.stat.reset();
                        self.lcdc.reset();
                    }
                    Ok(())
                }
                0xff41 => self.stat.write(address, data),
                0xff42..=0xff43 => self.scroll.write(address, data),
                0xff44..=0xff45 => self.stat.write(address, data),
                0xff47..=0xff49 => self.palette.write(address, data),
                0xff4a..=0xff4b => self.window.write(address, data),
                0xff4f => self.video_ram.write(address, data),
                #[cfg(feature = "cgb")]
                0xff68..=0xff6b => self.color_palette.write(address, data),
            }
        }
    }

    fn write_exact(&mut self, address: u16, buf: &[u8]) -> Result<(), WriteError> {
        match address {
            0xfe00 if buf.len() == 0xa0 => self.oam.write_exact(address, buf),
            _ => self.write_exact_fallback(address, buf),
        }
    }
}
