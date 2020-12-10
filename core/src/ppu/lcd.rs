pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub type Pixel = u32;

// wrapper type, because serde doesn't support big arrays
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LcdBuffer(pub [Pixel; WIDTH * HEIGHT]);

impl Default for LcdBuffer {
    fn default() -> Self {
        Self([Default::default(); WIDTH * HEIGHT])
    }
}
