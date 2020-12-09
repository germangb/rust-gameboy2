use crate::cpu::registers::Registers;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod registers;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CPU {
    registers: Registers,
    ime: bool,
}

#[cfg(test)]
mod test {}
