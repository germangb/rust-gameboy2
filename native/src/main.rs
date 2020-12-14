use core::{
    cartridge::{mbc3::MBC3, rom::ROM, Cartridge, NoCartridge, MBC1},
    Button, GameBoy,
};
use log::{error, info, warn, LevelFilter};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

type Error = Box<dyn std::error::Error>;

#[derive(StructOpt, Debug)]
struct App {
    #[structopt(short, long)]
    path: Option<PathBuf>,

    #[structopt(long)]
    skip_boot: bool,

    #[structopt(short, long, default_value = "2")]
    scale: u8,

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
    let mut window = Window::new(&app.window_title(), 160, 144, opts).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut gb = GameBoy::new(cartridge);

    if app.skip_boot {
        info!("Skip BOOT sequence.");

        gb.skip_boot();
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

        gb.update_frame();

        if window.is_key_pressed(Key::D, KeyRepeat::No) {
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
            .filter(Some("core"), LevelFilter::Off)
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
            0x01 | 0x02 | 0x03 => {
                info!("Cartridge type: MBC1 ({:02X}h)", kind);

                run(&app, MBC1::new(data))
            }
            0x0f | 0x10 | 0x11 | 0x12 | 0x13 => {
                info!("Cartridge type: MBC3 ({:02X}h)", kind);

                run(&app, MBC3::new(data))
            }
            _ => {
                error!("Unknown cartridge type: {:02X}h", kind);
                panic!();
            }
        }
    } else {
        run(&app, NoCartridge);
    }

    Ok(())
}
