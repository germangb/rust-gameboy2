use camera::Sensor;
use image::GrayImage;

pub struct CameraSensor {
    image: GrayImage,
}

impl CameraSensor {
    pub fn new() -> Self {
        let image = image::load_from_memory(include_bytes!("image.png"))
            .unwrap()
            .to_luma8();
        Self { image }
    }
}

impl Sensor for CameraSensor {
    fn capture(&mut self, buffer: &mut [[u8; 128]; 112]) {
        for i in 0..112 {
            for j in 0..128 {
                buffer[i as usize][j as usize] = self.image.get_pixel(j, i).0[0];
            }
        }
    }
}
