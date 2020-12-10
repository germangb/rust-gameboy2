use core::{cartridge::NoCartridge, GameBoy};
use minifb::{Key, Window, WindowOptions};
use simple_logger::SimpleLogger;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    //SimpleLogger::new().init().unwrap();

    let opts = WindowOptions::default();
    let mut window = Window::new("GameBoy", 160, 155, opts).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut gb = GameBoy::new(NoCartridge);

    while window.is_open() {
        gb.update();
        window
            .update_with_buffer(&gb.display()[..], 160, 144)
            .unwrap();
    }
}
