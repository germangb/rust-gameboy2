use educe::Educe;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub type Pixel = u32;
pub type Display = [Pixel; WIDTH * HEIGHT];

// wrapper type, because serde doesn't support big arrays
#[derive(Debug, Clone, Eq, PartialEq, Hash, Educe)]
#[educe(Deref, DerefMut)]
pub(super) struct DisplaySerde(#[educe(Deref, DerefMut)] pub Display);

impl Default for DisplaySerde {
    fn default() -> Self {
        Self([Default::default(); WIDTH * HEIGHT])
    }
}
