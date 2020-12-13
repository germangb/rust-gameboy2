use core::{
    cartridge::{mbc3::MBC3, Cartridge, NoCartridge},
    Button, GameBoy,
};
use log::{info, warn, LevelFilter};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

type Error = Box<dyn std::error::Error>;

#[derive(StructOpt, Debug)]
struct App {
    /// ROM path.
    #[structopt(short, long)]
    path: Option<PathBuf>,

    /// Skip boot sequence.
    #[structopt(short, long)]
    skip_boot: bool,

    /// Scaling x2.
    #[structopt(short, long)]
    x2: bool,

    /// verbosity flag
    #[structopt(short, long)]
    verbose: bool,
}

fn read_rom(path: &Path) -> Result<Vec<u8>, Error> {
    Ok(std::fs::read(path)?)
}

fn run(app: &App, cartridge: impl Cartridge) {
    let mut opts = WindowOptions::default();
    if app.x2 {
        opts.scale = Scale::X2;
    }
    let mut window = Window::new("GameBoy", 160, 144, opts).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut gb = GameBoy::new(cartridge);

    if app.skip_boot {
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
            .filter(Some("core"), LevelFilter::Trace)
            .filter(Some("core::cpu"), LevelFilter::Warn)
            .init();
    }

    info!("App = {:?}", app);

    if let Some(path) = &app.path {
        let ron = read_rom(path)?;

        info!("ROM read: {} bytes", ron.len());

        run(&app, MBC3::new(ron));
    } else {
        warn!("No ROM selected");

        run(&app, NoCartridge);
    }

    Ok(())
}
