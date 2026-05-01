//! Input injection via the `enigo` crate.
//!
//! Uses the `x11rb` backend on Linux/FreeBSD and the native Win32 backend on
//! Windows.  Each `EnigoInjector` owns one connection to the display server
//! and must only be used from a single thread at a time.

use anyhow::{Context as _, Result};
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard as _, Mouse as _, Settings};
use wg_shared::rdp::InputEvent;

use super::InputInjector;

pub struct EnigoInjector {
    enigo: Enigo,
}

// Safety: Enigo's x11rb connection is per-instance and is not shared across
// threads.  EnigoInjector is always used from a single dedicated inject thread.
unsafe impl Send for EnigoInjector {}

impl EnigoInjector {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())
            .context("enigo: failed to connect to display for input injection")?;
        Ok(Self { enigo })
    }
}

impl InputInjector for EnigoInjector {
    fn inject(&mut self, event: InputEvent) -> Result<()> {
        match event {
            InputEvent::MouseMove { x, y } => {
                self.enigo.move_mouse(x, y, Coordinate::Abs)
                    .context("mouse move")?;
            }
            InputEvent::MouseDown { btn } => {
                if let Some(b) = browser_btn(btn) {
                    self.enigo.button(b, Direction::Press)
                        .context("mouse down")?;
                }
            }
            InputEvent::MouseUp { btn } => {
                if let Some(b) = browser_btn(btn) {
                    self.enigo.button(b, Direction::Release)
                        .context("mouse up")?;
                }
            }
            InputEvent::MouseScroll { dy, .. } => {
                if dy != 0 {
                    self.enigo.scroll(dy, Axis::Vertical)
                        .context("scroll")?;
                }
            }
            InputEvent::KeyDown { code } => {
                if let Some(k) = browser_code_to_key(&code) {
                    self.enigo.key(k, Direction::Press)
                        .context("key down")?;
                }
            }
            InputEvent::KeyUp { code } => {
                if let Some(k) = browser_code_to_key(&code) {
                    self.enigo.key(k, Direction::Release)
                        .context("key up")?;
                }
            }
            InputEvent::Clipboard { text } => {
                self.enigo.fast_text(&text)
                    .context("clipboard paste")?;
            }
            // PLI is handled at the tunnel layer, not injected as input.
            InputEvent::Pli => {}
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Browser key / button mappings
// ---------------------------------------------------------------------------

fn browser_btn(btn: u8) -> Option<Button> {
    match btn {
        0 => Some(Button::Left),
        1 => Some(Button::Middle),
        2 => Some(Button::Right),
        _ => None,
    }
}

/// Map a `KeyboardEvent.code` string (physical key) to an enigo `Key`.
///
/// Uses lowercase Unicode for letter keys so the OS applies modifier
/// translation (Shift, CapsLock) correctly.
fn browser_code_to_key(code: &str) -> Option<Key> {
    // Letter keys: "KeyA" → Key::Unicode('a'), …, "KeyZ" → Key::Unicode('z')
    if let Some(rest) = code.strip_prefix("Key") {
        if rest.len() == 1 {
            if let Some(c) = rest.chars().next() {
                if c.is_ascii_alphabetic() {
                    return Some(Key::Unicode(c.to_ascii_lowercase()));
                }
            }
        }
    }

    // Digit row: "Digit0" … "Digit9"
    if let Some(rest) = code.strip_prefix("Digit") {
        if rest.len() == 1 {
            if let Some(c) = rest.chars().next() {
                if c.is_ascii_digit() {
                    return Some(Key::Unicode(c));
                }
            }
        }
    }

    // Special and named keys
    Some(match code {
        "Space"                           => Key::Space,
        "Enter" | "NumpadEnter"           => Key::Return,
        "Backspace"                       => Key::Backspace,
        "Delete"                          => Key::Delete,
        "Tab"                             => Key::Tab,
        "Escape"                          => Key::Escape,
        "ArrowUp"                         => Key::UpArrow,
        "ArrowDown"                       => Key::DownArrow,
        "ArrowLeft"                       => Key::LeftArrow,
        "ArrowRight"                      => Key::RightArrow,
        "ShiftLeft"   | "ShiftRight"      => Key::Shift,
        "ControlLeft" | "ControlRight"    => Key::Control,
        "AltLeft"     | "AltRight"        => Key::Alt,
        "MetaLeft"    | "MetaRight"       => Key::Meta,
        "CapsLock"                        => Key::CapsLock,
        "Home"                            => Key::Home,
        "End"                             => Key::End,
        "PageUp"                          => Key::PageUp,
        "PageDown"                        => Key::PageDown,
        "Insert"                          => Key::Insert,
        "F1"  => Key::F1,  "F2"  => Key::F2,  "F3"  => Key::F3,
        "F4"  => Key::F4,  "F5"  => Key::F5,  "F6"  => Key::F6,
        "F7"  => Key::F7,  "F8"  => Key::F8,  "F9"  => Key::F9,
        "F10" => Key::F10, "F11" => Key::F11, "F12" => Key::F12,
        _ => return None,
    })
}
