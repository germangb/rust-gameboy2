use educe::Educe;

pub const PALETTE: [Color; 4] = [0x7e8416, 0x577b46, 0x385d49, 0x2e463d];

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub type Color = u32;
pub type Display = [Color; WIDTH * HEIGHT];

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
    let mut r = color >> 16;
    let mut g = (color >> 8) & 0xff;
    let mut b = color & 0xff;
    r = (r * 3) / 4 + 8;
    g = (g * 3) / 4 + 8;
    b = (b * 3) / 4 + 8;
    (r << 16) | (g << 8) | b
}
