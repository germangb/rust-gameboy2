use core::{
    cartridge::{mbc1::MBC1, rom::Rom, Cartridge, NoCartridge},
    device::Device,
    Button, GameBoy,
};
use log::LevelFilter;
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use simple_logger::SimpleLogger;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn process_key(window: &Window, gb: &mut GameBoy<impl Cartridge>, key: Key, button: Button) {
    if window.is_key_pressed(key, KeyRepeat::No) {
        gb.press(&button);
    }
    if window.is_key_released(key) {
        gb.release(&button);
    }
}

fn main() {
    // SimpleLogger::new()
    //     .with_module_level("core", LevelFilter::Error)
    //     .with_module_level("core::ppu", LevelFilter::Trace)
    //     .init();
    pretty_env_logger::formatted_timed_builder()
        .filter(Some("core"), LevelFilter::Error)
        .filter(Some("core::ppu"), LevelFilter::Trace)
        .init();
    //json_env_logger::init();

    let mut opts = WindowOptions::default();
    opts.scale = Scale::X2;
    let mut window = Window::new("GameBoy", 160, 144, opts).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    //let cartridge = NoCartridge;
    let cartridge = MBC1::new(include_bytes!("tetris.gb").to_vec());
    let mut gb = GameBoy::new(cartridge);
    //gb.skip_boot();

    while window.is_open() {
        process_key(&window, &mut gb, Key::Z, Button::A);
        process_key(&window, &mut gb, Key::X, Button::B);
        process_key(&window, &mut gb, Key::Enter, Button::Start);
        process_key(&window, &mut gb, Key::RightShift, Button::Select);
        process_key(&window, &mut gb, Key::Left, Button::Left);
        process_key(&window, &mut gb, Key::Right, Button::Right);
        process_key(&window, &mut gb, Key::Up, Button::Up);
        process_key(&window, &mut gb, Key::Down, Button::Down);

        gb.update_frame();

        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            dump_tile_map(&gb);
        }

        if window.is_key_pressed(Key::D, KeyRepeat::No) {
            dump_tile_data(&gb);
        }

        window
            .update_with_buffer(&gb.display()[..], 160, 144)
            .unwrap();
    }
}

fn dump_tile_map(gb: &GameBoy<impl Cartridge>) {
    println!("TILE MAP");
    println!("===");
    println!("```");
    for (i, a) in (0x9800..=0x9bff).enumerate() {
        if i % 32 == 0 {
            print!("{:04x} | ", a);
        }
        let d = gb.read(a);
        if d == 0 {
            print!("__ ");
        } else {
            print!("{:02x} ", d);
        }
        if (i + 1) % 32 == 0 {
            println!();
        }
    }
    println!("```");
}

fn dump_tile_data(gb: &GameBoy<impl Cartridge>) {
    println!("TILE DATA");
    println!("===");
    println!("```");
    for (i, a) in (0x8000..=0x8fff).enumerate() {
        if i % 16 == 0 {
            print!("{:04x} | ", a);
        }
        print!("{:02x} ", gb.read(a));
        if (i + 1) % 0x10 == 0 {
            println!();
        }
    }
    println!("```");
}
