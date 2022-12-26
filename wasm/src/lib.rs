use core::cartridge::{NoCartridge, MBC3, MBC5};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

// re-exports
pub use core::Button;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct GameBoy {
    inner: core::GameBoy<MBC5>,
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once()
}

#[wasm_bindgen]
impl GameBoy {
    pub fn new() -> Self {
        let inner = core::GameBoy::new(MBC5::new(include_bytes!("tetris.gb").to_vec())).unwrap();
        Self { inner }
    }

    pub fn set_debug_overlays(&mut self, b: bool) {
        self.inner.set_debug_overlays(b)
    }

    pub fn update_frame(&mut self, ctx: &CanvasRenderingContext2d) {
        self.inner.update_frame().unwrap();
        let clamped = wasm_bindgen::Clamped(unsafe {
            std::slice::from_raw_parts(self.inner.display().as_ptr() as *const u8, 144 * 160 * 4)
        });
        let image_data = web_sys::ImageData::new_with_u8_clamped_array(clamped, 160)
            .expect("Error creating image data");
        ctx.put_image_data(&image_data, 0.0, 0.0)
            .expect("Error writing image to canvas");
    }

    pub fn press(&mut self, button: Button) {
        self.inner.press(&button)
    }

    pub fn release(&mut self, button: Button) {
        self.inner.release(&button)
    }
}
