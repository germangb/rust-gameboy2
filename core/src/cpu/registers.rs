#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

// Internal macro to check the state of the flags register.
#[rustfmt::skip]
macro_rules! flag {
    (_, $reg:expr, $mask:expr) => {
        ($reg.f & ($mask << 4)) != 0
    };

    ($reg:expr, Z) => { flag!(_, $reg, 0b1000) };
    ($reg:expr, N) => { flag!(_, $reg, 0b0100) };
    ($reg:expr, H) => { flag!(_, $reg, 0b0010) };
    ($reg:expr, C) => { flag!(_, $reg, 0b0001) };

    (_, $reg:expr, $mask:expr, $val:expr) => {
        if $val {
            $reg.f |= $mask << 4;
        } else {
            $reg.f &= !($mask << 4);
        }
    };

    ($reg:expr, Z = $val:expr) => { flag!(_, $reg, 0b1000, $val) };
    ($reg:expr, N = $val:expr) => { flag!(_, $reg, 0b0100, $val) };
    ($reg:expr, H = $val:expr) => { flag!(_, $reg, 0b0010, $val) };
    ($reg:expr, C = $val:expr) => { flag!(_, $reg, 0b0001, $val) };
}

// macro to implement getters&setter methods of u16 registers
macro_rules! register {
    (fn { $fn_get:ident, $fn_set:ident } ($hi:ident, $lo:ident)) => {
        pub fn $fn_get(&self) -> u16 {
            let hi = self.$hi as u16;
            let lo = self.$lo as u16;
            (hi << 8) | lo
        }

        pub fn $fn_set(&mut self, data: u16) {
            let hi = (data >> 8) & 0xff;
            let lo = data & 0xff;
            self.$hi = hi as _;
            self.$lo = lo as _;
        }
    };
}

impl Registers {
    register!(fn {af, set_af} (a, f));
    register!(fn {bc, set_bc} (b, c));
    register!(fn {de, set_de} (d, e));
    register!(fn {hl, set_hl} (h, l));
}

#[cfg(test)]
mod test {}
