//! Types shared between the agent (deserialiser) and the browser UI (serialiser)
//! for the RDP input event protocol.
//!
//! Events are framed over the tunnel stream as:
//!   `[4 bytes LE payload_len][payload_len bytes JSON]`
//!
//! The JSON uses `"t"` as the discriminant tag, e.g.:
//!   `{"t":"mouse_move","x":100,"y":200}`

use serde::{Deserialize, Serialize};

/// An input event sent from the browser to the agent over the RDP tunnel.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum InputEvent {
    /// Absolute mouse move to `(x, y)` in screen coordinates.
    MouseMove   { x: i32, y: i32 },
    /// Mouse button pressed.  `btn`: 0 = left, 1 = middle, 2 = right.
    MouseDown   { btn: u8 },
    /// Mouse button released.
    MouseUp     { btn: u8 },
    /// Scroll wheel.  `dy` > 0 = down, < 0 = up; `dx` = horizontal.
    MouseScroll { dx: i32, dy: i32 },
    /// Key pressed.  `code` is a Web `KeyboardEvent.code` value (e.g. `"KeyA"`).
    KeyDown     { code: String },
    /// Key released.
    KeyUp       { code: String },
    /// Text pasted from the browser clipboard.
    Clipboard   { text: String },
    /// Picture Loss Indication — agent should emit an IDR frame immediately.
    Pli,
}
