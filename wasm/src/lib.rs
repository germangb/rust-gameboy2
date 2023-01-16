pub use core::joypad::Button;
use core::ppu::{Color, LCD, LCD_HEIGHT, LCD_WIDTH};
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{CanvasRenderingContext2d, ImageData};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn is_cgb() -> bool {
    #[cfg(not(feature = "cgb"))]
    return false;
    #[cfg(feature = "cgb")]
    return true;
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once()
}

#[wasm_bindgen]
pub fn wasm_log_init() {
    wasm_log::init(wasm_log::Config::default());
}

struct LCDBuffer(pub [Color; LCD_WIDTH * LCD_HEIGHT]);

impl LCD for LCDBuffer {
    fn output_line(&mut self, ly: u8, data: &[Color; LCD_WIDTH]) {
        let offset = (ly as usize) * LCD_WIDTH;
        self.0[offset..(offset + LCD_WIDTH)].copy_from_slice(data);
    }
}

fn load_cartridge(file: Box<[u8]>) -> Box<dyn core::cartridge::Cartridge> {
    match file.get(0x147) {
        Some(0x00 | 0x08 | 0x09) => Box::new(core::cartridge::ROM::new(file)) as _,
        Some(0x01 | 0x02 | 0x03) => Box::new(core::cartridge::MBC1::new(file)) as _,
        Some(0x05 | 0x06) => Box::new(core::cartridge::MBC2::new(file)) as _,
        Some(0x0f | 0x10 | 0x11 | 0x12 | 0x13) => Box::new(core::cartridge::MBC3::new(file)) as _,
        Some(0x19 | 0x1a | 0x1b | 0x1c | 0x1d | 0x1e) => {
            Box::new(core::cartridge::MBC5::new(file)) as _
        }
        _ => Box::new(()) as _,
    }
}

#[wasm_bindgen]
pub struct GameBoy {
    inner: core::gb::GameBoy<Box<dyn core::cartridge::Cartridge>, LCDBuffer>,
}

#[wasm_bindgen]
impl GameBoy {
    pub fn new() -> Self {
        let lcd_buffer = LCDBuffer([[0x00; 4]; LCD_WIDTH * LCD_HEIGHT]);
        let data = include_bytes!("cpu_instrs.gb").to_vec().into_boxed_slice();
        let cartridge = load_cartridge(data);
        let inner = core::gb::GameBoy::new(cartridge, lcd_buffer);
        Self { inner }
    }

    pub fn reset(&mut self) {
        let lcd_buffer = LCDBuffer([[0x00; 4]; LCD_WIDTH * LCD_HEIGHT]);
        let cartridge = Box::new(()) as _;
        let inner = core::gb::GameBoy::new(cartridge, lcd_buffer);
        let _ = std::mem::replace(&mut self.inner, inner);
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let lcd_buffer = LCDBuffer([[0x00; 4]; LCD_WIDTH * LCD_HEIGHT]);
        let cartridge = load_cartridge(data.to_vec().into_boxed_slice());
        let inner = core::gb::GameBoy::new(cartridge, lcd_buffer);
        let _ = std::mem::replace(&mut self.inner, inner);
    }

    pub fn set_lcd_overlay_flags(&mut self, flags: u8) {
        self.inner.soc_mut().ppu_mut().lcd_debug_overlay =
            core::ppu::LCDDebugOverlay::from_bits_truncate(flags);
    }

    pub fn update(
        &mut self,
        lcd: &CanvasRenderingContext2d,
        pal: &CanvasRenderingContext2d,
        vram0: &CanvasRenderingContext2d,
        vram1: &CanvasRenderingContext2d,
    ) {
        self.inner.next_frame().unwrap();

        let lcd_data = unsafe {
            std::slice::from_raw_parts(
                self.inner.soc().ppu().output().0.as_ptr() as *const u8,
                160 * 144 * 4,
            )
        };
        let tile_data = self.inner.soc().vram().tile_data_cache().as_slice();
        let vram0_data = unsafe {
            std::slice::from_raw_parts(
                tile_data[..24 * 16 * 8 * 8].as_ptr() as *const u8,
                192 * 128 * 4,
            )
        };
        #[cfg(feature = "cgb")]
        let vram1_data = unsafe {
            std::slice::from_raw_parts(
                tile_data[24 * 16 * 8 * 8..].as_ptr() as *const u8,
                192 * 128 * 4,
            )
        };

        update_canvas_image(lcd, lcd_data, 160);
        update_canvas_image(vram0, vram0_data, 192);
        #[cfg(feature = "cgb")]
        update_canvas_image(vram1, vram1_data, 192);

        self.update_pal(pal);
    }

    #[cfg(not(feature = "cgb"))]
    fn update_pal(&mut self, pal: &CanvasRenderingContext2d) {
        let obp0 = self.inner.soc().ppu().obp0();
        let obp1 = self.inner.soc().ppu().obp1();
        let bgp = self.inner.soc().ppu().bgp();

        // obp0
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp0[0][0], obp0[0][1], obp0[0][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 0.0, 16.0, 16.0);
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp0[1][0], obp0[1][1], obp0[1][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 16.0, 16.0, 16.0);
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp0[2][0], obp0[2][1], obp0[2][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 32.0, 16.0, 16.0);
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp0[3][0], obp0[3][1], obp0[3][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 48.0, 16.0, 16.0);

        // obp1
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp1[0][0], obp1[0][1], obp1[0][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(16.0, 0.0, 16.0, 16.0);
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp1[1][0], obp1[1][1], obp1[1][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(16.0, 16.0, 16.0, 16.0);
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp1[2][0], obp1[2][1], obp1[2][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(16.0, 32.0, 16.0, 16.0);
        let fill_style = JsValue::from_str(&format!(
            "rgb({},{},{})",
            obp1[3][0], obp1[3][1], obp1[3][2]
        ));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(16.0, 48.0, 16.0, 16.0);

        // bgp
        let fill_style =
            JsValue::from_str(&format!("rgb({},{},{})", bgp[0][0], bgp[0][1], bgp[0][2]));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 64.0, 16.0, 16.0);
        let fill_style =
            JsValue::from_str(&format!("rgb({},{},{})", bgp[1][0], bgp[1][1], bgp[1][2]));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 64.0 + 16.0, 16.0, 16.0);
        let fill_style =
            JsValue::from_str(&format!("rgb({},{},{})", bgp[2][0], bgp[2][1], bgp[2][2]));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 64.0 + 32.0, 16.0, 16.0);
        let fill_style =
            JsValue::from_str(&format!("rgb({},{},{})", bgp[3][0], bgp[3][1], bgp[3][2]));
        pal.set_fill_style(&fill_style);
        pal.fill_rect(0.0, 64.0 + 48.0, 16.0, 16.0);
    }

    #[cfg(feature = "cgb")]
    fn update_pal(&mut self, pal: &CanvasRenderingContext2d) {
        let bgp = self.inner.soc().ppu().bgp();
        let obp = self.inner.soc().ppu().obp();
        for i in 0..4 {
            let off_x = (i as f64) * 8.0;
            let off_y = 0.0;
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[i][0][0], obp[i][0][1], obp[i][0][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 0.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[i][1][0], obp[i][1][1], obp[i][1][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 8.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[i][2][0], obp[i][2][1], obp[i][2][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 16.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[i][3][0], obp[i][3][1], obp[i][3][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 24.0, 8.0, 8.0);
            let off_y = 32.0;
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[4 + i][0][0],
                obp[4 + i][0][1],
                obp[4 + i][0][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 0.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[4 + i][1][0],
                obp[4 + i][1][1],
                obp[4 + i][1][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 8.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[4 + i][2][0],
                obp[4 + i][2][1],
                obp[4 + i][2][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 16.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                obp[4 + i][3][0],
                obp[4 + i][3][1],
                obp[4 + i][3][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 24.0, 8.0, 8.0);
        }

        for i in 0..4 {
            let off_x = (i as f64) * 8.0;
            let off_y = 64.0;
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[i][0][0], bgp[i][0][1], bgp[i][0][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 0.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[i][1][0], bgp[i][1][1], bgp[i][1][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 8.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[i][2][0], bgp[i][2][1], bgp[i][2][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 16.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[i][3][0], bgp[i][3][1], bgp[i][3][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 24.0, 8.0, 8.0);
            let off_y = 64.0 + 32.0;
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[4 + i][0][0],
                bgp[4 + i][0][1],
                bgp[4 + i][0][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 0.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[4 + i][1][0],
                bgp[4 + i][1][1],
                bgp[4 + i][1][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 8.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[4 + i][2][0],
                bgp[4 + i][2][1],
                bgp[4 + i][2][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 16.0, 8.0, 8.0);
            let fill_style = JsValue::from_str(&format!(
                "rgb({},{},{})",
                bgp[4 + i][3][0],
                bgp[4 + i][3][1],
                bgp[4 + i][3][2]
            ));
            pal.set_fill_style(&fill_style);
            pal.fill_rect(off_x, off_y + 24.0, 8.0, 8.0);
        }
    }

    pub fn press(&mut self, button: Button) {
        self.inner.press(&button)
    }

    pub fn release(&mut self, button: Button) {
        self.inner.release(&button)
    }
}

fn update_canvas_image(canvas: &CanvasRenderingContext2d, image_data: &[u8], width: u32) {
    let clamped = Clamped(image_data);
    let image_data = ImageData::new_with_u8_clamped_array(clamped, width).unwrap();
    canvas.put_image_data(&image_data, 0.0, 0.0).unwrap();
}
