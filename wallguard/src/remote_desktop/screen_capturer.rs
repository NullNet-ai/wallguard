use super::screenshot::Screenshot;
use nullnet_liberror::Error;

pub struct ScreenCapturer {
    inner: Box<dyn PlatformCapturer + Send>,
}

impl ScreenCapturer {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: create_capturer()?,
        })
    }

    pub fn screenshot(&mut self) -> Result<Screenshot, Error> {
        self.inner.capture()
    }
}

trait PlatformCapturer {
    fn capture(&mut self) -> Result<Screenshot, Error>;
}

#[cfg(target_os = "linux")]
fn create_capturer() -> Result<Box<dyn PlatformCapturer + Send>, Error> {
    Ok(Box::new(linux::X11Capturer::new()?))
}

#[cfg(target_os = "windows")]
fn create_capturer() -> Result<Box<dyn PlatformCapturer + Send>, Error> {
    Ok(Box::new(windows_backend::GdiCapturer::new()?))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn create_capturer() -> Result<Box<dyn PlatformCapturer + Send>, Error> {
    Err("Screen capture is not supported on this platform").handle_err(location!())
}

// ── Linux / X11 ──────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod linux {
    use super::{PlatformCapturer, Screenshot};
    use nullnet_liberror::{Error, ErrorHandler, Location, location};
    use x11rb::{
        connection::Connection,
        protocol::xproto::{ConnectionExt, ImageFormat},
        rust_connection::RustConnection,
    };

    pub struct X11Capturer {
        conn: RustConnection,
        root: u32,
        width: u16,
        height: u16,
        bits_per_pixel: u8,
    }

    impl X11Capturer {
        pub fn new() -> Result<Self, Error> {
            let (conn, screen_num) = RustConnection::connect(None).handle_err(location!())?;

            // Borrow conn inside a block so the borrow ends before conn is moved into Self.
            let (root, width, height, bits_per_pixel) = {
                let setup = conn.setup();
                let screen = &setup.roots[screen_num];
                let bpp = setup
                    .pixmap_formats
                    .iter()
                    .find(|f| f.depth == screen.root_depth)
                    .map(|f| f.bits_per_pixel)
                    .unwrap_or(32);
                (
                    screen.root,
                    screen.width_in_pixels,
                    screen.height_in_pixels,
                    bpp,
                )
            };

            Ok(Self {
                root,
                width,
                height,
                bits_per_pixel,
                conn,
            })
        }
    }

    impl PlatformCapturer for X11Capturer {
        fn capture(&mut self) -> Result<Screenshot, Error> {
            let reply = self
                .conn
                .get_image(
                    ImageFormat::Z_PIXMAP,
                    self.root,
                    0,
                    0,
                    self.width,
                    self.height,
                    !0u32,
                )
                .handle_err(location!())?
                .reply()
                .handle_err(location!())?;

            let rgb = raw_to_rgb(&reply.data, self.bits_per_pixel);

            Ok(Screenshot::new(
                rgb,
                self.width as usize,
                self.height as usize,
            ))
        }
    }

    fn raw_to_rgb(data: &[u8], bits_per_pixel: u8) -> Vec<u8> {
        match bits_per_pixel {
            32 => {
                // BGRX: blue=byte[0], green=byte[1], red=byte[2], unused=byte[3]
                let mut rgb = Vec::with_capacity(data.len() / 4 * 3);
                for chunk in data.chunks_exact(4) {
                    rgb.push(chunk[2]); // R
                    rgb.push(chunk[1]); // G
                    rgb.push(chunk[0]); // B
                }
                rgb
            }
            24 => {
                // BGR: blue=byte[0], green=byte[1], red=byte[2]
                let mut rgb = Vec::with_capacity(data.len());
                for chunk in data.chunks_exact(3) {
                    rgb.push(chunk[2]); // R
                    rgb.push(chunk[1]); // G
                    rgb.push(chunk[0]); // B
                }
                rgb
            }
            _ => {
                // Fallback: assume 32bpp BGRX
                let mut rgb = Vec::with_capacity(data.len() / 4 * 3);
                for chunk in data.chunks_exact(4) {
                    rgb.push(chunk[2]);
                    rgb.push(chunk[1]);
                    rgb.push(chunk[0]);
                }
                rgb
            }
        }
    }
}

// ── Windows / GDI ────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_backend {
    use super::{PlatformCapturer, Screenshot};
    use nullnet_liberror::{Error, ErrorHandler, Location, location};
    use std::mem;
    use winapi::shared::windef::{HBITMAP, HDC};
    use winapi::um::wingdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, SRCCOPY, SelectObject,
    };
    use winapi::um::winuser::{GetDC, GetSystemMetrics, ReleaseDC, SM_CXSCREEN, SM_CYSCREEN};

    pub struct GdiCapturer;

    impl GdiCapturer {
        pub fn new() -> Result<Self, Error> {
            Ok(Self)
        }
    }

    impl PlatformCapturer for GdiCapturer {
        fn capture(&mut self) -> Result<Screenshot, Error> {
            unsafe { capture_screen() }
        }
    }

    unsafe fn capture_screen() -> Result<Screenshot, Error> {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        let screen_dc: HDC = GetDC(std::ptr::null_mut());
        if screen_dc.is_null() {
            return Err("GetDC failed").handle_err(location!());
        }

        let compat_dc: HDC = CreateCompatibleDC(screen_dc);
        if compat_dc.is_null() {
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            return Err("CreateCompatibleDC failed").handle_err(location!());
        }

        let bitmap: HBITMAP = CreateCompatibleBitmap(screen_dc, width, height);
        if bitmap.is_null() {
            DeleteDC(compat_dc);
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            return Err("CreateCompatibleBitmap failed").handle_err(location!());
        }

        let old_obj = SelectObject(compat_dc, bitmap as _);

        if BitBlt(compat_dc, 0, 0, width, height, screen_dc, 0, 0, SRCCOPY) == 0 {
            SelectObject(compat_dc, old_obj);
            DeleteObject(bitmap as _);
            DeleteDC(compat_dc);
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            return Err("BitBlt failed").handle_err(location!());
        }

        let mut bmi: BITMAPINFO = mem::zeroed();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // negative = top-down DIB
            biPlanes: 1,
            biBitCount: 24,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        // GDI rows are padded to 32-bit boundaries; 24bpp = 3 bytes/pixel
        let stride = ((width * 3 + 3) & !3) as usize;
        let mut raw = vec![0u8; stride * height as usize];

        let scan_lines = GetDIBits(
            compat_dc,
            bitmap,
            0,
            height as u32,
            raw.as_mut_ptr() as _,
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(compat_dc, old_obj);
        DeleteObject(bitmap as _);
        DeleteDC(compat_dc);
        ReleaseDC(std::ptr::null_mut(), screen_dc);

        if scan_lines == 0 {
            return Err("GetDIBits failed").handle_err(location!());
        }

        let w = width as usize;
        let h = height as usize;
        let mut rgb = Vec::with_capacity(w * h * 3);

        // GDI returns BGR; convert to RGB row-by-row, skipping row padding
        for row in 0..h {
            let row_start = row * stride;
            for col in 0..w {
                let px = row_start + col * 3;
                rgb.push(raw[px + 2]); // R
                rgb.push(raw[px + 1]); // G
                rgb.push(raw[px]); // B
            }
        }

        Ok(Screenshot::new(rgb, w, h))
    }
}
