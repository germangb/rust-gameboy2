pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub type Pixel = u32;
pub type Display = [Pixel; WIDTH * HEIGHT];

// wrapper type, because serde doesn't support big arrays
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(super) struct DisplaySerde(pub Display);

impl Default for DisplaySerde {
    fn default() -> Self {
        Self([Default::default(); WIDTH * HEIGHT])
    }
}
