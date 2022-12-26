use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    struct Flags: u8 {
        const LCD_ENABLE                 = 0b10000000;
        const WINDOW_MAP                 = 0b01000000;
        const WINDOW_ENABLE              = 0b00100000;
        const BG_WINDOW_DATA             = 0b00010000;
        const BG_MAP                     = 0b00001000;
        const OBJ_SIZE                   = 0b00000100;
        const OBJ_ENABLE                 = 0b00000010;
        const BG_WINDOW_DISPLAY_PRIORITY = 0b00000001;
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LCDC {
    flags: Flags,
}

impl LCDC {
    #[cfg(todo)]
    pub fn bits(&self) -> u8 {
        self.flags.bits
    }

    pub fn lcd_on(&self) -> bool {
        self.flags.contains(Flags::LCD_ENABLE)
    }

    pub fn window_enable(&self) -> bool {
        self.flags.contains(Flags::WINDOW_ENABLE)
    }

    pub fn obj_enable(&self) -> bool {
        self.flags.contains(Flags::OBJ_ENABLE)
    }

    pub fn obj_size(&self) -> bool {
        self.flags.contains(Flags::OBJ_SIZE)
    }

    #[cfg(not(feature = "cgb"))]
    pub fn bg_window_priority(&self) -> bool {
        self.flags.contains(Flags::BG_WINDOW_DISPLAY_PRIORITY)
    }

    // Returns the address where the window map.
    // According to LCDC.6
    pub fn window_map_select(&self) -> u16 {
        if self.flags.contains(Flags::WINDOW_MAP) {
            0x9c00
        } else {
            0x9800
        }
    }

    // Returns the address where the BG & Window data.
    // According to LCDC.4
    pub fn bg_window_data_select(&self) -> u16 {
        if self.flags.contains(Flags::BG_WINDOW_DATA) {
            0x8000
        } else {
            0x8800
        }
    }

    // Returns the address where the window map.
    // According to LCDC.3
    pub fn bg_map_select(&self) -> u16 {
        if self.flags.contains(Flags::BG_MAP) {
            0x9c00
        } else {
            0x9800
        }
    }
}

impl Device for LCDC {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff40 => Ok(self.flags.bits),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff40 => {
                    log::trace!("LCDC: {:08b}", data);
                    self.flags = Flags::from_bits(data).unwrap();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, LR35902};

    #[test]
    fn lcdc() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000).unwrap();
        states.push(emu.ppu.lcdc.lcd_on());

        emu.write(0xff40, 0b10000000).unwrap();
        states.push(emu.ppu.lcdc.lcd_on());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    fn window_display() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000).unwrap();
        states.push(emu.ppu.lcdc.window_enable());

        emu.write(0xff40, 0b00100000).unwrap();
        states.push(emu.ppu.lcdc.window_enable());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    fn obj_display() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000).unwrap();
        states.push(emu.ppu.lcdc.obj_enable());

        emu.write(0xff40, 0b00000010).unwrap();
        states.push(emu.ppu.lcdc.obj_enable());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    #[cfg(not(feature = "cgb"))]
    fn bg_window_display_priority_display() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000).unwrap();
        states.push(emu.ppu.lcdc.bg_window_priority());

        emu.write(0xff40, 0b00000001).unwrap();
        states.push(emu.ppu.lcdc.bg_window_priority());

        assert_eq!(vec![false, true], states);
    }

    #[test]
    fn bg_window_data_select() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000).unwrap();
        states.push(emu.ppu.lcdc.bg_window_data_select());

        emu.write(0xff40, 0b00010000).unwrap();
        states.push(emu.ppu.lcdc.bg_window_data_select());

        assert_eq!(vec![0x8800, 0x8000], states);
    }

    #[test]
    fn bg_map_select() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000).unwrap();
        states.push(emu.ppu.lcdc.bg_map_select());

        emu.write(0xff40, 0b00001000).unwrap();
        states.push(emu.ppu.lcdc.bg_map_select());

        assert_eq!(vec![0x9800, 0x9c00], states);
    }

    #[test]
    fn window_map_select() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = Vec::new();

        emu.write(0xff40, 0b00000000).unwrap();
        states.push(emu.ppu.lcdc.window_map_select());

        emu.write(0xff40, 0b01000000).unwrap();
        states.push(emu.ppu.lcdc.window_map_select());

        assert_eq!(vec![0x9800, 0x9c00], states);
    }
}
