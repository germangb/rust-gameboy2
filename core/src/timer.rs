use crate::{
    device::{Device, Result},
    irq,
    utils::ClockDecimate,
    Update, CLOCK,
};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const DIV: u64 = 16384;
const TIMA: &[u64] = &[1024, 16, 64, 256];

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Timer {
    div: u8,
    div_clock: ClockDecimate,
    tima: u8,
    tima_clock: ClockDecimate,
    tma: u8,
    tac: u8,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            div: 0,
            div_clock: ClockDecimate::new(CLOCK, DIV),
            tima: 0,
            tima_clock: ClockDecimate::new(CLOCK, CLOCK / 1024),
            tma: 0,
            tac: 0,
        }
    }
}

impl Timer {
    fn update_tima_clock(&mut self) {
        let target = CLOCK / TIMA[self.tac as usize & 0b11];
        self.tima_clock = ClockDecimate::new(CLOCK, target);
    }

    fn is_tima_enabled(&self) -> bool {
        self.tac & 0b100 != 0
    }
}

impl Update for Timer {
    fn update(&mut self, ticks: u64, flags: &mut irq::Flags) {
        // update the DIV clock
        let div = self.div_clock.update(ticks);
        self.div = self.div.wrapping_add(div as u8);

        if self.is_tima_enabled() {
            // update the tima clock
            let mut tima = self.tima as u16;
            tima += self.tima_clock.update(ticks) as u16;

            if tima > 0xff {
                self.tima = self.tma;

                flags.set(irq::Flags::TIMER, true);
            } else {
                self.tima = tima as _;
            }
        }
    }
}

impl Device for Timer {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xff04 => Ok(self.div),
                0xff05 => Ok(self.tima),
                0xff06 => Ok(self.tma),
                0xff07 => Ok(self.tac),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0xff04 => self.div = 0,
                0xff05 => self.tima = data,
                0xff06 => {
                    info!("TMA = {:#08b} {:#02x}", data, data);

                    self.tma = data
                }
                0xff07 => {
                    info!("TAC = {:#08b} {:#02x}", data, data);

                    self.tac = data;
                    self.update_tima_clock();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Timer;
    use crate::{cartridge::NoCartridge, device::Device, irq, Emulator, Update};

    #[test]
    fn timer_interrupt() {
        let mut timer = Timer::default();
        let mut flags = irq::Flags::default();

        let mut states = Vec::new();

        // enable CLOCK / 1024 timer
        timer.write(0xff07, 0b0000_0100).unwrap();
        timer.write(0xff05, 0xfe).unwrap();

        timer.update(4, &mut flags);
        states.push(flags.contains(irq::Flags::TIMER)); // false

        timer.update(4, &mut flags);
        states.push(flags.contains(irq::Flags::TIMER)); // false (tima = 0xff)

        timer.update(4, &mut flags);
        states.push(flags.contains(irq::Flags::TIMER)); // false

        timer.update(4, &mut flags);
        states.push(flags.contains(irq::Flags::TIMER)); // true

        assert_eq!(vec![false, false, false, true], states);
    }

    #[test]
    fn div() {
        let mut emu = Emulator::new(NoCartridge);
        emu.timer.div = 0x95;

        let mut states = vec![emu.timer.read(0xff04).unwrap()];

        emu.write(0xff04, 0xab).unwrap();

        states.push(emu.timer.read(0xff04).unwrap());

        assert_eq!(vec![0x95, 0x00], states);
    }

    #[test]
    fn tma() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff06, 0xab).unwrap();

        assert_eq!(0xab, emu.timer.read(0xff06).unwrap())
    }

    #[test]
    fn tac() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff07, 0xaf).unwrap();

        assert_eq!(0xaf, emu.timer.read(0xff07).unwrap())
    }
}
