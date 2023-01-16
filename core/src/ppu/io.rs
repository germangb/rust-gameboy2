use crate::{
    device::Device,
    error::{ReadError, WriteError},
    ppu::{Color, PALETTE},
};
pub use lcdc::LCDC;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
pub use stat::STAT;

pub mod lcdc;
pub mod stat;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scroll {
    pub scy: u8,
    pub scx: u8,
}

impl Device for Scroll {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff42 => Ok(self.scy),
                0xff43 => Ok(self.scx),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff42 => {
                    log::trace!("Register SCY: {:02x}", data);

                    self.scy = data
                }
                0xff43 => {
                    log::trace!("Register SCX: {:02x}", data);

                    self.scx = data
                }
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Window {
    pub wy: u8,
    pub wx: u8,
}

impl Device for Window {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff4a => Ok(self.wy),
                0xff4b => Ok(self.wx),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff4a => {
                    log::trace!("Register WY: {:02x}", data);

                    self.wy = data;
                }
                0xff4b => {
                    log::trace!("Register WX: {:02x}", data);

                    self.wx = data;
                }
            }
        }

        Ok(())
    }
}

// So some CGB games still make use of this
// Maybe it's only the ones with dual GB/CGB compatibility (maybe investigate?)
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Palette {
    bgp: u8,
    obp0: u8,
    obp1: u8,
    // cached colors
    bgp_cache: [Color; 4],
    obp0_cache: [Color; 4],
    obp1_cache: [Color; 4],
}

impl Default for Palette {
    fn default() -> Self {
        let bgp = Default::default();
        let obp0 = Default::default();
        let obp1 = Default::default();
        Self {
            bgp,
            obp0,
            obp1,
            bgp_cache: Self::palette(bgp),
            obp0_cache: Self::palette(obp0),
            obp1_cache: Self::palette(obp1),
        }
    }
}

impl Palette {
    #[allow(unused)]
    pub fn bgp(&self) -> &[Color; 4] {
        &self.bgp_cache
    }

    #[allow(unused)]
    pub fn obp0(&self) -> &[Color; 4] {
        &self.obp0_cache
    }

    #[allow(unused)]
    pub fn obp1(&self) -> &[Color; 4] {
        &self.obp1_cache
    }

    #[allow(unused)]
    pub fn palette(pal: u8) -> [Color; 4] {
        [
            PALETTE[(pal & 0b11) as usize],
            PALETTE[((pal >> 2) & 0b11) as usize],
            PALETTE[((pal >> 4) & 0b11) as usize],
            PALETTE[((pal >> 6) & 0b11) as usize],
        ]
    }
}

impl Device for Palette {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff47 => Ok(self.bgp),
                0xff48 => Ok(self.obp0),
                0xff49 => Ok(self.obp1),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff47 => {
                    self.bgp = data;
                    self.bgp_cache = Self::palette(data);
                }
                0xff48 => {
                    self.obp0 = data;
                    self.obp0_cache = Self::palette(data);
                }
                0xff49 => {
                    self.obp1 = data;
                    self.obp1_cache = Self::palette(data);
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "cgb")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColorPalette {
    bgpi: u8,
    obpi: u8,
    bgp: Box<[u8]>,
    obp: Box<[u8]>,
    bgp_cache: Box<[[Color; 4]; 8]>,
    obp_cache: Box<[[Color; 4]; 8]>,
}

#[cfg(feature = "cgb")]
impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            bgpi: 0,
            obpi: 0,
            bgp: vec![0; 0x40].into_boxed_slice(),
            obp: vec![0; 0x40].into_boxed_slice(),
            bgp_cache: Default::default(),
            obp_cache: Default::default(),
        }
    }
}

#[cfg(feature = "cgb")]
impl ColorPalette {
    fn compute_palettes(&mut self) {
        for palette in 0..8 {
            self.bgp_cache[palette] = [
                Self::palette_color(&self.bgp, palette, 0),
                Self::palette_color(&self.bgp, palette, 1),
                Self::palette_color(&self.bgp, palette, 2),
                Self::palette_color(&self.bgp, palette, 3),
            ];
            self.obp_cache[palette] = [
                Self::palette_color(&self.obp, palette, 0),
                Self::palette_color(&self.obp, palette, 1),
                Self::palette_color(&self.obp, palette, 2),
                Self::palette_color(&self.obp, palette, 3),
            ];
        }
    }

    pub fn bgp(&self) -> &[[Color; 4]; 8] {
        &self.bgp_cache
    }

    pub fn obp(&self) -> &[[Color; 4]; 8] {
        &self.obp_cache
    }

    fn palette_color(pal_data: &[u8], palette: usize, color: usize) -> Color {
        let palette_offset = 8 * palette;
        let palette_data = &pal_data[palette_offset..palette_offset + 8];
        let color_offset = color * 2;
        let color = (palette_data[color_offset as usize] as u16)
            | (palette_data[color_offset as usize + 1] as u16) << 8;
        let r = (0xff * (color & 0x1f) / 0x1f) as u16;
        let g = (0xff * ((color >> 5) & 0x1f) / 0x1f) as u16;
        let b = (0xff * ((color >> 10) & 0x1f) / 0x1f) as u16;
        crate::ppu::lcd::color(r as u8, g as u8, b as u8)
    }

    fn write_color(pal_data: &mut [u8], mut idx: u8, data: u8) -> u8 {
        pal_data[(idx & 0x3f) as usize] = data;
        if idx & 0x80 != 0 {
            idx += 1;
            idx &= 0xbf;
        }
        idx
    }
}

#[cfg(feature = "cgb")]
impl Device for ColorPalette {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff68 => Ok(self.bgpi),
                0xff69 => Ok(self.bgp[(self.bgpi & 0x3f) as usize]),
                0xff6a => Ok(self.obpi),
                0xff6b => Ok(self.obp[(self.obpi & 0x3f) as usize]),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff68 => self.bgpi = data,
                0xff69 => self.bgpi = Self::write_color(&mut self.bgp[..], self.bgpi, data),
                0xff6a => self.obpi = data,
                0xff6b => self.obpi = Self::write_color(&mut self.obp[..], self.obpi, data),
            }
        }
        self.compute_palettes();
        Ok(())
    }
}

#[cfg(test)]
mod test {}
