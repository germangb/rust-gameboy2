use crate::{
    device::{invalid_read, invalid_write, Device},
    irq::Request,
    CLOCK,
};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const SEARCH_DOTS: u64 = 80; // 80 dots (19 us)
const PIXELS_DOTS: u64 = 230; // 168 to 291 cycles (40 to 60 us) depending on sprite count
const HBLANK_DOTS: u64 = 147; // 85 to 208 dots (20 to 49 us) depending on previous mode 3 duration
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

    fn coincidence_flag(&self) -> bool {
        (self.stat & 0b0000_0100) != 0
    }

    fn set_mode(&mut self, mode: Mode) {
        self.stat &= 0b1111_1100;
        self.stat |= unsafe { std::mem::transmute::<_, u8>(mode) }
    }

    fn search(&mut self, request: &mut Request) {
        if self.dots > SEARCH_DOTS {
            self.dots %= SEARCH_DOTS;
        } else {
            return;
        }

        self.set_mode(Mode::PIXELS);
    }

    fn pixels(&mut self, request: &mut Request) {
        if self.dots > PIXELS_DOTS {
            self.dots %= PIXELS_DOTS;
        } else {
            return;
        }

        // increment line
        //self.ly += 1;

        if self.hblank_int() {
            request.lcd_stat = true;
        }
        self.set_mode(Mode::HBLANK);
    }

    fn hblank(&mut self, request: &mut Request) {
        if self.dots > HBLANK_DOTS {
            self.dots %= HBLANK_DOTS;
        } else {
            return;
        }

        // increment line
        self.ly += 1;

        if self.ly == 144 {
            if self.vblank_int() {
                request.lcd_stat = true;
            }
            request.vblank = true;
            self.set_mode(Mode::VBLANK);
        } else {
            if self.oam_int() {
                request.lcd_stat = true;
            }
            self.set_mode(Mode::SEARCH);
        }
    }

    fn vblank(&mut self, request: &mut Request) {
        if self.dots > VBLANK_DOTS / 10 {
            self.dots %= VBLANK_DOTS / 10;
        } else {
            return;
        }

        // increment line
        self.ly += 1;

        if self.ly == 154 {
            if self.oam_int() {
                request.lcd_stat = true;
            }
            self.ly = 0;
            self.set_mode(Mode::SEARCH);
        }
    }

    pub fn update(&mut self, ticks: u64, request: &mut Request) {
        self.dots += ticks;

        // ly previous to update
        let ly = self.ly;

        match self.mode() {
            Mode::SEARCH => self.search(request),
            Mode::PIXELS => self.pixels(request),
            Mode::HBLANK => self.hblank(request),
            Mode::VBLANK => self.vblank(request),
        }

        if self.ly == self.lyc {
            self.stat |= 0b0000_0100;

            if self.ly != ly && self.lyc_int() {
                request.lcd_stat = true;
            }
        } else {
            self.stat &= 0b1111_1011;
        }
    }
}

impl Device for STAT {
    const DEBUG_NAME: &'static str = "LCD Status (STAT)";

    fn read(&self, address: u16) -> u8 {
        match address {
            0xff41 => self.stat,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff41 => {
                self.stat &= 0b0000_0111;
                self.stat |= data & 0b1111_1000;
            }
            0xff44 => todo!(),
            0xff45 => {
                info!("LYC = {:#02x} {:#08b}", data, data);

                self.lyc = data
            }
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn stat() {
        let mut emu = Emulator::new(NoCartridge);

        emu.ppu.stat.stat = 1;
        emu.write(0xff41, 0xff);

        assert_eq!(0b1111_1_001, emu.ppu.stat.read(0xff41));
    }

    #[test]
    fn lyc() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff45, 0x42);

        assert_eq!(0x42, emu.ppu.stat.read(0xff45));
    }

    #[test]
    fn ly() {
        let mut emu = Emulator::new(NoCartridge);

        emu.ppu.stat.ly = 7;

        assert_eq!(7, emu.read(0xff44));
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
