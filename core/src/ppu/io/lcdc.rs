use crate::device::{invalid_read, invalid_write, Device};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LCDC {
    lcdc: u8,
}

impl LCDC {
    pub fn lcd_on(&self) -> bool {
        (self.lcdc & 0b1000_0000) != 0
    }

    // Returns the address where the window map.
    // According to LCDC.6
    pub fn window_map_select(&self) -> u16 {
        todo!()
    }

    // Returns the address where the BG & Window data.
    // According to LCDC.4
    pub fn bg_window_data_select(&self) -> u16 {
        todo!()
    }

    // Returns the address where the window map.
    // According to LCDC.3
    pub fn bg_map_select(&self) -> u16 {
        todo!()
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

        self.lcdc = data
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};
    use crate::ppu::io::lcdc::LCDC;

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
}
