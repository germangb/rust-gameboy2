use crate::camera_sensor::CameraSensor;
use camera::PoketCamera;
use core::{
    cartridge::{Cartridge, NoCartridge, MBC1, MBC3, MBC5, ROM},
    Button, GameBoy,
};
use log::{error, info, warn, LevelFilter};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[cfg(feature = "camera")]
mod camera_sensor;

type Error = Box<dyn std::error::Error>;

#[derive(StructOpt, Debug)]
struct App {
    #[structopt(short, long)]
    path: Option<PathBuf>,

    /// Force MBC1 memory controller.
    #[structopt(long)]
    mbc1: bool,

    /// Force MBC3 memory controller.
    #[structopt(long)]
    mbc3: bool,

    /// Force single ROM (+ RAM) memory controller.
    #[structopt(long)]
    rom: bool,

    #[structopt(long)]
    skip_boot: bool,

    #[structopt(short, long, default_value = "2")]
    scale: u8,

    #[structopt(long)]
    borderless: bool,

    #[structopt(short, long)]
    verbose: bool,
}

impl App {
    fn window_title(&self) -> String {
        let mut title = "GameBoy".to_string();
        if let Some(path) = &self.path {
            title.push_str(&format!(" ({})", path.display()))
        }
        title
    }

    fn window_scale(&self) -> Scale {
        match self.scale {
            1 => Scale::X1,
            4 => Scale::X4,
            8 => Scale::X8,
            scale => {
                if scale != 2 {
                    warn!("Unsupported display scaling (x{}). Fallback to x2.", scale)
                }
                Scale::X2
            }
        }
    }
}

fn read_rom(path: &Path) -> Result<Vec<u8>, Error> {
    Ok(std::fs::read(path)?)
}

fn run(app: &App, cartridge: impl Cartridge) {
    let mut opts = WindowOptions::default();
    opts.scale = app.window_scale();
    opts.borderless = app.borderless;
    let mut window = Window::new(&app.window_title(), 160, 144, opts).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut gb = GameBoy::new(cartridge).unwrap();
    let mut debug = false;

    if app.skip_boot {
        info!("Skip BOOT sequence.");

        gb.skip_boot().unwrap();
    }

    while window.is_open() {
        handle_key(&window, &mut gb, Key::Z, Button::A);
        handle_key(&window, &mut gb, Key::X, Button::B);
        handle_key(&window, &mut gb, Key::Enter, Button::Start);
        handle_key(&window, &mut gb, Key::RightShift, Button::Select);
        handle_key(&window, &mut gb, Key::Left, Button::Left);
        handle_key(&window, &mut gb, Key::Right, Button::Right);
        handle_key(&window, &mut gb, Key::Up, Button::Up);
        handle_key(&window, &mut gb, Key::Down, Button::Down);

        gb.update_frame().unwrap();

        if window.is_key_pressed(Key::C, KeyRepeat::No) {
            info!("{:02x?}", gb.cpu().registers());
        }

        if window.is_key_pressed(Key::D, KeyRepeat::No) {
            debug = !debug;
            gb.set_debug_overlays(debug);
        }

        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            info!("Generating display.bin");

            unsafe {
                std::fs::write(
                    "display.bin",
                    std::mem::transmute::<_, [u8; 160 * 144 * 4]>(gb.display().clone()),
                )
                .unwrap();
            }
        }

        window
            .update_with_buffer(&gb.display()[..], 160, 144)
            .unwrap();
    }
}

fn handle_key(window: &Window, gb: &mut GameBoy<impl Cartridge>, key: Key, button: Button) {
    if window.is_key_pressed(key, KeyRepeat::No) {
        gb.press(&button);
    }
    if window.is_key_released(key) {
        gb.release(&button);
    }
}

fn main() -> Result<(), Error> {
    let app = App::from_args();

    if app.verbose {
        pretty_env_logger::formatted_timed_builder()
            //.filter(Some("core"), LevelFilter::Trace)
            //.filter(Some("core::cpu"), LevelFilter::Off)
            //.filter(Some("core::cartridge"), LevelFilter::Off)
            .filter(Some("core::ppu"), LevelFilter::Warn)
            .filter(Some(module_path!()), LevelFilter::Trace)
            .init();
    }

    if let Some(path) = &app.path {
        let data = read_rom(path)?;

        let kind = data[0x147];
        match kind {
            0x00 | 0x08 | 0x09 => {
                info!("Cartridge type: ROM ({:02X}h)", kind);

                run(&app, ROM::new(data))
            }
            0x06 | 0x01 | 0x02 | 0x03 => {
                info!("Cartridge type: MBC1 ({:02X}h)", kind);

                run(&app, MBC1::new(data))
            }
            0x1b | 0x0f | 0x10 | 0x11 | 0x12 | 0x13 => {
                info!("Cartridge type: MBC3 ({:02X}h)", kind);

                run(&app, MBC3::new(data))
            }
            0x19 | 0x1a | 0x1b | 0x1c | 0x1d | 0x1e => {
                info!("Cartridge type: MBC5 ({:02X}h)", kind);

                run(&app, MBC5::new(data))
            }
            #[cfg(feature = "camera")]
            0xfc => {
                info!("Cartridge type: PoketCamera ({:02X}h)", kind);

                let sensor = CameraSensor::new();

                run(&app, PoketCamera::new(sensor));
            }
            _ => {
                panic!("Unknown cartridge type: {:02X}h", kind);
            }
        }
    } else {
        run(&app, NoCartridge);
    }

    Ok(())
}
