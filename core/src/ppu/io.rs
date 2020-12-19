use crate::{
    device::{Device, Result},
    ppu::{lcd, lcd::Pixel},
};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod lcdc;
pub mod stat;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Scroll {
    pub scy: u8,
    pub scx: u8,
}

impl Device for Scroll {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xff42 => Ok(self.scy),
                0xff43 => Ok(self.scx),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0xff42 => {
                    info!("Register SCY: {:02x}", data);

                    self.scy = data
                }
                0xff43 => {
                    info!("Register SCX: {:02x}", data);

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
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xff4a => Ok(self.wy),
                0xff4b => Ok(self.wx),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0xff4a => {
                    info!("Register WY: {:02x}", data);

                    self.wy = data;
                }
                0xff4b => {
                    info!("Register WX: {:02x}", data);

                    self.wx = data;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Palette {
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
}

impl Palette {
    pub fn bgp(&self) -> [Pixel; 4] {
        Self::palette(self.bgp)
    }

    pub fn obp0(&self) -> [Pixel; 4] {
        Self::palette(self.obp0)
    }

    pub fn obp1(&self) -> [Pixel; 4] {
        Self::palette(self.obp1)
    }

    pub fn palette(pal: u8) -> [Pixel; 4] {
        [
            lcd::PALETTE[(pal & 0b11) as usize],
            lcd::PALETTE[((pal >> 2) & 0b11) as usize],
            lcd::PALETTE[((pal >> 4) & 0b11) as usize],
            lcd::PALETTE[((pal >> 6) & 0b11) as usize],
        ]
    }
}

fn log_pal(pal: u8) -> String {
    const S: [&str; 4] = ["░░", "▒▒", "▓▓", "██"];

    format!(
        "{}{}{}{}",
        S[pal as usize & 0b11],
        S[(pal as usize) >> 2 & 0b11],
        S[(pal as usize) >> 4 & 0b11],
        S[(pal as usize) >> 6 & 0b11],
    )
}

impl Device for Palette {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xff47 => Ok(self.bgp),
                0xff48 => Ok(self.obp0),
                0xff49 => Ok(self.obp1),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0xff47 => {
                    info!("Register BGP: {:08b} ({})", data, log_pal(data));

                    self.bgp = data
                }
                0xff48 => {
                    info!("Register OBP0: {:08b} ({})", data, log_pal(data));

                    self.obp0 = data
                }
                0xff49 => {
                    info!("Register OBP1: {:08b} ({})", data, log_pal(data));

                    self.obp1 = data
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColorPalette {
    bgpi: u8,
    obpi: u8,
    bgp: Box<[u8]>,
    obp: Box<[u8]>,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            bgpi: 0,
            obpi: 0,
            bgp: vec![0; 0x40].into_boxed_slice(),
            obp: vec![0; 0x40].into_boxed_slice(),
        }
    }
}

impl ColorPalette {
    pub fn bgp(&self, palette: usize) -> [Pixel; 4] {
        [
            Self::palette_color(&self.bgp, palette, 0),
            Self::palette_color(&self.bgp, palette, 1),
            Self::palette_color(&self.bgp, palette, 2),
            Self::palette_color(&self.bgp, palette, 3),
        ]
    }

    pub fn obp(&self, palette: usize) -> [Pixel; 4] {
        [
            Self::palette_color(&self.obp, palette, 0),
            Self::palette_color(&self.obp, palette, 1),
            Self::palette_color(&self.obp, palette, 2),
            Self::palette_color(&self.obp, palette, 3),
        ]
    }

    fn palette_color(pal_data: &[u8], palette: usize, color: usize) -> Pixel {
        let palette_offset = 8 * palette;
        let palette_data = &pal_data[palette_offset..palette_offset + 8];
        let color_offset = color * 2;
        let color = (palette_data[color_offset as usize] as u16)
            | (palette_data[color_offset as usize + 1] as u16) << 8;
        let r = (0xff * (color & 0x1f) / 0x1f) as Pixel;
        let g = (0xff * ((color >> 5) & 0x1f) / 0x1f) as Pixel;
        let b = (0xff * ((color >> 10) & 0x1f) / 0x1f) as Pixel;
        (r << 16) | (g << 8) | b
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

impl Device for ColorPalette {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xff68 => Ok(self.bgpi),
                0xff69 => Ok(self.bgp[(self.bgpi & 0x3f) as usize]),
                0xff6a => Ok(self.obpi),
                0xff6b => Ok(self.obp[(self.obpi & 0x3f) as usize]),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0xff68 => self.bgpi = data,
                0xff69 => self.bgpi = Self::write_color(&mut self.bgp[..], self.bgpi, data),
                0xff6a => self.obpi = data,
                0xff6b => self.obpi = Self::write_color(&mut self.obp[..], self.obpi, data),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn scroll() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff42, 0x12).unwrap();
        emu.write(0xff43, 0xab).unwrap();

        assert_eq!([0x12, 0xab], [emu.ppu.scroll.scy, emu.ppu.scroll.scx]);
    }

    #[test]
    fn window() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff4a, 0x12).unwrap();
        emu.write(0xff4b, 0xab).unwrap();

        assert_eq!(
            [0x12, 0x12, 0xab, 0xab],
            [
                emu.ppu.window.wy,
                emu.ppu.window.read(0xff4a).unwrap(),
                emu.ppu.window.wx,
                emu.ppu.window.read(0xff4b).unwrap()
            ]
        );
    }

    #[test]
    fn palette() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff47, 0x01).unwrap();
        emu.write(0xff48, 0x9a).unwrap();
        emu.write(0xff49, 0xef).unwrap();

        assert_eq!(
            [0x01, 0x01, 0x9a, 0x9a, 0xef, 0xef,],
            [
                emu.ppu.palette.bgp,
                emu.ppu.palette.read(0xff47).unwrap(),
                emu.ppu.palette.obp0,
                emu.ppu.palette.read(0xff48).unwrap(),
                emu.ppu.palette.obp1,
                emu.ppu.palette.read(0xff49).unwrap(),
            ]
        );
    }
}
