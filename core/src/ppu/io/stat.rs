use crate::{device::Device, error::Error, irq};
use log::{info, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const SEARCH_DOTS: u64 = 80; // 80 dots (19 us)
const PIXELS_DOTS: u64 = 230; // 168 to 291 cycles (40 to 60 us) depending on sprite count
const HBLANK_DOTS: u64 = 376 - PIXELS_DOTS; // 85 to 208 dots (20 to 49 us) depending on previous mode 3 duration
const VBLANK_DOTS: u64 = 4560; // 4560 dots (1087 us, 10 scanlines)

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq)]
enum Mode {
    HBLANK = 0,
    VBLANK = 1,
    SEARCH = 2,
    PIXELS = 3,
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct STAT {
    dots: u64,
    stat: u8,
    ly: u8,
    lyc: u8,
}

impl STAT {
    pub fn ly(&self) -> u8 {
        self.ly
    }

    pub fn lyc(&self) -> u8 {
        self.lyc
    }

    pub fn stat(&self) -> u8 {
        self.stat
    }

    fn mode(&self) -> Mode {
        unsafe { std::mem::transmute(self.stat & 0b0000_0011) }
    }

    fn lyc_int(&self) -> bool {
        (self.stat & 0b0100_0000) != 0
    }

    fn oam_int(&self) -> bool {
        (self.stat & 0b0010_0000) != 0
    }

    fn vblank_int(&self) -> bool {
        (self.stat & 0b0001_0000) != 0
    }

    fn hblank_int(&self) -> bool {
        (self.stat & 0b0000_1000) != 0
    }

    fn set_mode(&mut self, mode: Mode) {
        self.stat &= 0b1111_1100;
        self.stat |= unsafe { std::mem::transmute::<_, u8>(mode) }
    }

    fn search(&mut self) {
        if self.dots > SEARCH_DOTS {
            self.dots %= SEARCH_DOTS;
        } else {
            return;
        }

        self.set_mode(Mode::PIXELS);
    }

    fn pixels(&mut self, flags: &mut irq::Flags) {
        if self.dots > PIXELS_DOTS {
            self.dots %= PIXELS_DOTS;
        } else {
            return;
        }

        // increment line
        self.ly += 1;

        if self.hblank_int() {
            flags.set(irq::Flags::LCD_STAT, true);
        }
        self.set_mode(Mode::HBLANK);
    }

    fn hblank(&mut self, flags: &mut irq::Flags) {
        if self.dots > HBLANK_DOTS {
            self.dots %= HBLANK_DOTS;
        } else {
            return;
        }

        // increment line
        //self.ly += 1;

        if self.ly == 144 {
            if self.vblank_int() {
                flags.set(irq::Flags::LCD_STAT, true);
            }
            flags.set(irq::Flags::VBLANK, true);
            self.set_mode(Mode::VBLANK);
        } else {
            if self.oam_int() {
                flags.set(irq::Flags::LCD_STAT, true);
            }
            self.set_mode(Mode::SEARCH);
        }
    }

    fn vblank(&mut self, flags: &mut irq::Flags) {
        if self.dots > VBLANK_DOTS / 10 {
            self.dots %= VBLANK_DOTS / 10;
        } else {
            return;
        }

        // increment line
        self.ly += 1;

        if self.ly == 154 {
            if self.oam_int() {
                flags.set(irq::Flags::LCD_STAT, true);
            }
            self.ly = 0;
            self.set_mode(Mode::SEARCH);
        }
    }

    fn update_lyc_flag(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0b0000_0100;
        } else {
            self.stat &= 0b1111_1011;
        }
    }

    pub fn update(&mut self, ticks: u64, flags: &mut irq::Flags) -> bool {
        self.dots += ticks;

        // ly previous to update
        let ly = self.ly;
        let mode = self.mode();

        match self.mode() {
            Mode::SEARCH => self.search(),
            Mode::PIXELS => self.pixels(flags),
            Mode::HBLANK => self.hblank(flags),
            Mode::VBLANK => self.vblank(flags),
        }

        self.update_lyc_flag();

        if self.lyc_int() && self.ly == self.lyc && self.ly != ly {
            flags.set(irq::Flags::LCD_STAT, true);
        }

        // transition PIXELS -> HBLANK triggers draw of new scanline
        matches!((mode, self.mode()), (Mode::PIXELS, Mode::HBLANK))
    }
}

impl Device for STAT {
    const DEBUG_NAME: &'static str = "LCD Status (STAT)";

    fn read(&self, address: u16) -> Result<u8, Error> {
        match address {
            0xff41 => Ok(self.stat),
            0xff44 => Ok(self.ly),
            0xff45 => Ok(self.lyc),
            _ => Err(Error::InvalidAddr(address)),
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        match address {
            0xff41 => {
                self.stat &= 0b0000_0111;
                self.stat |= data & 0b0111_1000;
            }
            0xff44 => {
                warn!("WRITE to LY is undefined.");
            }
            0xff45 => {
                info!("LYC: {:02x}", data);

                self.lyc = data;
                self.update_lyc_flag();
            }
            _ => return Err(Error::InvalidAddr(address)),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn stat() {
        let mut emu = Emulator::new(NoCartridge);

        emu.ppu.stat.stat = 1;
        emu.write(0xff41, 0xff).unwrap();

        assert_eq!(0b0111_1_001, emu.ppu.stat.read(0xff41).unwrap());
    }

    #[test]
    fn lyc() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff45, 0x42).unwrap();

        assert_eq!(0x42, emu.ppu.stat.read(0xff45).unwrap());
    }

    #[test]
    fn ly() {
        let mut emu = Emulator::new(NoCartridge);

        emu.ppu.stat.ly = 7;

        assert_eq!(7, emu.read(0xff44).unwrap());
    }

    #[test]
    fn lcd_lyc_interrupt() {
        todo!()
    }

    #[test]
    fn lcd_oam_interrupt() {
        todo!()
    }

    #[test]
    fn lcd_vblank_interrupt() {
        todo!()
    }

    #[test]
    fn vblank_interrupt() {
        todo!()
    }
}
