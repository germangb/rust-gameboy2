use crate::ppu::{lcd, Color};
use palette::{
    encoding::{Linear, Srgb},
    Mix,
};

pub(crate) const DEBUG_OBJ: Color = lcd::color(0xff, 0x00, 0x00);
pub(crate) const DEBUG_OBJ_DOUBLE: Color = lcd::color(0x00, 0x00, 0xff);

type Rgb = palette::rgb::Rgb<Linear<Srgb>, f64>;

pub fn mix(left: Color, right: Color, t: f64) -> Color {
    from_rgb(&to_rgb(left).mix(&to_rgb(right), t))
}

fn to_rgb(color: Color) -> Rgb {
    let r = lcd::r(&color) as f64 / 255.0;
    let g = lcd::g(&color) as f64 / 255.0;
    let b = lcd::b(&color) as f64 / 255.0;
    Rgb::new(r, g, b)
}

fn from_rgb(color: &Rgb) -> Color {
    lcd::color(
        (color.red * 255.0) as u8,
        (color.green * 255.0) as u8,
        (color.blue * 255.0) as u8,
    )
}
