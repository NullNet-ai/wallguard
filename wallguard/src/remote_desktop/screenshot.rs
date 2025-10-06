use image::{ExtendedColorType, ImageEncoder, codecs::webp::WebPEncoder};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::io::Cursor;

#[derive(Default, Debug)]
pub struct Screenshot {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
}

impl Screenshot {
    pub fn new(buffer: Vec<u8>, width: usize, height: usize) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn compare(&self, other: &Screenshot) -> bool {
        if self.buffer.len() != other.buffer.len() {
            return false;
        }

        unsafe {
            libc::memcmp(
                self.buffer.as_ptr() as *const libc::c_void,
                other.buffer.as_ptr() as *const libc::c_void,
                self.buffer.len(),
            ) == 0
        }
    }

    pub fn as_webp(&self) -> Result<Vec<u8>, Error> {
        let mut data = Vec::new();
        let mut cursor = Cursor::new(&mut data);
        let encoder = WebPEncoder::new_lossless(&mut cursor);

        encoder
            .write_image(
                &self.buffer,
                self.width as u32,
                self.height as u32,
                ExtendedColorType::Rgba8,
            )
            .handle_err(location!())?;

        Ok(data)
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
