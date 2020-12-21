use crate::ppu::lcd;
use cfg_if::cfg_if;
use educe::Educe;

pub const PALETTE: [Color; 4] = [
    color(0x7e, 0x84, 0x16),
    color(0x57, 0x7b, 0x46),
    color(0x38, 0x5d, 0x49),
    color(0x2e, 0x46, 0x3d),
];

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub type Color = [u8; 4];
pub type Display = [Color; WIDTH * HEIGHT];

cfg_if! {
    if #[cfg(feature = "rgba")] {
        pub(crate) fn r(color: &Color) -> u8 { color[0] }
        pub(crate) fn g(color: &Color) -> u8 { color[1] }
        pub(crate) fn b(color: &Color) -> u8 { color[2] }
    } else {
        // argb
        pub(crate) fn r(color: &Color) -> u8 { color[1] }
        pub(crate) fn g(color: &Color) -> u8 { color[2] }
        pub(crate) fn b(color: &Color) -> u8 { color[3] }
    }
}

pub(crate) const fn color(r: u8, g: u8, b: u8) -> Color {
    #[cfg(feature = "rgba")]
    return [r, g, b, 0xff];
    #[cfg(not(feature = "rgba"))]
    return [0xff, r, g, b];
}

#[derive(Debug, Clone, Eq, PartialEq, Educe)]
#[educe(Deref, DerefMut)]
pub(super) struct DisplayBuffer(#[educe(Deref, DerefMut)] pub Display);

impl Default for DisplayBuffer {
    fn default() -> Self {
        Self([Default::default(); WIDTH * HEIGHT])
    }
}

pub(super) type ColorId = usize;
pub(super) type Line = [ColorId; WIDTH];

#[derive(Debug, Clone, Eq, PartialEq, Educe)]
#[educe(Deref, DerefMut)]
pub(super) struct LineBuffer(#[educe(Deref, DerefMut)] pub Line);

impl Default for LineBuffer {
    fn default() -> Self {
        Self([Default::default(); WIDTH])
    }
}

pub(super) fn transform(color: Color) -> Color {
    let mut r = lcd::r(&color) as u16;
    let mut g = lcd::g(&color) as u16;
    let mut b = lcd::b(&color) as u16;
    r = r * 3 / 4 + 8;
    g = g * 3 / 4 + 8;
    b = b * 3 / 4 + 8;
    lcd::color(r as u8, g as u8, b as u8)
}
