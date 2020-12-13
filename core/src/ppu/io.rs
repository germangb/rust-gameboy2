use crate::{
    device::{invalid_read, invalid_write, Device},
    ppu::lcd::Pixel,
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
    const DEBUG_NAME: &'static str = "Scroll";

    fn read(&self, address: u16) -> u8 {
        match address {
            0xff42 => self.scy,
            0xff43 => self.scx,
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff42 => {
                info!("SCY = {:#02x}", data);

                self.scy = data
            }
            0xff43 => {
                info!("SCX = {:#02x}", data);

                self.scx = data
            }
            _ => invalid_write(address),
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Window {
    pub wy: u8,
    pub wx: u8,
}

impl Device for Window {
    const DEBUG_NAME: &'static str = "Window";

    fn read(&self, address: u16) -> u8 {
        match address {
            0xff4a => self.wy,
            0xff4b => self.wx,
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff4a => {
                info!("WY = {:#02x}", data);

                self.wy = data
            }
            0xff4b => {
                info!("WX = {:#02x}", data);

                self.wx = data
            }
            _ => invalid_write(address),
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Palette {
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
}

fn log_pal(pal: u8) -> String {
    const S: [&str; 4] = ["░", "▒", "▓", "█"];

    format!(
        "{:02b}{}{:02b}{}{:02b}{}{:02b}{}",
        pal & 0b11,
        S[pal as usize & 0b11],
        pal >> 2 & 0b11,
        S[(pal as usize) >> 2 & 0b11],
        pal >> 4 & 0b11,
        S[(pal as usize) >> 4 & 0b11],
        pal >> 6 & 0b11,
        S[(pal as usize) >> 6 & 0b11],
    )
}

impl Device for Palette {
    const DEBUG_NAME: &'static str = "Color Palette";

    fn read(&self, address: u16) -> u8 {
        match address {
            0xff47 => self.bgp,
            0xff48 => self.obp0,
            0xff49 => self.obp1,
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff47 => {
                info!("BGP = {:#08b} ({})", data, log_pal(data));

                self.bgp = data
            }
            0xff48 => {
                info!("OBP0 = {:#08b} ({})", data, log_pal(data));

                self.obp0 = data
            }
            0xff49 => {
                info!("OBP1 = {:#08b} ({})", data, log_pal(data));

                self.obp1 = data
            }
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn scroll() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff42, 0x12);
        emu.write(0xff43, 0xab);

        assert_eq!([0x12, 0xab], [emu.ppu.scroll.scy, emu.ppu.scroll.scx]);
    }

    #[test]
    fn window() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff4a, 0x12);
        emu.write(0xff4b, 0xab);

        assert_eq!(
            [0x12, 0x12, 0xab, 0xab],
            [
                emu.ppu.window.wy,
                emu.ppu.window.read(0xff4a),
                emu.ppu.window.wx,
                emu.ppu.window.read(0xff4b)
            ]
        );
    }

    #[test]
    fn palette() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff47, 0x01);
        emu.write(0xff48, 0x9a);
        emu.write(0xff49, 0xef);

        assert_eq!(
            [0x01, 0x01, 0x9a, 0x9a, 0xef, 0xef,],
            [
                emu.ppu.palette.bgp,
                emu.ppu.palette.read(0xff47),
                emu.ppu.palette.obp0,
                emu.ppu.palette.read(0xff48),
                emu.ppu.palette.obp1,
                emu.ppu.palette.read(0xff49),
            ]
        );
    }
}
