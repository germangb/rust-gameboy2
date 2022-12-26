use crate::{cartridge::Cartridge, device::Device, ppu::LCD, LR35902};

pub trait Breakpoint {
    /// Called right before stepping the emulation.
    fn init(&mut self, soc: &LR35902<impl Cartridge, impl LCD>);

    /// Called right after stepping the emulation, to determine if the
    /// breakpoint has been hit. Return true if it has been hit, o false
    /// otherwise.
    fn breakpoint(&self, soc: &LR35902<impl Cartridge, impl LCD>) -> bool;
}

impl Breakpoint for () {
    fn init(&mut self, soc: &LR35902<impl Cartridge, impl LCD>) {}

    fn breakpoint(&self, soc: &LR35902<impl Cartridge, impl LCD>) -> bool {
        false
    }
}

/// Breakpoint when new frame is reached (reach OAM SEARCH captured in STAT
/// register).
#[derive(Debug, Clone)]
pub struct NextFrame {
    stat: u8,
}

impl NextFrame {
    pub fn new() -> Self {
        Self { stat: 0 }
    }
}

impl Breakpoint for NextFrame {
    fn init(&mut self, soc: &LR35902<impl Cartridge, impl LCD>) {
        self.stat = soc.read(0xff41).unwrap();
    }

    fn breakpoint(&self, soc: &LR35902<impl Cartridge, impl LCD>) -> bool {
        let stat = soc.read(0xff41).unwrap();
        (self.stat & 0b11) == 1 && (stat & 0b11) == 2
    }
}

/// Breakpoint when the PC reached a particular value.
#[derive(Debug, Clone)]
pub struct PC(pub u16);

impl Breakpoint for PC {
    fn init(&mut self, soc: &LR35902<impl Cartridge, impl LCD>) {}

    fn breakpoint(&self, soc: &LR35902<impl Cartridge, impl LCD>) -> bool {
        soc.cpu().registers().pc == self.0
    }
}

/// Breakpoint when LY register reaches a particular value
#[derive(Debug, Clone)]
pub struct LY {
    ly_pre: u8,
    ly: u8,
}

impl LY {
    pub fn new(ly: u8) -> Self {
        Self { ly_pre: 0, ly }
    }
}

impl Breakpoint for LY {
    fn init(&mut self, soc: &LR35902<impl Cartridge, impl LCD>) {
        self.ly_pre = soc.read(0xff44).unwrap();
    }

    fn breakpoint(&self, soc: &LR35902<impl Cartridge, impl LCD>) -> bool {
        let ly = soc.read(0xff44).unwrap();
        self.ly_pre != self.ly && ly == self.ly
    }
}

macro_rules! tuple {
    ($($gen:ident,)*) => {
        impl<$($gen:Breakpoint,)*> Breakpoint for ($($gen,)*) {
            fn init(&mut self, soc: &LR35902<impl Cartridge, impl LCD>) {
                let ($($gen,)*) = self;
                $($gen.init(soc);)*
            }

            fn breakpoint(&self, soc: &LR35902<impl Cartridge, impl LCD>) -> bool {
                let ($($gen,)*) = self;
                false  $( || $gen.breakpoint(soc))*
            }
        }
    }
}

tuple!(A,);
tuple!(A, B,);
tuple!(A, B, C,);
tuple!(A, B, C, D,);

#[derive(Debug, Clone)]
pub enum Either<A: Breakpoint, B: Breakpoint> {
    A(A),
    B(B),
}

impl<A: Breakpoint, B: Breakpoint> Breakpoint for Either<A, B> {
    fn init(&mut self, soc: &LR35902<impl Cartridge, impl LCD>) {
        match self {
            Either::A(breakpoint) => breakpoint.init(soc),
            Either::B(breakpoint) => breakpoint.init(soc),
        }
    }

    fn breakpoint(&self, soc: &LR35902<impl Cartridge, impl LCD>) -> bool {
        match self {
            Either::A(breakpoint) => breakpoint.breakpoint(soc),
            Either::B(breakpoint) => breakpoint.breakpoint(soc),
        }
    }
}
