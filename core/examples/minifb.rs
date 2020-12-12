use core::{
    cartridge::{Cartridge, NoCartridge, Rom},
    device::Device,
    GameBoy,
};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use simple_logger::SimpleLogger;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    SimpleLogger::new().init().unwrap();
    //pretty_env_logger::init();
    //json_env_logger::init();

    let mut opts = WindowOptions::default();
    let mut window = Window::new("GameBoy", 160, 144, opts).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let cartridge = NoCartridge;
    let mut gb = GameBoy::new(cartridge);

    while window.is_open() {
        gb.update_frame();

        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            dump_tile_map(&gb);
        }

        if window.is_key_pressed(Key::D, KeyRepeat::No) {
            dump_tile_data(&gb);
        }

        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            println!("LCDC={:08b}", gb.read(0xff40));
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
