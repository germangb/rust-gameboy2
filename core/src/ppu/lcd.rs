use crate::ppu::Color;
use educe::Educe;

cfg_if::cfg_if! {
    if #[cfg(feature = "rgba")] {
        pub(crate) const fn r(color: &Color) -> u8 { color[0] }
        pub(crate) const fn g(color: &Color) -> u8 { color[1] }
        pub(crate) const fn b(color: &Color) -> u8 { color[2] }

        pub(crate) const fn color(r: u8, g: u8, b: u8) -> Color {
            [r, g, b, 0xff]
        }
    } else if #[cfg(feature = "bgra")] {
        pub(crate) const fn r(color: &Color) -> u8 { color[2] }
        pub(crate) const fn g(color: &Color) -> u8 { color[1] }
        pub(crate) const fn b(color: &Color) -> u8 { color[0] }

        pub(crate) const fn color(r: u8, g: u8, b: u8) -> Color {
            [b, g, r, 0xff]
        }
    } else {
        pub(crate) const fn r(color: &Color) -> u8 { color[2] }
        pub(crate) const fn g(color: &Color) -> u8 { color[1] }
        pub(crate) const fn b(color: &Color) -> u8 { color[0] }

        pub(crate) const fn color(r: u8, g: u8, b: u8) -> Color {
            [b, g, r, 0xff]
        }

        pub(crate) const fn color_from_u32(rgb: u32) -> Color {
            [((rgb >> 16) & 0xff) as u8, ((rgb >> 8) & 0xff) as u8, ((rgb >> 0) & 0xff) as u8, 0xff]
        }
    }
}

pub(super) type ColorId = usize;
pub(super) type Line = [ColorId; super::LCD_WIDTH];
pub(super) type ColorLine = [Color; super::LCD_WIDTH];

#[derive(Debug, Clone, Eq, PartialEq, Educe)]
#[educe(Deref, DerefMut)]
pub(super) struct LineBuffer(#[educe(Deref, DerefMut)] pub Line);

impl Default for LineBuffer {
    fn default() -> Self {
        Self([Default::default(); super::LCD_WIDTH])
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Educe)]
#[educe(Deref, DerefMut)]
pub(super) struct ColorLineBuffer(#[educe(Deref, DerefMut)] pub ColorLine);

impl Default for ColorLineBuffer {
    fn default() -> Self {
        Self([Default::default(); super::LCD_WIDTH])
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Educe)]
#[educe(Deref, DerefMut)]
pub(super) struct BoolLineBuffer(#[educe(Deref, DerefMut)] pub [bool; super::LCD_WIDTH]);

impl Default for BoolLineBuffer {
    fn default() -> Self {
        Self([Default::default(); super::LCD_WIDTH])
    }
}
