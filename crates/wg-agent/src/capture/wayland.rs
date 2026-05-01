//! Wayland screen capture via the XDG Desktop Portal.
//!
//! This backend is a compile-time stub for Phase A.  Full implementation
//! requires `ashpd` + PipeWire and an interactive portal permission dialog,
//! which is not appropriate for an unattended daemon.  For now the probe
//! always fails so the X11 backend (via XWayland or a native X11 session)
//! is used instead.
//!
//! TODO: implement using `ashpd 0.9` + `pipewire` crate when the portal
//! interaction can be pre-authorised (e.g., via a stored token).

use anyhow::{Result, bail};

use super::{CaptureBackend, Frame};

pub struct WaylandCapture;

impl WaylandCapture {
    pub fn new() -> Result<Self> {
        bail!(
            "Wayland capture via XDG portal is not yet implemented; \
             set DISPLAY and unset WAYLAND_DISPLAY to use X11 via XWayland"
        )
    }
}

impl CaptureBackend for WaylandCapture {
    fn capture(&mut self) -> Result<Frame> {
        bail!("WaylandCapture is a stub")
    }
}
