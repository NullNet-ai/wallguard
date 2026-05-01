//! Keyboard and mouse input injection for the remote desktop tunnel.
//!
//! Call [`open_input_injector`] to get a platform backend.  Returns `Err`
//! when no display is reachable so the caller can forward video without
//! input rather than aborting the session.

use anyhow::Result;
use wg_shared::rdp::InputEvent;

mod enigo_injector;

pub trait InputInjector: Send {
    fn inject(&mut self, event: InputEvent) -> Result<()>;
}

/// Open the best available input injector for the current environment.
pub fn open_input_injector() -> Result<Box<dyn InputInjector>> {
    enigo_injector::EnigoInjector::new()
        .map(|i| Box::new(i) as Box<dyn InputInjector>)
}
