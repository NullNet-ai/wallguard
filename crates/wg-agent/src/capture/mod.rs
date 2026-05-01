//! Screen capture backends for remote desktop.
//!
//! Call [`open_capture_backend`] to get a platform-appropriate backend.
//! Returns `Err` on headless systems so the caller can omit
//! `Feature::RemoteDesktop` from the capability advertisement.

use anyhow::Result;

#[cfg(unix)]
mod x11;
#[cfg(target_os = "linux")]
mod wayland;
#[cfg(target_os = "windows")]
mod windows_dxgi;

/// A single captured screen frame in BGRA format, row-major.
pub struct Frame {
    pub width:  u32,
    pub height: u32,
    /// Always `width * 4`; kept explicit for future SHM paths with row padding.
    pub stride: u32,
    pub data:   Vec<u8>,
}

/// Synchronous screen capture trait.  Each call to [`capture`] returns the
/// latest full-screen BGRA frame.  Implementations must be `Send` so the
/// caller can move them into a dedicated capture thread.
pub trait CaptureBackend: Send {
    fn capture(&mut self) -> Result<Frame>;
}

/// Open the best available capture backend for the current environment.
///
/// Returns `Err` if no display server is reachable (headless appliance).
pub fn open_capture_backend() -> Result<Box<dyn CaptureBackend>> {
    #[cfg(target_os = "windows")]
    {
        windows_dxgi::DxgiCapture::new().map(|b| Box::new(b) as Box<dyn CaptureBackend>)
    }

    #[cfg(target_os = "linux")]
    {
        // Try Wayland portal first; if the session isn't Wayland, fall through.
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            match wayland::WaylandCapture::new() {
                Ok(b) => return Ok(Box::new(b) as Box<dyn CaptureBackend>),
                Err(e) => tracing::debug!("Wayland capture unavailable ({e}), falling back to X11"),
            }
        }
        x11::X11Capture::new().map(|b| Box::new(b) as Box<dyn CaptureBackend>)
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    {
        // FreeBSD, macOS, etc. — X11 only.
        x11::X11Capture::new().map(|b| Box::new(b) as Box<dyn CaptureBackend>)
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        anyhow::bail!("remote desktop: no capture backend available on this platform")
    }
}
