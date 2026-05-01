//! X11 screen capture using `x11rb`.
//!
//! Works on Linux and FreeBSD (any system running an X11 display server).
//! Uses `GetImage` (ZPixmap) without shared memory for broad compatibility;
//! the BGRA pixel layout that most X11 servers emit with 32-bit depth is
//! preserved as-is.

use anyhow::{Context as _, Result, bail};
use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{ConnectionExt as _, ImageFormat};
use x11rb::rust_connection::RustConnection;

use super::{CaptureBackend, Frame};

pub struct X11Capture {
    conn:   RustConnection,
    root:   u32,
    width:  u32,
    height: u32,
}

impl X11Capture {
    pub fn new() -> Result<Self> {
        let (conn, screen_num) = RustConnection::connect(None)
            .context("X11: cannot connect to display (is $DISPLAY set?)")?;

        let setup  = conn.setup();
        let screen = &setup.roots[screen_num];
        let root   = screen.root;
        let width  = screen.width_in_pixels  as u32;
        let height = screen.height_in_pixels as u32;

        if width == 0 || height == 0 {
            bail!("X11: root window has zero dimensions ({width}×{height})");
        }

        tracing::debug!(width, height, "X11 capture backend initialised");
        Ok(Self { conn, root, width, height })
    }
}

impl CaptureBackend for X11Capture {
    fn capture(&mut self) -> Result<Frame> {
        let reply = self.conn
            .get_image(
                ImageFormat::Z_PIXMAP,
                self.root,
                0, 0,
                self.width  as u16,
                self.height as u16,
                !0u32, // plane mask — all planes
            )
            .context("X11: GetImage request failed")?
            .reply()
            .context("X11: GetImage reply error")?;

        // X11 ZPixmap with 32bpp TrueColor is typically BGRX or BGRA in
        // memory on little-endian hosts.  We forward it directly as BGRA;
        // the encoder handles the alpha byte being ignored.
        let data = reply.data;
        let expected = (self.width * self.height * 4) as usize;
        if data.len() < expected {
            bail!(
                "X11: GetImage returned {} bytes, expected {} ({}×{}×4)",
                data.len(), expected, self.width, self.height
            );
        }

        Ok(Frame {
            width:  self.width,
            height: self.height,
            stride: self.width * 4,
            data,
        })
    }
}
