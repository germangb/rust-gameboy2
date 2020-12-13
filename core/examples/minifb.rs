use core::{
    cartridge::{mbc1::MBC1, mbc3::MBC3, rom::ROM, Cartridge, NoCartridge},
    device::Device,
    Button, GameBoy,
};
use log::{info, LevelFilter};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use simple_logger::SimpleLogger;
use std::ops::{Range, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn handle_key(window: &Window, gb: &mut GameBoy<impl Cartridge>, key: Key, button: Button) {
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
        .filter(Some("core"), LevelFilter::Warn)
        .filter(Some("core::joypad"), LevelFilter::Off)
        .filter(Some("core::boot"), LevelFilter::Trace)
        .filter(Some("core::cpu"), LevelFilter::Warn)
        .filter(Some("core::irq"), LevelFilter::Trace)
        .filter(Some("core::cartridge"), LevelFilter::Warn)
        .filter(Some("core::ppu"), LevelFilter::Warn)
        .init();
    //json_env_logger::init();

    let mut opts = WindowOptions::default();
    opts.scale = Scale::X2;
    let mut window = Window::new("GameBoy", 160, 144, opts).unwrap();
    window.set_position(1980/2 - 160/2, 1080/2 - 144/4);

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    //let cartridge = NoCartridge;
    let cartridge = MBC3::new(include_bytes!("gold.gbc").to_vec());
    let mut gb = GameBoy::new(cartridge);
    //gb.skip_boot();

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

        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            dump_tile_map(&gb);
        }

        if window.is_key_pressed(Key::V, KeyRepeat::No) {
            dump(&gb, 0x8000..=0x9fff);
        }

        if window.is_key_pressed(Key::O, KeyRepeat::No) {
            dump(&gb, 0xfe00..=0xfe9f);
        }

        if window.is_key_pressed(Key::C, KeyRepeat::No) {
            println!("{:04x?}", gb.cpu().registers());
        }

        window
            .update_with_buffer(&gb.display()[..], 160, 144)
            .unwrap();
    }
}

fn dump(gb: &GameBoy<impl Cartridge>, range: RangeInclusive<u16>) {
    let mut line = String::new();
    for (i, a) in range.enumerate() {
        if i % 16 == 0 {
            line.clear();
            line.push_str(&format!("{:04x} | ", a));
        }
        line.push_str(&format!("{:02x} ", gb.read(a)));
        if (i + 1) % 16 == 0 {
            println!("{}", line);
        }
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
