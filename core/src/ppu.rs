use crate::{
    device::{Device, Result},
    irq,
    ppu::{
        io::{ColorPalette, Palette, Scroll, Window, LCDC, STAT},
        lcd::{Color, ColorId, Display, DisplayBuffer, LineBuffer},
        oam::{Entry, Flags, OAM},
    },
    ram::{Attributes, VideoRAM},
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
    back: Box<DisplayBuffer>,
    #[cfg_attr(feature = "serde", serde(skip))]
    front: Box<DisplayBuffer>,
    #[cfg_attr(feature = "serde", serde(skip))]
    line: Box<LineBuffer>,
    oam: OAM,
    video_ram: VideoRAM,
    lcdc: LCDC,
    stat: STAT,
    scroll: Scroll,
    window: Window,
    palette: Palette,
    color_palette: ColorPalette,
    #[cfg(feature = "debug")]
    debug_overlays: bool,
}

impl PPU {
    pub fn display(&self) -> &Display {
        &self.front
    }

    fn clear_display(&mut self) {
        #[cfg(not(feature = "cgb"))]
        let color = lcd::PALETTE[0];
        #[cfg(feature = "cgb")]
        let color = 0xffffff;
        self.back.iter_mut().for_each(|p| *p = color);
        self.swap_buffers();
    }

    #[cfg(not(feature = "debug"))]
    pub fn set_debug_overlays(&mut self, _: bool) {
        warn!("Emulator is not built with debug overlays.");
    }

    #[cfg(feature = "debug")]
    pub fn set_debug_overlays(&mut self, b: bool) {
        self.debug_overlays = b;
    }

    fn swap_buffers(&mut self) {
        self.back.iter_mut().for_each(|p| *p = lcd::transform(*p));

        #[cfg(feature = "debug")]
        if self.debug_overlays {
            self.draw_debug_overlays();
        }

        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn finish(&mut self) {}

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
            .draw(self.back.as_mut())
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
            .draw(self.back.as_mut())
            .unwrap();
        }
    }

    // render the given line to the display buffer
    fn draw_scanline(&mut self, ly: u8) {
        // When Bit 0 is cleared, both background and window become blank (white), and
        // the Window Display Bit is ignored in that case. Only Sprites may still be
        // displayed (if enabled in Bit 1).
        #[cfg(not(feature = "cgb"))]
        if self.lcdc.bg_window_priority() {
            self.draw_bg_win(ly);
        }

        #[cfg(feature = "cgb")]
        self.draw_bg_win(ly);

        if self.lcdc.obj_enable() {
            self.draw_sprites(ly);
        }

        if ly == 143 {
            self.swap_buffers();
        }
    }

    fn draw_bg_win(&mut self, ly: u8) {
        let Scroll { scy, scx } = self.scroll;
        let Window { wy, wx } = self.window;

        for dot in 0..lcd::WIDTH {
            let row = scy.wrapping_add(ly) as u16;
            let col = scx.wrapping_add(dot as u8) as u16;

            let (color, color_id) = if self.lcdc.window_enable() {
                // WX - Window X Position minus 7
                let dot = (dot + 7) as u8;

                if ly >= wy && dot >= wx {
                    let row = (ly - wy) as u16;
                    let col = (dot - wx) as u16;
                    let (mut color, color_id) =
                        self.decode_bg_win(row, col, self.lcdc.window_map_select());

                    #[cfg(feature = "debug")]
                    if self.debug_overlays {
                        color = debug::mix(color, 0x00ffff, 0.5);
                    }

                    (color, color_id)
                } else {
                    self.decode_bg_win(row, col, self.lcdc.bg_map_select())
                }
            } else {
                self.decode_bg_win(row, col, self.lcdc.bg_map_select())
            };

            self.draw_pixel(dot, color, color_id);
        }
    }

    fn draw_sprites(&mut self, ly: u8) {
        let ly = ly as i16;
        self.oam.sort_by_x();

        // used to stop processing sprites after a total of 10 has been reached
        // this is not accurate to how the GB works but it's close enough...
        let mut sprite_bitset: u64 = 0;

        for i in 0..40 {
            if sprite_bitset.count_ones() == 10 {
                break;
            }
            let Entry {
                y,
                x,
                tile_index,
                flags,
            } = self.oam.table()[i];

            let y = (y as i16) - 16;
            let x = (x as i16) - 8;
            let size = if self.lcdc.obj_size() { 16 } else { 8 };

            if ly >= y && ly < y + size {
                for dot in x.max(0)..(x + 8).min(lcd::WIDTH as _) {
                    // current BG | WINDOW color
                    let (color, color_id) = self.pixel(dot as _);

                    #[cfg(feature = "cgb")]
                    if color_id == 4 {
                        continue;
                    }
                    if flags.contains(Flags::OBJ_TO_BG_PRIORITY) && color_id != 0 {
                        continue;
                    }

                    sprite_bitset |= (1 << (i as u64));

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
                    let color_palette = self.color_palette.obp(flags.palette());
                    #[cfg(feature = "cgb")]
                    let bank = flags.bank();

                    let (mut color, _) = self
                        .decode_tile(
                            row as u16,
                            col as u16,
                            tile_index,
                            bank,
                            0x8000,
                            color_palette,
                            true,
                        )
                        .unwrap_or((color, 0));

                    #[cfg(feature = "debug")]
                    if self.debug_overlays {
                        color = if self.lcdc.obj_size() {
                            debug::mix(color, debug::DEBUG_OBJ_DOUBLE, 0.5)
                        } else {
                            debug::mix(color, debug::DEBUG_OBJ, 0.5)
                        };
                    };

                    self.draw_pixel(dot as _, color, 0);
                }
            }
        }
    }

    fn pixel(&self, dot: usize) -> (Color, ColorId) {
        let ly = self.stat.ly();
        let display_offset = lcd::WIDTH * (ly as usize);
        (self.back[display_offset + dot], self.line[dot])
    }

    fn draw_pixel(&mut self, dot: usize, color: Color, color_id: ColorId) {
        let ly = self.stat.ly();
        let display_offset = lcd::WIDTH * (ly as usize);
        self.back[display_offset + dot] = color;
        self.line[dot] = color_id;
    }

    // decode bg and window pixel
    fn decode_bg_win(&self, row: u16, col: u16, map: u16) -> (Color, ColorId) {
        // tile pixel
        let pixel_row = row % 8;
        let pixel_col = 7 - (col % 8);
        // transformed tile pixel
        let mut tile_pixel_row = pixel_row;
        let mut tile_pixel_col = pixel_col;
        // tile index
        let tile_index_address = map + 32 * (row / 8) + (col / 8);
        let tile_index = self.video_ram.data(0, tile_index_address);

        #[cfg(feature = "cgb")]
        let tile_attributes = self.video_ram.attributes(tile_index_address);
        #[cfg(feature = "cgb")]
        if tile_attributes.contains(Attributes::HORIZONTAL_FLIP) {
            tile_pixel_col = 7 - tile_pixel_col;
        }
        #[cfg(feature = "cgb")]
        if tile_attributes.contains(Attributes::VERTICAL_FLIP) {
            tile_pixel_row = 7 - tile_pixel_row;
        }

        #[cfg(not(feature = "cgb"))]
        let color_palette = self.palette.bgp();
        #[cfg(not(feature = "cgb"))]
        let bank = 0;
        #[cfg(feature = "cgb")]
        let color_palette = self.color_palette.bgp(tile_attributes.palette());
        #[cfg(feature = "cgb")]
        let bank = tile_attributes.bank();

        let (mut color, mut color_id) = self
            .decode_tile(
                tile_pixel_row,
                tile_pixel_col,
                tile_index,
                bank,
                self.lcdc.bg_window_data_select(),
                color_palette,
                false,
            )
            .unwrap();

        // When Bit 7 is set, the corresponding BG tile will have priority above all
        // OBJs (regardless of the priority bits in OAM memory). There's also a Master
        // Priority flag in LCDC register Bit 0 which overrides all other priority bits
        // when cleared.
        #[cfg(feature = "cgb")]
        if tile_attributes.contains(Attributes::BG_OAM_PRIPRITY) && color_id != 0 {
            color_id = 4;
            #[cfg(feature = "debug")]
            if self.debug_overlays {
                color = debug::mix(color, 0xffff00, 0.25);
            }
        }

        #[cfg(feature = "debug")]
        if self.debug_overlays && (pixel_row == 0 || pixel_col == 0) {
            color = debug::mix(color, 0x000000, 0.25);
        }

        (color, color_id)
    }

    // decode tile pixel color (None if reansparent)
    fn decode_tile(
        &self,
        pixel_row: u16,
        pixel_col: u16,
        tile_index: u8,
        bank: usize,
        data_select: u16,
        color_palette: [Color; 4],
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

impl Update for PPU {
    fn update(&mut self, ticks: u64, flags: &mut irq::Flags) {
        let line = self.stat.ly();

        if self.stat.update(ticks, flags) {
            self.draw_scanline(line);
        }
    }
}

impl Device for PPU {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
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
                0xff68..=0xff6b => self.color_palette.read(address),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
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
                0xff4f => self.video_ram.write(address, data),
                0xff68..=0xff6b => self.color_palette.write(address, data),
            }
        }
    }
}
