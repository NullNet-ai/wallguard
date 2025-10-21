use super::screenshot::Screenshot;
use captis::{Capturer, init_capturer};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

pub struct ScreenCapturer {
    capturer: Box<dyn Capturer>,
}

impl ScreenCapturer {
    pub fn new() -> Result<Self, Error> {
        let capturer = init_capturer().handle_err(location!())?;

        Ok(Self {
            capturer: Box::new(capturer),
        })
    }

    pub fn screenshot(&mut self) -> Result<Screenshot, Error> {
        let buffer = self.capturer.capture(0).handle_err(location!())?;

        Ok(Screenshot::new(
            buffer.to_vec(),
            buffer.width() as usize,
            buffer.height() as usize,
        ))
    }
}
