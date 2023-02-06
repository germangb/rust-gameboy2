use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlVideoElement};

#[wasm_bindgen]
pub struct CameraSensor {
    ctx: CanvasRenderingContext2d,
    video: Option<HtmlVideoElement>,
}

#[wasm_bindgen]
impl CameraSensor {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Self { ctx, video: None }
    }

    pub fn with_video(ctx: CanvasRenderingContext2d, video: HtmlVideoElement) -> Self {
        Self {
            ctx,
            video: Some(video),
        }
    }
}

impl camera::Sensor for CameraSensor {
    fn capture(&mut self, buffer: &mut [[u8; 128]; 112]) {
        if let Some(video) = &self.video {
            self.ctx
                .draw_image_with_html_video_element_and_dw_and_dh(video, 0.0, 0.0, 128.0, 112.0)
                .expect("Error writing video to canvas");
        }
        let image_data = self
            .ctx
            .get_image_data(0.0, 0.0, 128.0, 112.0)
            .expect("Error getting image data");
        let data = image_data.data().0;
        for (i, row) in buffer.iter_mut().enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                let offset = (128 * i + j) * 4;
                let r = data[offset] as f64 / 255.0;
                let g = data[offset + 1] as f64 / 255.0;
                let b = data[offset + 2] as f64 / 255.0;
                let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                *col = (y * 255.0).min(255.0) as u8;
            }
        }
    }
}
