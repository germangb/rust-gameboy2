#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClockDecimate {
    base: u64,
    target: u64,
    clocks_per_tick: u64,
    base_ticks: u64,
    carry: u64,
}

impl ClockDecimate {
    pub fn new(base: u64, target: u64) -> Self {
        assert!(base >= target);

        let clocks_per_tick = base / target;
        Self {
            base,
            target,
            clocks_per_tick,
            base_ticks: 0,
            carry: 0,
        }
    }

    /// Process tick from the base clock.
    /// Returns the number of ticks elapsed from the target clock.
    pub fn update(&mut self, base_ticks: u64) -> u64 {
        self.base_ticks += base_ticks;
        let total_ticks = self.base_ticks + self.carry;
        if total_ticks >= self.clocks_per_tick {
            let ticks = total_ticks / self.clocks_per_tick;
            self.carry = total_ticks % self.clocks_per_tick;
            self.base_ticks = 0;
            ticks
        } else {
            0
        }
    }
}

#[cfg(test)]
mod test {}
