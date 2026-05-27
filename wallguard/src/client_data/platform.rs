use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::fmt;
use std::path::Path;

use crate::data_transmission::sysconfig::data::{
    ConfigXml, NftablesRuleset, SystemConfigurationFile,
};

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Generic,
    PfSense,
    OpnSense,
    NfTables,
}

impl TryFrom<&str> for Platform {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "generic" => Ok(Platform::Generic),
            "pfsense" => Ok(Platform::PfSense),
            "opnsense" => Ok(Platform::OpnSense),
            "nftables" => Ok(Platform::NfTables),
            _ => {
                let errmsg = format!("Unsupported platform {value}");
                Err(errmsg).handle_err(location!())
            }
        }
    }
}

impl TryFrom<String> for Platform {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Platform::PfSense => "pfsense",
            Platform::OpnSense => "opnsense",
            Platform::Generic => "generic",
            Platform::NfTables => "nftables",
        };

        write!(f, "{value}")
    }
}

impl Platform {
    pub fn can_monitor_config(&self) -> bool {
        !matches!(self, Platform::Generic)
    }

    pub fn can_monitor_telemetry(&self) -> bool {
        true
    }

    pub fn can_monitor_traffic(&self) -> bool {
        true
    }

    pub fn get_sysconf_files(&self) -> Vec<SystemConfigurationFile> {
        match self {
            Platform::PfSense | Platform::OpnSense => {
                let file = ConfigXml::default();
                vec![SystemConfigurationFile::ConfigXml(file)]
            }
            Platform::NfTables => {
                let file = NftablesRuleset::default();
                vec![SystemConfigurationFile::NftablesRuleset(file)]
            }
            Platform::Generic => vec![],
        }
    }
}

// ── Desktop environment detection ─────────────────────────────────────────────

/// Returns `true` when a display server (X11 or Wayland) is reachable from the
/// current process, regardless of how the `Platform` variant was configured.
///
/// `pub(crate)` so that `pseudo_rd` can use this as a cheap pre-filter before
/// attempting the more expensive (and potentially noisy) `Enigo::new()` probe.
pub(crate) fn has_desktop_environment() -> bool {
    has_x11_display() || has_wayland_display()
}

/// Attempts an actual connection to the X11 display server.
///
/// On Linux the `x11rb` crate is available and lets us do a real handshake,
/// which catches cases where DISPLAY is set but the server has died, and
/// also catches system-service scenarios where DISPLAY is not inherited but
/// a live Xorg socket exists under `/tmp/.X11-unix/`.
///
/// On every other Unix-like OS we fall back to checking the env var and the
/// socket directory, which covers XQuartz on macOS and XWayland-with-Xorg
/// on FreeBSD.
///
/// On Windows we ask GDI for the primary screen width: a non-zero result
/// means at least one monitor is active.
fn has_x11_display() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Try a real handshake via DISPLAY first (covers the normal desktop case).
        if x11rb::connect(None).is_ok() {
            return true;
        }
        // Scan /tmp/.X11-unix/ for sockets — catches system-service deployments
        // where DISPLAY is not exported into the environment.
        x11_socket_exists()
    }

    #[cfg(target_os = "macos")]
    {
        // XQuartz sets DISPLAY and creates Unix sockets.
        if std::env::var_os("DISPLAY").is_some() {
            return true;
        }
        // Native Quartz display (no XQuartz): ask CoreGraphics directly.
        has_quartz_display()
    }

    #[cfg(target_os = "windows")]
    {
        has_windows_display()
    }

    #[cfg(target_os = "freebsd")]
    {
        if std::env::var_os("DISPLAY").is_some() {
            return true;
        }

        x11_socket_exists()
    }

    // All other targets: conservatively return false.
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
    )))]
    {
        false
    }
}

/// Returns `true` when a Wayland compositor socket is reachable.
///
/// The compositor creates a socket at `$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY`
/// (or at `$XDG_RUNTIME_DIR/wayland-0` when the var is unset).  We verify
/// the socket path actually exists rather than just checking the env var,
/// because the env var may linger after the compositor dies.
fn has_wayland_display() -> bool {
    // WAYLAND_DISPLAY may be an absolute path or a socket name relative to
    // XDG_RUNTIME_DIR. Unset means "wayland-0" by convention.
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());

    if display.starts_with('/') {
        return Path::new(&display).exists();
    }

    // Resolve relative to XDG_RUNTIME_DIR, then fall back to /run/user/<uid>.
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", get_current_uid()));

    Path::new(&runtime_dir).join(&display).exists()
}

/// Scans `/tmp/.X11-unix/` for socket files created by an X11 server.
///
/// Socket names follow the pattern `X<n>` (e.g. `X0`, `X1`).  When a socket
/// is found and `DISPLAY` is not already set in the environment we synthesise
/// it (e.g. `:0`) so that downstream tools — `enigo`, `copypasta`, etc. —
/// can connect without requiring the caller to have inherited `DISPLAY` from
/// a login session.
///
/// # Safety of `set_var`
/// `std::env::set_var` is inherently racy in a multi-threaded process.
/// This call is safe in practice because:
///   * It is only reached when `DISPLAY` is absent (checked immediately before).
///   * It is invoked once, early in `Context::new()`, before the agent's
///     worker tasks start reading the environment.
///   * After this point `DISPLAY` is only ever read, never written again.
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn x11_socket_exists() -> bool {
    let Ok(entries) = std::fs::read_dir("/tmp/.X11-unix") else {
        return false;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Socket names are "X0", "X1", …
        if let Some(num) = name.strip_prefix('X')
            && num.parse::<u32>().is_ok()
        {
            if std::env::var_os("DISPLAY").is_none() {
                // SAFETY: see doc-comment above.
                unsafe {
                    std::env::set_var("DISPLAY", format!(":{num}"));
                }
            }
            return true;
        }
    }

    false
}

/// Calls `CGMainDisplayID()` via FFI to check for an active Quartz (macOS
/// native) display without requiring the `core-graphics` crate.
///
/// `CGMainDisplayID()` always returns a value; `CGDisplayIsActive()` then
/// confirms that the display is powered on and not in a sleep state.
#[cfg(target_os = "macos")]
fn has_quartz_display() -> bool {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGMainDisplayID() -> u32;
        fn CGDisplayIsActive(display: u32) -> bool;
    }
    unsafe { CGDisplayIsActive(CGMainDisplayID()) }
}

/// Uses `GetSystemMetrics(SM_CXSCREEN)` to check whether a primary screen is
/// present.  Returns zero in headless / no-monitor scenarios.
#[cfg(target_os = "windows")]
fn has_windows_display() -> bool {
    unsafe { winapi::um::winuser::GetSystemMetrics(winapi::um::winuser::SM_CXSCREEN) > 0 }
}

/// Returns the effective UID of the current process for building the
/// XDG_RUNTIME_DIR fallback path.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
fn get_current_uid() -> u32 {
    // `nix` is already a dependency on unix targets.
    nix::unistd::getuid().as_raw()
}

#[cfg(target_os = "windows")]
fn get_current_uid() -> u32 {
    0
}
