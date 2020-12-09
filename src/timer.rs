use crate::{
    dev::{invalid_read, invalid_write, Device},
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

impl Timer {
    fn new() -> Self {
        Self {
            div: 0,
            div_clock: ClockDecimate::new(CLOCK, DIV),
            tima: 0,
            tima_clock: ClockDecimate::new(CLOCK, CLOCK / 1024),
            tma: 0,
            tac: 0,
        }
    }

    fn update_tima_clock(&mut self) {
        let target = CLOCK / TIMA[self.tac as usize & 0b11];
        self.tima_clock = ClockDecimate::new(CLOCK, target);
    }

    fn is_tima_enabled(&self) -> bool {
        self.tac & 0b100 != 0
    }
}

impl Update for Timer {
    fn update(&mut self, step: &EmulationStep) {
        // update the DIV clock
        let div = self.div_clock.update(step.clock_ticks);
        self.div = self.div.wrapping_add(div as u8);

        if self.is_tima_enabled() {
            // update the tima clock
            let mut tima = self.tima as u16;
            tima += self.tima_clock.update(step.clock_ticks) as u16;

            if tima > 0xff {
                self.tima = self.tma;
            } else {
                self.tima = tima as _;
            }
        }
    }
}

impl Device for Timer {
    fn debug_name() -> Option<&'static str> {
        Some("Timer")
    }

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
mod test {}
