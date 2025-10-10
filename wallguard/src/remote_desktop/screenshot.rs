use openh264::formats::{RGB8Source, RGBSource};
use std::ops::Deref;

#[derive(Default, Debug, Clone)]
pub struct Screenshot {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
}

impl Screenshot {
    pub fn new(mut buffer: Vec<u8>, width: usize, height: usize) -> Self {
        let mut write_idx = 0;
        for read_idx in (0..buffer.len()).step_by(4) {
            /* R */
            buffer[write_idx] = buffer[read_idx];
            /* G */
            buffer[write_idx + 1] = buffer[read_idx + 1];
            /* B */
            buffer[write_idx + 2] = buffer[read_idx + 2];

            write_idx += 3;
        }

        buffer.truncate(width * height * 3);
        buffer.shrink_to_fit();

        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Deref for Screenshot {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl RGBSource for Screenshot {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn pixel_f32(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let idx = (y * self.width + x) * 3; // RGB stride (changed from 4)
        let r = self.buffer[idx] as f32 / 255.0;
        let g = self.buffer[idx + 1] as f32 / 255.0;
        let b = self.buffer[idx + 2] as f32 / 255.0;
        (r, g, b)
    }
}

impl RGB8Source for Screenshot {
    fn dimensions_padded(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn rgb8_data(&self) -> &[u8] {
        &self.buffer
    }
}
