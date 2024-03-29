// the pixel format feature flags are mutually exclusive
// if more than one of them is enabled, trigger a compilation error
#[cfg(not(any(
all(feature = "argb", not(feature = "rgba"), not(feature = "bgra")),
all(not(feature = "argb"), feature = "rgba", not(feature = "bgra")),
all(not(feature = "argb"), not(feature = "rgba"), feature = "bgra"),
all(not(feature = "argb"), not(feature = "rgba"), not(feature = "bgra")),
)))]
compile_error!("You must enable only one feature flag related to pixel formats (either 'argb', 'rgba', or 'bgra').");

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

cfg_if::cfg_if! {
    if #[cfg(feature = "rgba")] {
        pub(crate) fn r(color: &Color) -> u8 { color[0] }
        pub(crate) fn g(color: &Color) -> u8 { color[1] }
        pub(crate) fn b(color: &Color) -> u8 { color[2] }

        pub(crate) const fn color(r: u8, g: u8, b: u8) -> Color {
            return [r, g, b, 0xff];
        }
    } else if #[cfg(feature = "bgra")] {
        pub(crate) fn r(color: &Color) -> u8 { color[2] }
        pub(crate) fn g(color: &Color) -> u8 { color[1] }
        pub(crate) fn b(color: &Color) -> u8 { color[0] }

        pub(crate) const fn color(r: u8, g: u8, b: u8) -> Color {
            return [b, g, r, 0xff];
        }
    } else {
        // argb
        pub(crate) fn r(color: &Color) -> u8 { color[2] }
        pub(crate) fn g(color: &Color) -> u8 { color[1] }
        pub(crate) fn b(color: &Color) -> u8 { color[0] }

        pub(crate) const fn color(r: u8, g: u8, b: u8) -> Color {
            return [0xff, r, g, b];
        }
    }
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
    let mut r = r(&color) as u16;
    let mut g = g(&color) as u16;
    let mut b = b(&color) as u16;
    r = r * 3 / 4 + 8;
    g = g * 3 / 4 + 8;
    b = b * 3 / 4 + 8;
    self::color(r as u8, g as u8, b as u8)
}
