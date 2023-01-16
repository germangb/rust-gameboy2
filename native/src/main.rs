use core::{
    cartridge::{Cartridge, MBC1, MBC2, MBC3, MBC5, ROM},
    cpu::Registers,
    debug::Breakpoint,
    device::Device,
    joypad::Button,
    ppu::{Color, ColorPalette, LCDDebugOverlay, LCD, LCD_HEIGHT, LCD_WIDTH},
    ram::vram::TileDataCache,
};
use dialog::{DialogBox, FileSelectionMode};
use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle},
    text::Text,
};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::{cell::RefCell, convert::Infallible, rc::Rc};

type GameBoy = core::gb::GameBoy<Box<dyn Cartridge>, GameBoyLCD>;

// VRAM window
const WINDOW_VRAM_TITLE: &str = "VRAM";
const WINDOW_VRAM_W: usize = TileDataCache::DOTS_WIDTH;
const WINDOW_VRAM_H: usize = TileDataCache::DOTS_HEIGHT;

// LCD window
const WINDOW_LCD_TITLE: &str = "LCD";
const WINDOW_LCD_W: usize = LCD_WIDTH + 16;
const WINDOW_LCD_H: usize = LCD_HEIGHT + 7;

// CPU window
const WINDOW_CPU_TITLE: &str = "CPU";
const WINDOW_CPU_W: usize = 5 * 30;
const WINDOW_CPU_H: usize = 7 * 16;

// MEM window
const WINDOW_MEM_TITLE: &str = "MEM";
const WINDOW_MEM_W: usize = 5 * 52;
const WINDOW_MEM_H: usize = 7 * 16;

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

struct GameBoyLCD(Rc<RefCell<[Color; WINDOW_LCD_W * WINDOW_LCD_H]>>);
impl LCD for GameBoyLCD {
    fn output_line(&mut self, ly: u8, data: &[Color; LCD_WIDTH]) {
        let display = self.0.borrow_mut();
        let off = WINDOW_LCD_W * (ly as usize);
        let len = LCD_WIDTH * 4;
        let src_raw = data.as_ptr() as *const u8;
        let src = unsafe { std::slice::from_raw_parts(src_raw, len) };
        let dst_raw = display[off..(off + LCD_WIDTH)].as_ptr() as *mut u8;
        unsafe { std::slice::from_raw_parts_mut(dst_raw, len) }.copy_from_slice(src);
    }
}

struct LCDDrawTarget(Rc<RefCell<[Color; WINDOW_LCD_W * WINDOW_LCD_H]>>);
struct EGDrawTarget {
    pub buffer: Box<[u32]>,
    pub w: usize,
    pub h: usize,
}

impl EGDrawTarget {
    fn new(w: usize, h: usize) -> Self {
        let buffer = vec![0; w * h].into_boxed_slice();
        Self { buffer, w, h }
    }
}

impl Dimensions for LCDDrawTarget {
    fn bounding_box(&self) -> Rectangle {
        Rectangle {
            top_left: Point::new(0, 0),
            size: Size::new(WINDOW_LCD_W as _, WINDOW_LCD_H as _),
        }
    }
}

impl Dimensions for EGDrawTarget {
    fn bounding_box(&self) -> Rectangle {
        Rectangle {
            top_left: Point::new(0, 0),
            size: Size::new(self.w as _, self.h as _),
        }
    }
}

impl DrawTarget for LCDDrawTarget {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let mut display = self.0.borrow_mut();
        let bb = self.bounding_box();
        for Pixel(coord, color) in pixels.into_iter().filter(|Pixel(c, _)| bb.contains(*c)) {
            let x = coord.x as usize;
            let y = coord.y as usize;
            display[y * WINDOW_LCD_W + x] = [color.b(), color.g(), color.r(), 0xff];
        }
        Ok(())
    }
}

impl DrawTarget for EGDrawTarget {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();
        for Pixel(coord, color) in pixels.into_iter().filter(|Pixel(c, _)| bb.contains(*c)) {
            let x = coord.x as usize;
            let y = coord.y as usize;
            let (r, g, b) = (color.r() as u32, color.g() as u32, color.b() as u32);
            self.buffer[y * self.w + x] = r << 16 | g << 8 | b;
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
        //.stroke_width(1)
        .stroke_color(Rgb888::BLACK)
        .build()
}

fn draw_color_palette(
    palette: &ColorPalette,
    size: i32,
    offx: i32,
    offy: i32,
    display: &mut LCDDrawTarget,
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
fn draw_color_palettes(gb: &GameBoy, display: &mut LCDDrawTarget) {
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
fn draw_color_palettes(gb: &GameBoy, display: &mut LCDDrawTarget) {
    draw_color_palette(gb.soc().ppu().bgp(), 8, 0, 32, display);
    draw_color_palette(gb.soc().ppu().obp0(), 8, 0, 0, display); // obp0
    draw_color_palette(gb.soc().ppu().obp1(), 8, 8, 0, display); // obp1
}

struct Windows {
    pub window_lcd: Window,
    pub window_cpu: Window,
    #[cfg(feature = "mem")]
    pub window_mem: Window,
    #[cfg(not(feature = "cgb"))]
    pub window_vram: Window,
    #[cfg(feature = "cgb")]
    pub window_vram0: Window,
    #[cfg(feature = "cgb")]
    pub window_vram1: Window,
}

impl Windows {
    fn new() -> Self {
        #[rustfmt::skip]
        #[cfg(not(feature = "cgb"))]
        let window_vram =
        Window::new(WINDOW_VRAM_TITLE, WINDOW_VRAM_W, WINDOW_VRAM_H, WindowOptions::default()).unwrap();
        #[rustfmt::skip]
        #[cfg(feature = "cgb")]
        let (window_vram0, window_vram1) =
        (
            Window::new(&format!("{WINDOW_VRAM_TITLE} 0"), WINDOW_VRAM_W, WINDOW_VRAM_H, WindowOptions::default()).unwrap(),
            Window::new(&format!("{WINDOW_VRAM_TITLE} 1"), WINDOW_VRAM_W, WINDOW_VRAM_H, WindowOptions::default()).unwrap(),
        );
        let mut opts_lcd = WindowOptions::default();
        opts_lcd.scale = Scale::X2;
        let mut window_lcd =
            Window::new(WINDOW_LCD_TITLE, WINDOW_LCD_W, WINDOW_LCD_H, opts_lcd).unwrap();
        window_lcd.limit_update_rate(Some(std::time::Duration::from_micros(16750)));
        let mut opts_cpu = WindowOptions::default();
        opts_cpu.scale = Scale::X2;
        let mut window_cpu =
            Window::new(WINDOW_CPU_TITLE, WINDOW_CPU_W, WINDOW_CPU_H, opts_cpu).unwrap();
        window_cpu.limit_update_rate(Some(std::time::Duration::from_micros(16750)));
        #[cfg(feature = "mem")]
        let mut opts_mem = WindowOptions::default();
        #[cfg(feature = "mem")]
        {
            opts_mem.scale = Scale::X1;
        }
        #[cfg(feature = "mem")]
        let mut window_mem =
            Window::new(WINDOW_MEM_TITLE, WINDOW_MEM_W, WINDOW_MEM_H, opts_mem).unwrap();
        #[cfg(feature = "mem")]
        window_mem.limit_update_rate(Some(std::time::Duration::from_micros(16750)));
        Self {
            window_lcd,
            window_cpu,
            #[cfg(feature = "mem")]
            window_mem,
            #[cfg(not(feature = "cgb"))]
            window_vram,
            #[cfg(feature = "cgb")]
            window_vram0,
            #[cfg(feature = "cgb")]
            window_vram1,
        }
    }

    fn is_key_pressed(&self, key: Key, repeat: KeyRepeat) -> bool {
        #[cfg(not(feature = "cgb"))]
        let vram = self.window_vram.is_key_pressed(key, repeat);
        #[cfg(feature = "cgb")]
        let vram = self.window_vram0.is_key_pressed(key, repeat)
            || self.window_vram1.is_key_pressed(key, repeat);
        #[cfg(not(feature = "mem"))]
        let mem = false;
        #[cfg(feature = "mem")]
        let mem = self.window_mem.is_key_pressed(key, repeat);
        self.window_lcd.is_key_pressed(key, repeat)
            || self.window_cpu.is_key_pressed(key, repeat)
            || vram
            || mem
    }

    fn is_open(&self) -> bool {
        self.window_lcd.is_open()
    }
}

fn main() {
    pretty_env_logger::init();
    let (mut gb, display) = make_emulator();

    // load rom from std args
    if let Some(path) = std::env::args().skip(1).next() {
        gb = load_rom(Some(&path), Rc::clone(&display), gb);
    } else {
        gb = load_rom(None, Rc::clone(&display), gb);
    }

    let mut windows = Windows::new();

    let mut lcd_eg = LCDDrawTarget(Rc::clone(&display));
    let mut cpu_eg = EGDrawTarget::new(WINDOW_CPU_W, WINDOW_CPU_H);
    #[cfg(feature = "mem")]
    let mut mem_eg = EGDrawTarget::new(WINDOW_MEM_W, WINDOW_MEM_H);

    let mut buf = Box::new([0u8; 0x10000]);
    #[cfg(nope)]
    {
        // run until PC == 0x100
        while gb.soc().cpu().registers().pc != 0x100 {
            gb.soc_mut().step().unwrap();
        }
    }

    // breakpoints
    let mut pc_break = None;
    let mut ly_break = None;

    // state
    let mut pause = false;
    let mut speed = 1;
    let mut frame = 0;
    let mut lcd_debug_overlay = LCDDebugOverlay::all();

    while windows.is_open() {
        frame += 1;
        handle_joypad_input(&windows.window_lcd, &mut gb);
        handle_lcd_debug_overlay(&windows.window_lcd, &mut lcd_debug_overlay);
        gb.soc_mut().ppu_mut().lcd_debug_overlay = lcd_debug_overlay;

        // load ROM
        if windows.is_key_pressed(Key::C, KeyRepeat::No) {
            if let Ok(Some(path)) = dialog::FileSelection::new("ROM")
                .mode(FileSelectionMode::Open)
                .show()
            {
                gb = load_rom(Some(&path), Rc::clone(&display), gb);
                pause = false;
            }
        }

        // reset
        if windows.is_key_pressed(Key::R, KeyRepeat::No) {
            gb = load_rom(None, Rc::clone(&display), gb);
            pause = false;
        }

        // emulation speed
        if windows.is_key_pressed(Key::K, KeyRepeat::Yes) {
            speed += 1;
        }
        if windows.is_key_pressed(Key::J, KeyRepeat::Yes) && speed > 1 {
            speed -= 1;
        }

        let mut disable_pause = false;

        // poke memory
        #[cfg(feature = "mem")]
        if windows.window_mem.is_key_down(Key::RightShift)
            && windows.window_mem.is_key_pressed(Key::P, KeyRepeat::No)
        {
            disable_pause = true;
            let address = if let Ok(address) = dialog::Input::new("address").title("poke").show() {
                address.and_then(|addr| u16::from_str_radix(&addr, 16).ok())
            } else {
                None
            };
            if let Some(address) = address {
                let data = if let Ok(data) = dialog::Input::new("data").title("poke").show() {
                    data.and_then(|addr| u8::from_str_radix(&addr, 16).ok())
                } else {
                    None
                };
                if let Some(data) = data {
                    gb.soc_mut().write(address, data).unwrap();
                }
            }
        }

        // change PC
        if windows.window_cpu.is_key_down(Key::RightShift)
            && windows.window_cpu.is_key_pressed(Key::P, KeyRepeat::No)
        {
            disable_pause = true;
            if let Ok(Some(address)) = dialog::Input::new("PC").show() {
                if let Ok(address) = u16::from_str_radix(&address, 16) {
                    gb.soc_mut().cpu_mut().registers_mut().pc = address;
                }
            };
        }

        // pause emulation
        if !disable_pause && windows.is_key_pressed(Key::P, KeyRepeat::No) {
            pause = !pause;
        }

        // set breakpoint
        if windows.window_cpu.is_key_pressed(Key::B, KeyRepeat::No) {
            if let Ok(Some(address)) = dialog::Input::new("PC").title("breakpoint").show() {
                pc_break = u16::from_str_radix(&address, 16).ok();
            }
        }

        if windows.window_cpu.is_key_pressed(Key::L, KeyRepeat::No) {
            if let Ok(Some(val)) = dialog::Input::new("ly").title("breakpoint").show() {
                ly_break = val.parse::<u8>().ok();
            }
        }

        // read memory
        #[cfg(feature = "mem")]
        if windows.window_mem.is_key_pressed(Key::R, KeyRepeat::No) {
            if let Ok(Some(address)) = dialog::Input::new("address").show() {
                if let Ok(address) = u16::from_str_radix(&address, 16) {
                    dialog::Message::new(format!("{address:04X}"))
                        .show()
                        .unwrap();
                }
            };
        }

        // step emulation
        if !pause {
            // breakpoints
            let mut all = (
                core::debug::NextFrame::new(),
                core::debug::Either::A(()),
                core::debug::Either::A(()),
            );
            if let Some(pc) = pc_break {
                all.1 = core::debug::Either::B(core::debug::PC(pc));
            }
            if let Some(ly) = ly_break {
                all.2 = core::debug::Either::B(core::debug::LY::new(ly));
            }
            match gb.soc_mut().step_breakpoint(all.clone()) {
                Ok(breakpoint) => {
                    pause = breakpoint.1.breakpoint(gb.soc()) || breakpoint.2.breakpoint(gb.soc());
                }
                Err(err) => {
                    pause = true;
                    log::error!("{err:?}");
                }
            }
        } else {
            if windows.window_cpu.is_key_pressed(Key::S, KeyRepeat::Yes) {
                if let Err(err) = gb.soc_mut().step() {
                    pause = true;
                    log::error!("{err:?}");
                }
            }

            if windows.window_lcd.is_key_pressed(Key::S, KeyRepeat::Yes) {
                if let Err(err) = gb.next_frame() {
                    pause = true;
                    log::error!("{err:?}");
                }
            }
        }

        // draw VRAM
        #[cfg(not(feature = "cgb"))]
        {
            let data = gb.soc().vram().tile_data_cache().as_slice();
            #[rustfmt::skip]
            let buf = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u32, data.len()) };
            windows
                .window_vram
                .update_with_buffer(buf, WINDOW_VRAM_W, WINDOW_VRAM_H)
                .unwrap();
        }
        #[cfg(feature = "cgb")]
        {
            let data = gb.soc().vram().tile_data_cache().as_slice();
            let len = data.len();
            #[rustfmt::skip]
            let buf = unsafe { std::slice::from_raw_parts(data[..len / 2].as_ptr() as *const u32, len / 2) };
            #[rustfmt::skip]
            windows.window_vram0.update_with_buffer(buf, WINDOW_VRAM_W, WINDOW_VRAM_H).unwrap();
            #[rustfmt::skip]
            let buf = unsafe { std::slice::from_raw_parts(data[len / 2..].as_ptr() as *const u32, len / 2) };
            windows
                .window_vram1
                .update_with_buffer(buf, WINDOW_VRAM_W, WINDOW_VRAM_H)
                .unwrap();
        }
        // draw LCD
        {
            draw_color_palettes(&gb, &mut lcd_eg);
            Rectangle::new(Point::new(0, LCD_HEIGHT as _), Size::new(LCD_WIDTH as _, 8))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::BLACK))
                .draw(&mut lcd_eg)
                .unwrap();
            if pause {
                let color = if frame / 8 % 2 == 0 {
                    Rgb888::WHITE
                } else {
                    Rgb888::BLACK
                };
                Rectangle::new(Point::new(0, 0), Size::new(LCD_WIDTH as _, LCD_HEIGHT as _))
                    .into_styled(
                        PrimitiveStyleBuilder::new()
                            .stroke_color(color)
                            .stroke_width(1)
                            .stroke_alignment(StrokeAlignment::Inside)
                            .build(),
                    )
                    .draw(&mut lcd_eg)
                    .unwrap();
            }
            #[rustfmt::skip]
            Text::new(
                &format!("x{speed}"),
                Point::new(0, (LCD_HEIGHT + 6) as _),
                MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
            )
                .draw(&mut lcd_eg)
                .unwrap();
            #[rustfmt::skip]
            let buf = unsafe { std::slice::from_raw_parts(display.as_ptr() as *const u32, WINDOW_LCD_W * WINDOW_LCD_H) };
            windows
                .window_lcd
                .update_with_buffer(buf, WINDOW_LCD_W, WINDOW_LCD_H)
                .unwrap();
        }
        // draw CPU
        {
            static BLACK: [u32; WINDOW_CPU_W * WINDOW_CPU_H] = [0; WINDOW_CPU_W * WINDOW_CPU_H];
            cpu_eg.buffer[..].copy_from_slice(&BLACK[..]);
            let pc = gb.soc().cpu().registers().pc as usize;
            let pc_size = 16 - 3;
            let mut addr_buf = String::new();
            let mut inst_buf = String::new();
            let mut mark_buf = String::new();
            for addr in (pc.max(3) - 3)..(pc + pc_size).min(0x10000) {
                addr_buf.push_str(&format!("{addr:04X}\n"));
                mark_buf.push(if addr == pc { '>' } else { '\n' });
                let inst = gb.soc().read(addr as u16).unwrap();
                inst_buf.push_str(&format!("{inst:02X}\n"));
            }
            Text::new(
                &addr_buf,
                Point::new(0, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            Text::new(
                &mark_buf,
                Point::new(20, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::RED),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            Text::new(
                &inst_buf,
                Point::new(25, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            let sp = gb.soc().cpu().registers().sp as usize;
            let sp_size = 7;
            let sp_base = sp.max(sp_size) - sp_size;
            addr_buf.clear();
            mark_buf.clear();
            inst_buf.clear();
            for addr in (sp.max(sp_size) - sp_size)..=(sp + 3).min(0xffff) {
                addr_buf.push_str(&format!("{addr:04X}\n"));
                mark_buf.push(if addr == sp { '>' } else { '\n' });
                let inst = gb.soc().read(addr as u16).unwrap();
                inst_buf.push_str(&format!("{inst:02X}\n"));
            }
            Text::new(
                &addr_buf,
                Point::new(5 * 8 + 0, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            Text::new(
                &mark_buf,
                Point::new(5 * 8 + 20, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::GREEN),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            Text::new(
                &inst_buf,
                Point::new(5 * 8 + 25, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            let freq = gb.soc().clock_freq();
            Text::new(
                &format!("{freq}"),
                Point::new(WINDOW_CPU_W as i32 - 5 * 10, WINDOW_CPU_H as i32 - 1),
                MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            Text::new(
                "Hz",
                Point::new(WINDOW_CPU_W as i32 - 5 * 2, WINDOW_CPU_H as i32 - 1),
                MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            Text::new(
                "A    F\nB    C   \nD    E   \nH    L   \n\nPC\nSP\nHL      (  )",
                Point::new(5 * 8 * 2, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            #[rustfmt::skip]
            let Registers { a, f, b, c, d, e, h, l, sp, pc } = gb.soc().cpu().registers();
            let hl = ((*h as u16) << 8) | (*l as u16);
            let hl_data = gb.soc().read(hl);
            Text::new(
                &format!("  {a:02X}   {f:02X}\n  {b:02X}   {c:02X}\n  {d:02X}   {e:02X}\n  {h:02X}   {l:02X}\n\n   {pc:04X}\n   {sp:04X}\n   {hl:04X}"),
                Point::new(5 * 8 * 2, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            if let Ok(data) = hl_data {
                Text::new(
                    &format!("\n\n\n\n\n\n\n         {data:02X}"),
                    Point::new(5 * 8 * 2, 6),
                    MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
                )
                .draw(&mut cpu_eg)
                .unwrap();
            }
            let mut flags = "          ".to_string();
            #[rustfmt::skip] if f & 0b1000_0000 != 0 { flags.push('Z') } else { flags.push(' ') };
            #[rustfmt::skip] if f & 0b0100_0000 != 0 { flags.push('N') } else { flags.push(' ') };
            #[rustfmt::skip] if f & 0b0010_0000 != 0 { flags.push('H') } else { flags.push(' ') };
            #[rustfmt::skip] if f & 0b0001_0000 != 0 { flags.push('C') } else { flags.push(' ') };
            Text::new(
                &flags,
                Point::new(5 * 8 * 2, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            Text::new(
                "Breakpoint\n\nPC\nLY",
                Point::new(5 * 8, 6 + 7 * 12),
                MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY),
            )
            .draw(&mut cpu_eg)
            .unwrap();
            if let Some(breakpoint) = pc_break {
                Text::new(
                    &format!("\n\n   {breakpoint:04X}"),
                    Point::new(5 * 8, 6 + 7 * 12),
                    MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
                )
                .draw(&mut cpu_eg)
                .unwrap();
            }
            if let Some(ly) = ly_break {
                Text::new(
                    &format!("\n\n\n   {ly:02X}"),
                    Point::new(5 * 8, 6 + 7 * 12),
                    MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
                )
                .draw(&mut cpu_eg)
                .unwrap();
            }
            windows
                .window_cpu
                .update_with_buffer(&cpu_eg.buffer[..], WINDOW_CPU_W, WINDOW_CPU_H)
                .unwrap();
        }
        // draw MEM
        #[cfg(feature = "mem")]
        {
            static BLACK: [u32; WINDOW_MEM_W * WINDOW_MEM_H] = [0; WINDOW_MEM_W * WINDOW_MEM_H];
            mem_eg.buffer[..].copy_from_slice(&BLACK[..]);
            let mut addr_buf = String::new();
            let mut data_buf = String::new();
            let mut zero_buf = String::new();
            let mut data = [0u8; 0x10];
            for addr in (0x0000..0x0100).step_by(0x10) {
                addr_buf.push_str(&format!("{addr:04X}\n"));
                gb.soc().read_exact(addr, &mut data[..]).unwrap();
                for (i, data) in data.iter().enumerate() {
                    if i % 8 == 0 {
                        zero_buf.push('|');
                        data_buf.push(' ');
                    } else {
                        zero_buf.push(' ');
                        data_buf.push(' ');
                    }
                    if *data == 0x00 {
                        zero_buf.push_str(&format!("{data:02X}"));
                        data_buf.push_str(&format!("  "));
                    } else {
                        zero_buf.push_str(&format!("  "));
                        data_buf.push_str(&format!("{data:02X}"));
                    }
                }
                zero_buf.push('\n');
                data_buf.push('\n');
            }
            Text::new(
                &addr_buf,
                Point::new(0, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY),
            )
            .draw(&mut mem_eg)
            .unwrap();
            Text::new(
                &zero_buf,
                Point::new(5 * 4, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::CSS_DIM_GRAY),
            )
            .draw(&mut mem_eg)
            .unwrap();
            Text::new(
                &data_buf,
                Point::new(5 * 4, 6),
                MonoTextStyle::new(&FONT_5X7, Rgb888::WHITE),
            )
            .draw(&mut mem_eg)
            .unwrap();
            windows
                .window_mem
                .update_with_buffer(&mem_eg.buffer[..], WINDOW_MEM_W, WINDOW_MEM_H)
                .unwrap();
        }
    }
}

fn handle_lcd_debug_overlay(window: &Window, flags: &mut LCDDebugOverlay) {
    if window.is_key_pressed(Key::Key0, KeyRepeat::No) {
        if *flags == LCDDebugOverlay::empty() {
            *flags = LCDDebugOverlay::all();
        } else {
            *flags = LCDDebugOverlay::empty();
        }
    }
    if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
        *flags ^= LCDDebugOverlay::TILEMAP;
    }
    if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
        *flags ^= LCDDebugOverlay::WINDOW;
    }
    if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
        *flags ^= LCDDebugOverlay::SPRITES;
    }
    if window.is_key_pressed(Key::Key4, KeyRepeat::No) {
        *flags ^= LCDDebugOverlay::LYC;
    }
}

fn handle_joypad_input(window: &Window, gb: &mut GameBoy) {
    fn handle_key(window: &Window, gb: &mut GameBoy, key: Key, button: Button) {
        if window.is_key_pressed(key, KeyRepeat::No) {
            gb.press(&button);
        }
        if window.is_key_released(key) {
            gb.release(&button);
        }
    }

    handle_key(&window, gb, Key::Z, Button::A);
    handle_key(&window, gb, Key::X, Button::B);
    handle_key(&window, gb, Key::Enter, Button::Start);
    handle_key(&window, gb, Key::RightShift, Button::Select);
    handle_key(&window, gb, Key::Left, Button::Left);
    handle_key(&window, gb, Key::Right, Button::Right);
    handle_key(&window, gb, Key::Up, Button::Up);
    handle_key(&window, gb, Key::Down, Button::Down);
}

fn load_cartridge(file: Box<[u8]>) -> Box<dyn Cartridge> {
    match file.get(0x147) {
        Some(0x00 | 0x08 | 0x09) => Box::new(ROM::new(file)) as _,
        Some(0x01 | 0x02 | 0x03) => Box::new(MBC1::new(file)) as _,
        Some(0x05 | 0x06) => Box::new(MBC2::new(file)) as _,
        Some(0x0f | 0x10 | 0x11 | 0x12 | 0x13) => Box::new(MBC3::new(file)) as _,
        Some(0x19 | 0x1a | 0x1b | 0x1c | 0x1d | 0x1e) => Box::new(MBC5::new(file)) as _,
        Some(0xfc) => Box::new(camera::PocketCamera::new(CameraSensor::new())) as _,
        _ => Box::new(()) as _,
    }
}

fn load_rom(
    path: Option<&str>,
    display: Rc<RefCell<[Color; WINDOW_LCD_W * WINDOW_LCD_H]>>,
    _: GameBoy,
) -> GameBoy {
    let cartridge = if let Some(path) = path {
        let file = std::fs::read(path).unwrap().into_boxed_slice();
        load_cartridge(file)
    } else {
        Box::new(()) as _
    };
    GameBoy::new(cartridge, GameBoyLCD(display))
}

fn make_emulator() -> (GameBoy, Rc<RefCell<[Color; WINDOW_LCD_W * WINDOW_LCD_H]>>) {
    let disp = Rc::new(RefCell::new([[0, 0, 0, 0xff]; WINDOW_LCD_W * WINDOW_LCD_H]));
    let lcd = GameBoyLCD(Rc::clone(&disp));
    let gameboy = GameBoy::new(Box::new(()) as _, lcd);
    (gameboy, disp)
}
