use crate::ppu::lcd;
use embedded_graphics::{drawable, pixelcolor::Rgb888, prelude::*, DrawTarget};
use palette::{
    encoding::{Linear, Srgb},
    Mix,
};

pub(crate) const DEBUG_OBJ: lcd::Color = lcd::color(0xff, 0x00, 0x00);
pub(crate) const DEBUG_OBJ_DOUBLE: lcd::Color = lcd::color(0x00, 0x00, 0xff);

type Rgb = palette::rgb::Rgb<Linear<Srgb>, f64>;

pub fn mix(left: lcd::Color, right: lcd::Color, t: f64) -> lcd::Color {
    from_rgb(&to_rgb(left).mix(&to_rgb(right), t))
}

fn to_rgb(color: lcd::Color) -> Rgb {
    let r = lcd::r(&color) as f64 / 255.0;
    let g = lcd::g(&color) as f64 / 255.0;
    let b = lcd::b(&color) as f64 / 255.0;
    Rgb::new(r, g, b)
}

fn from_rgb(color: &Rgb) -> lcd::Color {
    lcd::color(
        (color.red * 255.0) as u8,
        (color.green * 255.0) as u8,
        (color.blue * 255.0) as u8,
    )
}

impl DrawTarget<Rgb888> for lcd::DisplayBuffer {
    type Error = std::convert::Infallible;

    fn draw_pixel(&mut self, item: drawable::Pixel<Rgb888>) -> Result<(), Self::Error> {
        let drawable::Pixel(point, color) = item;

        let y = (point.y as usize) % lcd::HEIGHT;
        let x = (point.x as usize) % lcd::WIDTH;

        self.0[lcd::WIDTH * y + x] = lcd::color(color.r(), color.g(), color.b());

        Ok(())
    }

    fn size(&self) -> Size {
        Size {
            width: lcd::WIDTH as _,
            height: lcd::HEIGHT as _,
        }
    }
}
