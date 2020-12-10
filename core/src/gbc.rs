use crate::{joypad::Button, Emulator};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Game Boy Color emulator.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameBoyColor<C> {
    booted: bool,
    emulator: Emulator<C>,
}

impl<C> GameBoyColor<C> {
    pub fn new(cartridge: C) -> Self {
        todo!()
    }

    pub fn press(&mut self, button: &Button) {
        todo!()
    }

    pub fn release(&mut self, button: &Button) {
        todo!()
    }
}
