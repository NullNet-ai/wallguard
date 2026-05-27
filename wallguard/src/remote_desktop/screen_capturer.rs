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
    use crate::client_data::platform::has_wayland_display;

    // Prefer native Wayland capture when the compositor socket is reachable.
    // libwayshot uses the wlr-screencopy protocol — no portal, no D-Bus dialog.
    if has_wayland_display() {
        match wayland::WaylandCapturer::new() {
            Ok(c) => {
                log::info!("Screen capture: using Wayland (wlr-screencopy) backend");
                return Ok(Box::new(c));
            }
            Err(e) => log::debug!(
                "Wayland capturer unavailable ({}); falling back to X11",
                e.to_str()
            ),
        }
    }

    log::info!("Screen capture: using X11 backend");
    Ok(Box::new(x11::X11Capturer::new()?))
}

#[cfg(target_os = "freebsd")]
fn create_capturer() -> Result<Box<dyn PlatformCapturer + Send>, Error> {
    Ok(Box::new(x11::X11Capturer::new()?))
}

#[cfg(target_os = "windows")]
fn create_capturer() -> Result<Box<dyn PlatformCapturer + Send>, Error> {
    Ok(Box::new(windows_backend::GdiCapturer::new()?))
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "windows")))]
fn create_capturer() -> Result<Box<dyn PlatformCapturer + Send>, Error> {
    use nullnet_liberror::{ErrorHandler, Location, location};
    Err("Screen capture is not supported on this platform").handle_err(location!())
}

// ── X11 (Linux + FreeBSD) ────────────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod x11 {
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

// ── Wayland / wlr-screencopy (Linux) ─────────────────────────────────────────
//
// Implements `zwlr-screencopy-unstable-v1` directly with `wayland-client` +
// `wayland-protocols-wlr`.  Using the SHM (CPU) buffer path means no GPU
// stack (gbm / EGL) is needed — the compositor copies pixels into a shared
// memory file that we mmap and read.
//
// Supported compositors: sway, hyprland, labwc, river, wayfire, KDE Plasma 6.
// GNOME/mutter does not implement wlr-screencopy; those sessions fall through
// to X11 via XWayland automatically (see create_capturer).

#[cfg(target_os = "linux")]
mod wayland {
    use super::{PlatformCapturer, Screenshot};
    use nullnet_liberror::{Error, ErrorHandler, Location, location};
    use std::num::NonZeroUsize;
    use std::os::fd::AsFd;
    use wayland_client::{
        Connection, Dispatch, EventQueue, QueueHandle,
        protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool},
    };
    use wayland_protocols_wlr::screencopy::v1::client::{
        zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
        zwlr_screencopy_manager_v1::{self, ZwlrScreencopyManagerV1},
    };

    // ── Shared state threaded through the event dispatch ──────────────────────

    #[derive(Default)]
    struct Session {
        // Wayland globals — populated during initialisation roundtrips.
        shm: Option<wl_shm::WlShm>,
        output: Option<wl_output::WlOutput>,
        manager: Option<ZwlrScreencopyManagerV1>,

        // Per-frame values from zwlr_screencopy_frame_v1::Buffer event.
        width: u32,
        height: u32,
        stride: u32,
        format: u32, // raw wl_shm format value

        // Completion flags from Ready / Failed events.
        ready: bool,
        failed: bool,
    }

    // ── Dispatch implementations ───────────────────────────────────────────────

    impl Dispatch<wl_registry::WlRegistry, ()> for Session {
        fn event(
            state: &mut Self,
            registry: &wl_registry::WlRegistry,
            event: wl_registry::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            let wl_registry::Event::Global {
                name,
                interface,
                version,
            } = event
            else {
                return;
            };
            match interface.as_str() {
                "wl_shm" => {
                    state.shm = Some(registry.bind(name, 1, qh, ()));
                }
                "wl_output" if state.output.is_none() => {
                    state.output = Some(registry.bind(name, version.min(4), qh, ()));
                }
                "zwlr_screencopy_manager_v1" => {
                    state.manager = Some(registry.bind(name, version.min(3), qh, ()));
                }
                _ => {}
            }
        }
    }

    impl Dispatch<ZwlrScreencopyFrameV1, ()> for Session {
        fn event(
            state: &mut Self,
            _: &ZwlrScreencopyFrameV1,
            event: zwlr_screencopy_frame_v1::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            match event {
                zwlr_screencopy_frame_v1::Event::Buffer {
                    format,
                    width,
                    height,
                    stride,
                } => {
                    state.format = match format {
                        wayland_client::WEnum::Value(f) => f as u32,
                        wayland_client::WEnum::Unknown(n) => n,
                    };
                    state.width = width;
                    state.height = height;
                    state.stride = stride;
                }
                zwlr_screencopy_frame_v1::Event::Ready { .. } => state.ready = true,
                zwlr_screencopy_frame_v1::Event::Failed => state.failed = true,
                _ => {}
            }
        }
    }

    // No-op dispatches for objects whose events we don't need.
    macro_rules! noop_dispatch {
        ($iface:ty, $ev:ty) => {
            impl Dispatch<$iface, ()> for Session {
                fn event(
                    _: &mut Self,
                    _: &$iface,
                    _: $ev,
                    _: &(),
                    _: &Connection,
                    _: &QueueHandle<Self>,
                ) {
                }
            }
        };
    }
    noop_dispatch!(wl_shm::WlShm, wl_shm::Event);
    noop_dispatch!(wl_shm_pool::WlShmPool, wl_shm_pool::Event);
    noop_dispatch!(wl_buffer::WlBuffer, wl_buffer::Event);
    noop_dispatch!(wl_output::WlOutput, wl_output::Event);
    noop_dispatch!(ZwlrScreencopyManagerV1, zwlr_screencopy_manager_v1::Event);

    // ── WaylandCapturer ───────────────────────────────────────────────────────

    pub struct WaylandCapturer {
        event_queue: EventQueue<Session>,
        qh: QueueHandle<Session>,
        session: Session,
    }

    impl WaylandCapturer {
        pub fn new() -> Result<Self, Error> {
            let conn = Connection::connect_to_env().handle_err(location!())?;
            let display = conn.display();
            let mut event_queue = conn.new_event_queue::<Session>();
            let qh = event_queue.handle();

            let mut session = Session::default();
            let _registry = display.get_registry(&qh, ());

            // Two roundtrips: first populates the global list, second lets the
            // compositor process any pending acks.
            event_queue
                .roundtrip(&mut session)
                .handle_err(location!())?;
            event_queue
                .roundtrip(&mut session)
                .handle_err(location!())?;

            if session.shm.is_none() || session.output.is_none() || session.manager.is_none() {
                return Err("compositor missing required globals \
                     (wl_shm, wl_output, or zwlr_screencopy_manager_v1) — \
                     wlr-screencopy may not be supported")
                .handle_err(location!());
            }

            Ok(Self {
                event_queue,
                qh,
                session,
            })
        }
    }

    impl PlatformCapturer for WaylandCapturer {
        fn capture(&mut self) -> Result<Screenshot, Error> {
            // Clone Wayland proxies upfront so we don't hold borrows into
            // `self.session` while also passing `&mut self.session` to roundtrip.
            let output = self.session.output.as_ref().unwrap().clone();
            let manager = self.session.manager.as_ref().unwrap().clone();
            let shm = self.session.shm.as_ref().unwrap().clone();

            // Reset per-frame state.
            self.session.width = 0;
            self.session.height = 0;
            self.session.stride = 0;
            self.session.ready = false;
            self.session.failed = false;

            // Ask the compositor for a screencopy frame (cursor not included).
            let frame = manager.capture_output(0, &output, &self.qh, ());

            // Roundtrip to receive the Buffer event (gives us dimensions + format).
            self.event_queue
                .roundtrip(&mut self.session)
                .handle_err(location!())?;

            if self.session.width == 0 {
                return Err("no Buffer event received from compositor").handle_err(location!());
            }

            let (w, h, stride) = (self.session.width, self.session.height, self.session.stride);
            let size = (stride * h) as usize;

            // Allocate an anonymous shared-memory file for the pixel data.
            let shm_fd = create_shm_fd(size)?;

            // Map it into our address space so we can read the pixels after copy.
            let map_ptr = unsafe {
                nix::sys::mman::mmap(
                    None,
                    NonZeroUsize::new(size)
                        .ok_or("zero-size frame")
                        .handle_err(location!())?,
                    nix::sys::mman::ProtFlags::PROT_READ | nix::sys::mman::ProtFlags::PROT_WRITE,
                    nix::sys::mman::MapFlags::MAP_SHARED,
                    shm_fd.as_fd(),
                    0,
                )
                .handle_err(location!())?
            };

            // Create the Wayland SHM pool + buffer objects.
            let pool = shm.create_pool(shm_fd.as_fd(), size as i32, &self.qh, ());
            let buf_fmt =
                wl_shm::Format::try_from(self.session.format).unwrap_or(wl_shm::Format::Xrgb8888);
            let buffer =
                pool.create_buffer(0, w as i32, h as i32, stride as i32, buf_fmt, &self.qh, ());

            // Trigger the copy; dispatch until Ready or Failed.
            frame.copy(&buffer);
            while !self.session.ready && !self.session.failed {
                self.event_queue
                    .roundtrip(&mut self.session)
                    .handle_err(location!())?;
            }

            let result = if self.session.failed {
                Err("Wayland screencopy frame failed").handle_err(location!())
            } else {
                let raw =
                    unsafe { std::slice::from_raw_parts(map_ptr.as_ptr() as *const u8, size) };
                let rgb = bgrx_to_rgb(raw, stride, w, h);
                Ok(Screenshot::new(rgb, w as usize, h as usize))
            };

            // Cleanup — always runs whether capture succeeded or not.
            unsafe {
                let _ = nix::sys::mman::munmap(map_ptr, size);
            }
            buffer.destroy();
            pool.destroy();
            frame.destroy();

            result
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Creates an anonymous file of `size` bytes suitable for a `WlShmPool`.
    ///
    /// Uses `shm_open` + `shm_unlink` (glibc 2.2+) instead of `memfd_create`
    /// (glibc 2.27+) so the binary stays compatible with the glibc 2.17 target.
    /// The name is unlinked immediately after opening, so no entry persists in
    /// `/dev/shm` and the fd behaves as an anonymous shared-memory object.
    fn create_shm_fd(size: usize) -> Result<std::os::fd::OwnedFd, Error> {
        use nix::fcntl::OFlag;
        use nix::sys::mman::{shm_open, shm_unlink};
        use nix::sys::stat::Mode;
        use std::ffi::CString;
        use std::sync::atomic::{AtomicU32, Ordering};

        // PID + monotonic counter → unique name even if two captures race.
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = CString::new(format!("/wallguard-{}-{n}", std::process::id())).unwrap();

        let fd = shm_open(
            name.as_c_str(),
            OFlag::O_CREAT | OFlag::O_RDWR | OFlag::O_EXCL,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )
        .handle_err(location!())?;

        // Remove the name immediately — fd stays valid, /dev/shm stays clean.
        let _ = shm_unlink(name.as_c_str());

        nix::unistd::ftruncate(&fd, size as nix::libc::off_t).handle_err(location!())?;

        Ok(fd)
    }

    /// Converts BGRX (XRGB8888 in little-endian memory: B G R X per pixel)
    /// to packed RGB, stripping row padding imposed by `stride`.
    fn bgrx_to_rgb(data: &[u8], stride: u32, width: u32, height: u32) -> Vec<u8> {
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
        for row in 0..height as usize {
            let row_start = row * stride as usize;
            for col in 0..width as usize {
                let px = row_start + col * 4;
                rgb.push(data[px + 2]); // R
                rgb.push(data[px + 1]); // G
                rgb.push(data[px]); // B
            }
        }
        rgb
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
            capture_screen()
        }
    }

    fn capture_screen() -> Result<Screenshot, Error> {
        // SAFETY: all GDI/User32 calls are valid given the handle lifetimes
        // managed within this function; no pointers escape.
        unsafe {
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
}
