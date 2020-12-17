use camera::Sensor;

pub struct CameraSensor;

impl Sensor for CameraSensor {
    fn capture(&mut self, buffer: &mut [[u8; 128]; 112]) {
        for i in 0u8..112 {
            for j in 0u8..128 {
                buffer[i as usize][j as usize] = i ^ j;
            }
        }
    }
}
