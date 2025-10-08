use super::screenshot::Screenshot;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use scrap::{Capturer, Display};

pub struct ScreenCapturer {
    capturer: Capturer,
}

impl ScreenCapturer {
    pub fn new() -> Result<Self, Error> {
        let display = Display::primary().handle_err(location!())?;
        let capturer = Capturer::new(display).handle_err(location!())?;

        Ok(Self { capturer })
    }

    pub fn screenshot(&mut self) -> Result<Screenshot, Error> {
        let frame = self.capturer.frame().handle_err(location!())?;

        let mut buffer = Vec::default();

        buffer.extend_from_slice(&frame);
        
        // BGRA to RGBA
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Ok(Screenshot::new(buffer, self.width(), self.height()))
    }

    pub fn width(&self) -> usize {
        self.capturer.width()
    }

    pub fn height(&self) -> usize {
        self.capturer.height()
    }
}
