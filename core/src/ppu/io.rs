use crate::device::{Device, Result};
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
