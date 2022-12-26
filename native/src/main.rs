use core::{
    cartridge::Cartridge,
    gb::GameBoy,
    joypad::Button,
    ppu::{Color, ColorPalette, LCD, LCD_HEIGHT, LCD_WIDTH},
};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{cell::RefCell, convert::Infallible, io::Cursor, rc::Rc};

// LCD window
const WINDOW_LCD_TITLE: &str = "LCD";
const WINDOW_LCD_W: usize = LCD_WIDTH + 16;
const WINDOW_LCD_H: usize = LCD_HEIGHT;

// VRAM window
const WINDOW_VRAM_TITLE: &str = "VRAM";
const WINDOW_VRAM_W: usize = 24 * 8;
const WINDOW_VRAM_H: usize = 16 * 8;

type Display = [Color; WINDOW_LCD_W * WINDOW_LCD_H];

struct CameraSensor {
    fosdem: image::GrayImage,
    ferris: image::GrayImage,
    time: u32,
}

impl CameraSensor {
    #[rustfmt::skip]
    fn new() -> Self {
        Self {
            fosdem: image::load_from_memory(include_bytes!("fosdem.png")).unwrap().to_luma8(),
            ferris: image::load_from_memory(include_bytes!("ferris.png")).unwrap().to_luma8(),
            time: 0,
        }
    }

    fn fosdem(&self, buf: &mut [[u8; 128]; 112]) {
        for i in 0..112 {
            for j in 0..128 {
                buf[i as usize][j as usize] = self
                    .fosdem
                    .get_pixel(
                        ((j * 3) / 2 + self.time) % 128,
                        ((i * 3) / 2 + self.time) % 112,
                    )
                    .0[0];
            }
        }
    }

    fn ferris(&self, buf: &mut [[u8; 128]; 112]) {
        for i in 0..112 {
            for j in 0..128 {
                let time = (self.time as f64) * 0.1;
                let (si, sj) = (i as f64 - 56.0, j as f64 - 64.0);
                let (si, sj) = (
                    (si * time.sin() + sj * time.cos()) + 56.0,
                    (si * time.cos() - sj * time.sin()) + 64.0,
                );
                buf[i][j] = self
                    .ferris
                    .get_pixel(sj.clamp(0.0, 127.0) as _, si.clamp(0.0, 111.0) as _)
                    .0[0];
            }
        }
    }
}

impl camera::Sensor for CameraSensor {
    fn capture(&mut self, buf: &mut [[u8; 128]; 112]) {
        self.time += 1;
        match (self.time / 64) % 2 {
            0 => self.ferris(buf),
            1 => self.fosdem(buf),
            _ => unreachable!(),
        }
    }
}

struct LCDDisplay(Rc<RefCell<Display>>);
struct EmbeddedGraphicsDisplay(Rc<RefCell<Display>>);

impl LCD for LCDDisplay {
    fn output_line(&mut self, ly: u16, data: &[Color; LCD_WIDTH]) {
        let display = self.0.borrow_mut();
        let off = WINDOW_LCD_W * (ly as usize);
        let len = LCD_WIDTH * 4;
        let src_raw = data.as_ptr() as *const u8;
        let src = unsafe { std::slice::from_raw_parts(src_raw, len) };
        let dst_raw = display[off..(off + LCD_WIDTH)].as_ptr() as *mut u8;
        unsafe { std::slice::from_raw_parts_mut(dst_raw, len) }.copy_from_slice(src);
    }
}

impl Dimensions for EmbeddedGraphicsDisplay {
    fn bounding_box(&self) -> Rectangle {
        Rectangle {
            top_left: Point::new(0, 0),
            size: Size::new(WINDOW_LCD_W as _, WINDOW_LCD_H as _),
        }
    }
}

impl DrawTarget for EmbeddedGraphicsDisplay {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let mut display = self.0.borrow_mut();
        for Pixel(coord, color) in pixels {
            let x = coord.x as usize;
            let y = coord.y as usize;
            display[y * WINDOW_LCD_W + x] = [color.b(), color.g(), color.r(), 0xff];
        }
        Ok(())
    }
}

fn color_style(color: [u8; 4]) -> PrimitiveStyle<Rgb888> {
    fn color_to_rgb888(color: [u8; 4]) -> Rgb888 {
        Rgb888::new(color[2], color[1], color[0])
    }
    PrimitiveStyleBuilder::new()
        .fill_color(color_to_rgb888(color))
        .stroke_alignment(StrokeAlignment::Inside)
        .stroke_width(1)
        .stroke_color(Rgb888::BLACK)
        .build()
}

fn draw_color_palette(
    palette: &ColorPalette,
    size: i32,
    offx: i32,
    offy: i32,
    display: &mut EmbeddedGraphicsDisplay,
) {
    let square = Size::new(size as _, size as _);
    Rectangle::new(Point::new(offx + 160, offy + size * 0), square)
        .into_styled(color_style(palette[0]))
        .draw(display)
        .unwrap();
    Rectangle::new(Point::new(offx + 160, offy + size * 1), square)
        .into_styled(color_style(palette[1]))
        .draw(display)
        .unwrap();
    Rectangle::new(Point::new(offx + 160, offy + size * 2), square)
        .into_styled(color_style(palette[2]))
        .draw(display)
        .unwrap();
    Rectangle::new(Point::new(offx + 160, offy + size * 3), square)
        .into_styled(color_style(palette[3]))
        .draw(display)
        .unwrap();
}

#[cfg(feature = "cgb")]
fn draw_color_palettes(
    gb: &GameBoy<impl Cartridge, impl LCD>,
    display: &mut EmbeddedGraphicsDisplay,
) {
    draw_color_palette(&gb.soc().ppu().obp()[0], 4, 0, 0, display);
    draw_color_palette(&gb.soc().ppu().obp()[1], 4, 4, 0, display);
    draw_color_palette(&gb.soc().ppu().obp()[2], 4, 8, 0, display);
    draw_color_palette(&gb.soc().ppu().obp()[3], 4, 12, 0, display);
    draw_color_palette(&gb.soc().ppu().obp()[4], 4, 0, 16, display);
    draw_color_palette(&gb.soc().ppu().obp()[5], 4, 4, 16, display);
    draw_color_palette(&gb.soc().ppu().obp()[6], 4, 8, 16, display);
    draw_color_palette(&gb.soc().ppu().obp()[7], 4, 12, 16, display);
    draw_color_palette(&gb.soc().ppu().bgp()[0], 4, 0, 32, display);
    draw_color_palette(&gb.soc().ppu().bgp()[1], 4, 4, 32, display);
    draw_color_palette(&gb.soc().ppu().bgp()[2], 4, 8, 32, display);
    draw_color_palette(&gb.soc().ppu().bgp()[3], 4, 12, 32, display);
    draw_color_palette(&gb.soc().ppu().bgp()[4], 4, 0, 48, display);
    draw_color_palette(&gb.soc().ppu().bgp()[5], 4, 4, 48, display);
    draw_color_palette(&gb.soc().ppu().bgp()[6], 4, 8, 48, display);
    draw_color_palette(&gb.soc().ppu().bgp()[7], 4, 12, 48, display);
}

#[cfg(not(feature = "cgb"))]
fn draw_color_palettes(
    gb: &GameBoy<impl Cartridge, impl LCD>,
    display: &mut EmbeddedGraphicsDisplay,
) {
    draw_color_palette(gb.soc().ppu().bgp(), 8, 0, 32, display);
    draw_color_palette(gb.soc().ppu().obp0(), 8, 0, 0, display); // obp0
    draw_color_palette(gb.soc().ppu().obp1(), 8, 8, 0, display); // obp1
}

fn make_emulator_cartridge() -> impl Cartridge {
    //camera::PocketCamera::new(CameraSensor::new())
    //core::cartridge::MBC5::new(include_bytes!("cannon-fodder.gbc").to_vec().into_boxed_slice())
    core::cartridge::MBC5::new(include_bytes!("gold.gbc").to_vec().into_boxed_slice())
}

fn make_emulator_display() -> (LCDDisplay, Rc<RefCell<Display>>) {
    let display = Rc::new(RefCell::new([[0, 0, 0, 0xff]; WINDOW_LCD_W * WINDOW_LCD_H]));
    let lcd = LCDDisplay(Rc::clone(&display));
    (lcd, display)
}

fn main() {
    pretty_env_logger::init();
    let mut opts_lcd = WindowOptions::default();
    let mut opts_vram = WindowOptions::default();
    opts_lcd.scale = Scale::X2;
    opts_lcd.topmost = true;
    opts_vram.scale = Scale::X2;
    let mut window_lcd =
        Window::new(WINDOW_LCD_TITLE, WINDOW_LCD_W, WINDOW_LCD_H, opts_lcd).unwrap();
    #[cfg(not(feature = "cgb"))]
    let mut window_vram =
        Window::new(WINDOW_VRAM_TITLE, WINDOW_VRAM_W, WINDOW_VRAM_H, opts_vram).unwrap();
    #[rustfmt::skip]
    #[cfg(feature = "cgb")]
    let (mut window_vram_bank0, mut window_vram_bank1) =
        (
            Window::new(&format!("{WINDOW_VRAM_TITLE}[0]"), WINDOW_VRAM_W, WINDOW_VRAM_H, opts_vram).unwrap(),
            Window::new(&format!("{WINDOW_VRAM_TITLE}[1]"), WINDOW_VRAM_W, WINDOW_VRAM_H, opts_vram).unwrap(),
        );
    #[cfg(feature = "cgb")]
    window_lcd.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    #[cfg(not(feature = "cgb"))]
    {
        window_vram.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    }
    #[cfg(feature = "cgb")]
    {
        window_vram_bank0.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        window_vram_bank1.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    }

    let (lcd, display) = make_emulator_display();
    let cartridge = make_emulator_cartridge();
    let mut embedded_graphics_display = EmbeddedGraphicsDisplay(Rc::clone(&display));
    let mut gb = GameBoy::new(cartridge, lcd).unwrap();
    let mut paused = false;
    let mut frame = 0;
    #[cfg(feature = "debug")]
    let mut debug = false;
    while window_lcd.is_open() {
        handle_key(&window_lcd, &mut gb, Key::Z, Button::A);
        handle_key(&window_lcd, &mut gb, Key::X, Button::B);
        handle_key(&window_lcd, &mut gb, Key::Enter, Button::Start);
        handle_key(&window_lcd, &mut gb, Key::RightShift, Button::Select);
        handle_key(&window_lcd, &mut gb, Key::Left, Button::Left);
        handle_key(&window_lcd, &mut gb, Key::Right, Button::Right);
        handle_key(&window_lcd, &mut gb, Key::Up, Button::Up);
        handle_key(&window_lcd, &mut gb, Key::Down, Button::Down);
        if !paused {
            gb.emulate_frame().unwrap();
            frame += 1;
            if window_lcd.is_key_down(Key::F) {
                gb.emulate_frame().unwrap();
                gb.emulate_frame().unwrap();
                gb.emulate_frame().unwrap();
                gb.emulate_frame().unwrap();
                gb.emulate_frame().unwrap();
                gb.emulate_frame().unwrap();
                gb.emulate_frame().unwrap();
                frame += 7;
            }
        } else {
            if window_lcd.is_key_pressed(Key::S, KeyRepeat::Yes) {
                gb.emulate_frame().unwrap();
                frame += 1;
            }
        }
        if window_lcd.is_key_pressed(Key::P, KeyRepeat::No) {
            paused = !paused;
        }
        #[cfg(feature = "debug")]
        if window_lcd.is_key_pressed(Key::D, KeyRepeat::No) {
            debug = !debug;
            gb.set_debug_overlays(debug);
        }
        draw_color_palettes(&gb, &mut embedded_graphics_display);
        let buffer = unsafe {
            let raw = display.as_ptr() as *const u32;
            std::slice::from_raw_parts(raw, WINDOW_LCD_W * WINDOW_LCD_H)
        };
        window_lcd
            .update_with_buffer(buffer, WINDOW_LCD_W, WINDOW_LCD_H)
            .unwrap();
        #[cfg(not(feature = "cgb"))]
        {
            let buffer = unsafe {
                let raw = gb.soc().vram().tile_data().as_ptr() as *const u32;
                std::slice::from_raw_parts(raw, 24 * 16 * 8 * 8)
            };
            window_vram
                .update_with_buffer(buffer, 24 * 8, 16 * 8)
                .unwrap();
        }
        #[cfg(feature = "cgb")]
        {
            let len = gb.soc().vram().tile_data().len();
            let buffer = unsafe {
                let raw = gb.soc().vram().tile_data()[..len / 2].as_ptr() as *const u32;
                std::slice::from_raw_parts(raw, 24 * 16 * 8 * 8)
            };
            window_vram_bank0
                .update_with_buffer(buffer, 24 * 8, 16 * 8)
                .unwrap();
            let buffer = unsafe {
                let raw = gb.soc().vram().tile_data()[len / 2..].as_ptr() as *const u32;
                std::slice::from_raw_parts(raw, 24 * 16 * 8 * 8)
            };
            window_vram_bank1
                .update_with_buffer(buffer, 24 * 8, 16 * 8)
                .unwrap();
        }
    }
}

fn handle_key(
    window: &Window,
    gb: &mut GameBoy<impl Cartridge, impl LCD>,
    key: Key,
    button: Button,
) {
    if window.is_key_pressed(key, KeyRepeat::No) {
        gb.press(&button);
    }
    if window.is_key_released(key) {
        gb.release(&button);
    }
}
