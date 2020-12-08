#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
}

// Internal macro to check the state of the flags register.
macro_rules! flag {
    ($reg:expr, Z) => {
        ($reg.f & 0b1000_0000) != 0
    };
    ($reg:expr, N) => {
        ($reg.f & 0b0100_0000) != 0
    };
    ($reg:expr, H) => {
        ($reg.f & 0b0010_0000) != 0
    };
    ($reg:expr, C) => {
        ($reg.f & 0b0001_0000) != 0
    };

    ($reg:expr, Z = $val:expr) => {
        if $val {
            $reg.f |= 0b1000_0000
        } else {
            $reg.f &= !0b1000_0000
        }
    };
    ($reg:expr, N = $val:expr) => {
        if $val {
            $reg.f |= 0b0100_0000
        } else {
            $reg.f &= !0b0100_0000
        }
    };
    ($reg:expr, H = $val:expr) => {
        if $val {
            $reg.f |= 0b0010_0000
        } else {
            $reg.f &= !0b0010_0000
        }
    };
    ($reg:expr, C = $val:expr) => {
        if $val {
            $reg.f |= 0b0001_0000
        } else {
            $reg.f &= !0b0001_0000
        }
    };
}

// macro to implement getters&setter methods of u16 registers
macro_rules! words {
    (fn { $fn_get:ident, $fn_set:ident } ($hi:ident, $lo:ident)) => {
        fn $fn_get(&self) -> u16 {
            let hi = self.$hi as u16;
            let lo = self.$lo as u16;
            (hi << 8) | lo
        }

        fn $fn_set(&mut self, data: u16) {
            let hi = (data >> 8) & 0xff;
            let lo = data & 0xff;
            self.$hi = hi as _;
            self.$lo = lo as _;
        }
    };
}

impl Registers {
    words!(fn {af, set_af} (a, f));
    words!(fn {bc, set_bc} (b, c));
    words!(fn {de, set_de} (d, e));
    words!(fn {hl, set_hl} (h, l));
}

#[cfg(test)]
mod test {
    use super::Registers;

    #[test]
    fn flags() {
        let registers = Registers {
            f: 0b1010_0000,
            ..Default::default()
        };

        assert!(flag!(registers, Z));
        assert!(!flag!(registers, N));
        assert!(flag!(registers, H));
        assert!(!flag!(registers, C));
    }

    #[test]
    fn flags_set_true() {
        let mut registers = Registers {
            f: 0,
            ..Default::default()
        };

        flag!(registers, Z = true);
        flag!(registers, N = true);
        flag!(registers, H = true);
        flag!(registers, C = true);

        assert_eq!(0b1111_0000, registers.f);
    }

    #[test]
    fn flags_set_false() {
        let mut registers = Registers {
            f: 0xff,
            ..Default::default()
        };

        flag!(registers, Z = false);
        flag!(registers, N = false);
        flag!(registers, H = false);
        flag!(registers, C = false);

        assert_eq!(0b0000_1111, registers.f);
    }

    #[test]
    fn af() {
        let mut registers = Registers::default();
        registers.a = 0x01;
        registers.f = 0x23;
        assert_eq!(0x0123, registers.af());
    }

    #[test]
    fn bc() {
        let mut registers = Registers::default();
        registers.b = 0x01;
        registers.c = 0x23;
        assert_eq!(0x0123, registers.bc());
    }

    #[test]
    fn de() {
        let mut registers = Registers::default();
        registers.d = 0x01;
        registers.e = 0x23;
        assert_eq!(0x0123, registers.de());
    }

    #[test]
    fn hl() {
        let mut registers = Registers::default();
        registers.h = 0x01;
        registers.l = 0x23;
        assert_eq!(0x0123, registers.hl());
    }

    #[test]
    fn set_af() {
        let mut registers = Registers::default();
        registers.set_af(0x0123);
        assert_eq!(0x01, registers.a);
        assert_eq!(0x23, registers.f);
    }

    #[test]
    fn set_bc() {
        let mut registers = Registers::default();
        registers.set_bc(0x0123);
        assert_eq!(0x01, registers.b);
        assert_eq!(0x23, registers.c);
    }

    #[test]
    fn set_de() {
        let mut registers = Registers::default();
        registers.set_de(0x0123);
        assert_eq!(0x01, registers.d);
        assert_eq!(0x23, registers.e);
    }

    #[test]
    fn set_hl() {
        let mut registers = Registers::default();
        registers.set_hl(0x0123);
        assert_eq!(0x01, registers.h);
        assert_eq!(0x23, registers.l);
    }
}
