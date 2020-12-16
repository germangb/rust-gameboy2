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

    pub fn bg_window_priority(&self) -> bool {
        (self.lcdc & 0b0000_0001) != 0
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

        info!("LCDC: {:08b}", data);

        self.lcdc = data
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, ppu::io::lcdc::LCDC, Emulator};

    #[test]
    fn lcdc() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000);
        states.push(emu.ppu.lcdc.lcd_on());

        emu.write(0xff40, 0b10000000);
        states.push(emu.ppu.lcdc.lcd_on());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    fn window_display() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000);
        states.push(emu.ppu.lcdc.window_enable());

        emu.write(0xff40, 0b00100000);
        states.push(emu.ppu.lcdc.window_enable());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    fn obj_display() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000);
        states.push(emu.ppu.lcdc.obj_enable());

        emu.write(0xff40, 0b00000010);
        states.push(emu.ppu.lcdc.obj_enable());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    fn bg_window_display_priority_display() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000);
        states.push(emu.ppu.lcdc.bg_window_priority());

        emu.write(0xff40, 0b00000001);
        states.push(emu.ppu.lcdc.bg_window_priority());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    fn bg_window_data_select() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000);
        states.push(emu.ppu.lcdc.bg_window_data_select());

        emu.write(0xff40, 0b00010000);
        states.push(emu.ppu.lcdc.bg_window_data_select());

        assert_eq!(vec![0x8800, 0x8000], states);
    }

    #[test]
    fn bg_map_select() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000);
        states.push(emu.ppu.lcdc.bg_map_select());

        emu.write(0xff40, 0b00001000);
        states.push(emu.ppu.lcdc.bg_map_select());

        assert_eq!(vec![0x9800, 0x9c00], states);
    }

    #[test]
    fn window_map_select() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000);
        states.push(emu.ppu.lcdc.window_map_select());

        emu.write(0xff40, 0b01000000);
        states.push(emu.ppu.lcdc.window_map_select());

        assert_eq!(vec![0x9800, 0x9c00], states);
    }
}
