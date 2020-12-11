use crate::{
    device::{invalid_read, invalid_write, Device},
    irq::Request,
    utils::ClockDecimate,
    EmulationStep, Update, CLOCK,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const DIV: u64 = 16384;
const TIMA: &[u64] = &[1024, 16, 64, 256];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
    fn update(&mut self, step: &EmulationStep, request: &mut Request) {
        // update the DIV clock
        let div = self.div_clock.update(step.clock_ticks);
        self.div = self.div.wrapping_add(div as u8);

        if self.is_tima_enabled() {
            // update the tima clock
            let mut tima = self.tima as u16;
            tima += self.tima_clock.update(step.clock_ticks) as u16;

            if tima > 0xff {
                self.tima = self.tma;

                request.timer = true;
            } else {
                self.tima = tima as _;
            }
        }
    }
}

impl Device for Timer {
    const DEBUG_NAME: &'static str = "Timer";

    fn read(&self, address: u16) -> u8 {
        match address {
            0xff04 => self.div,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff04 => self.div = 0,
            0xff05 => self.tima = data,
            0xff06 => self.tma = data,
            0xff07 => {
                self.tac = data;
                self.update_tima_clock();
            }
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Timer;
    use crate::{
        cartridge::NoCartridge, device::Device, irq::Request, EmulationStep, Emulator, Update,
    };

    #[test]
    fn timer_interrupt() {
        let mut timer = Timer::default();
        let mut request = Request::default();

        let mut states = Vec::new();

        // enable CLOCK / 1024 timer
        timer.write(0xff07, 0b0000_0100);
        timer.write(0xff05, 0xfe);

        timer.update(&EmulationStep { clock_ticks: 4 }, &mut request);
        states.push(request.timer); // false

        timer.update(&EmulationStep { clock_ticks: 4 }, &mut request);
        states.push(request.timer); // false (tima = 0xff)

        timer.update(&EmulationStep { clock_ticks: 4 }, &mut request);
        states.push(request.timer); // false

        timer.update(&EmulationStep { clock_ticks: 4 }, &mut request);
        states.push(request.timer); // true

        assert_eq!(vec![false, false, false, true], states);
    }

    #[test]
    fn div() {
        let mut emu = Emulator::new(NoCartridge);
        emu.timer.div = 0x95;

        let mut states = vec![emu.timer.read(0xff04)];

        emu.write(0xff04, 0xab);

        states.push(emu.timer.read(0xff04));

        assert_eq!(vec![0x95, 0x00], states);
    }

    #[test]
    fn tma() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff06, 0xab);

        assert_eq!(0xab, emu.timer.read(0xff06))
    }

    #[test]
    fn tac() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xff07, 0xaf);

        assert_eq!(0xaf, emu.timer.read(0xff07))
    }
}
