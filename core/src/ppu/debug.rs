use crate::ppu::lcd;
use embedded_graphics::{drawable, pixelcolor::Rgb888, prelude::*, DrawTarget};
use palette::{
    encoding::{Linear, Srgb},
    Mix,
};

// debug drawing of sprites
pub(crate) const DEBUG_OBJ: lcd::Pixel = 0xff0000;
pub(crate) const DEBUG_OBJ_DOUBLE: lcd::Pixel = 0x0000ff;

type Rgb = palette::rgb::Rgb<Linear<Srgb>, f64>;

pub fn mix(left: lcd::Pixel, right: lcd::Pixel, t: f64) -> lcd::Pixel {
    from_rgb(&to_rgb(left).mix(&to_rgb(right), t))
}

fn to_rgb(color: lcd::Pixel) -> Rgb {
    let r = ((color >> 16) & 0xff) as f64 / 255.0;
    let g = ((color >> 8) & 0xff) as f64 / 255.0;
    let b = (color & 0xff) as f64 / 255.0;
    Rgb::new(r, g, b)
}

fn from_rgb(color: &Rgb) -> lcd::Pixel {
    let r = ((color.red * 255.0) as lcd::Pixel) << 16;
    let g = ((color.green * 255.0) as lcd::Pixel) << 8;
    let b = (color.blue * 255.0) as lcd::Pixel;
    r | g | b
}

impl DrawTarget<Rgb888> for lcd::DisplaySerde {
    type Error = std::convert::Infallible;

    fn draw_pixel(&mut self, item: drawable::Pixel<Rgb888>) -> Result<(), Self::Error> {
        let drawable::Pixel(point, color) = item;

        let y = (point.y as usize) % lcd::HEIGHT;
        let x = (point.x as usize) % lcd::WIDTH;

        let r = (color.r() as lcd::Pixel) << 16;
        let g = (color.g() as lcd::Pixel) << 8;
        let b = color.b() as lcd::Pixel;
        self.0[lcd::WIDTH * y + x] = r | g | b;

        Ok(())
    }

    fn size(&self) -> Size {
        Size {
            width: lcd::WIDTH as _,
            height: lcd::HEIGHT as _,
        }
    }
}
