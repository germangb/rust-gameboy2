use crate::device::{invalid_read, invalid_write, Device};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LCDC {
    lcdc: u8,
}

impl LCDC {
    pub fn lcd_on(&self) -> bool {
        (self.lcdc & 0b1000_0000) != 0
    }

    pub fn window_enable(&self) -> bool {
        (self.lcdc & 0b0010_0000) != 0
    }

    pub fn obj_enable(&self) -> bool {
        (self.lcdc & 0b0000_0010) != 0
    }

    pub fn obj_size(&self) -> bool {
        (self.lcdc & 0b0000_0100) != 0
    }

    // Returns the address where the window map.
    // According to LCDC.6
    pub fn window_map_select(&self) -> u16 {
        if (self.lcdc & 0b0100_0000) != 0 {
            0x9c00
        } else {
            0x9800
        }
    }

    // Returns the address where the BG & Window data.
    // According to LCDC.4
    pub fn bg_window_data_select(&self) -> u16 {
        if (self.lcdc & 0b0001_0000) != 0 {
            0x8000
        } else {
            0x8800
        }
    }

    // Returns the address where the window map.
    // According to LCDC.3
    pub fn bg_map_select(&self) -> u16 {
        if (self.lcdc & 0b0000_1000) != 0 {
            0x9c00
        } else {
            0x9800
        }
    }
}

impl Device for LCDC {
    const DEBUG_NAME: &'static str = "LCD Control (LCDC)";

    fn read(&self, address: u16) -> u8 {
        if address != 0xff40 {
            invalid_read(address);
        }

        self.lcdc
    }

    fn write(&mut self, address: u16, data: u8) {
        if address != 0xff40 {
            invalid_write(address);
        }

        info!("LCDC = {:#08b}", data);

        self.lcdc = data
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, ppu::io::lcdc::LCDC, Emulator};

    #[test]
    fn lcd_off() {
        let lcdc = LCDC { lcdc: 0 };

        assert!(!lcdc.lcd_on());
    }

    #[test]
    fn lcd_on() {
        let lcdc = LCDC { lcdc: 0b1000_0000 };

        assert!(lcdc.lcd_on());
    }

    #[test]
    fn lcdc() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff40, 0xab);

        assert_eq!(0xab, emu.ppu.lcdc.read(0xff40));
    }

    #[test]
    fn window_display() {
        todo!()
    }

    #[test]
    fn obj_display() {
        todo!()
    }

    #[test]
    fn bg_window_display_priority_display() {
        todo!()
    }

    #[test]
    fn bg_window_data_select() {
        todo!()
    }

    #[test]
    fn bg_map_select() {
        todo!()
    }

    #[test]
    fn window_map_select() {
        todo!()
    }
}
