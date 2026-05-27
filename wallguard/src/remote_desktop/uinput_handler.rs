/// Kernel-level input injection via `/dev/uinput`.
///
/// Creates two virtual devices:
/// - A keyboard that covers every key used by `MessageHandler`.
/// - An absolute-positioning pointer (tablet mode) with left/right/middle buttons.
///
/// Because events go through the kernel input layer they reach both X11 and
/// Wayland applications, making this backend session-type agnostic.  The agent
/// runs as root so `/dev/uinput` access is always available.
///
/// # Coordinate space
/// The pointer uses `ABS_X / ABS_Y` with the range `[0, screen_w)` × `[0, screen_h)`.
/// Screen dimensions are read from the first Wayland output at construction
/// time (or fall back to 1920×1080 if they cannot be determined).
use evdev::{
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, UinputAbsSetup,
    uinput::VirtualDeviceBuilder,
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

// Composite handle over the two virtual devices.
pub struct UinputHandler {
    keyboard: evdev::uinput::VirtualDevice,
    pointer: evdev::uinput::VirtualDevice,
    /// Width of the virtual screen (ABS_X upper bound).
    screen_w: i32,
    /// Height of the virtual screen (ABS_Y upper bound).
    screen_h: i32,
}

impl UinputHandler {
    /// Create the virtual keyboard and pointer devices.
    ///
    /// `screen_w` / `screen_h` set the coordinate range for the pointer.
    /// Pass `None` to use the default 1920×1080 range.
    pub fn new(screen_w: Option<i32>, screen_h: Option<i32>) -> Result<Self, Error> {
        let screen_w = screen_w.unwrap_or(1920);
        let screen_h = screen_h.unwrap_or(1080);

        let keyboard = build_keyboard()?;
        let pointer = build_pointer(screen_w, screen_h)?;

        Ok(Self {
            keyboard,
            pointer,
            screen_w,
            screen_h,
        })
    }

    // ── mouse ────────────────────────────────────────────────────────────────

    pub fn move_abs(&mut self, x: i32, y: i32) -> Result<(), Error> {
        let x = x.clamp(0, self.screen_w - 1);
        let y = y.clamp(0, self.screen_h - 1);
        self.pointer
            .emit(&[
                InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, x),
                InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, y),
                InputEvent::new(EventType::SYNCHRONIZE, 0, 0),
            ])
            .handle_err(location!())
    }

    pub fn button_press(&mut self, btn: MouseButton) -> Result<(), Error> {
        self.pointer
            .emit(&[
                InputEvent::new(EventType::KEY, btn.evdev_code(), 1),
                InputEvent::new(EventType::SYNCHRONIZE, 0, 0),
            ])
            .handle_err(location!())
    }

    pub fn button_release(&mut self, btn: MouseButton) -> Result<(), Error> {
        self.pointer
            .emit(&[
                InputEvent::new(EventType::KEY, btn.evdev_code(), 0),
                InputEvent::new(EventType::SYNCHRONIZE, 0, 0),
            ])
            .handle_err(location!())
    }

    // ── keyboard ─────────────────────────────────────────────────────────────

    pub fn key_press(&mut self, key: &str) -> Result<(), Error> {
        self.send_key(key, 1)
    }

    pub fn key_release(&mut self, key: &str) -> Result<(), Error> {
        self.send_key(key, 0)
    }

    pub fn key_click(&mut self, key: &str) -> Result<(), Error> {
        self.send_key(key, 1)?;
        self.send_key(key, 0)
    }

    fn send_key(&mut self, key: &str, value: i32) -> Result<(), Error> {
        let (needs_shift, code) = map_key(key);
        if needs_shift {
            self.keyboard
                .emit(&[InputEvent::new(
                    EventType::KEY,
                    Key::KEY_LEFTSHIFT.code(),
                    value,
                )])
                .handle_err(location!())?;
        }
        self.keyboard
            .emit(&[
                InputEvent::new(EventType::KEY, code, value),
                InputEvent::new(EventType::SYNCHRONIZE, 0, 0),
            ])
            .handle_err(location!())
    }
}

// ── MouseButton ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl MouseButton {
    fn evdev_code(self) -> u16 {
        match self {
            MouseButton::Left => Key::BTN_LEFT.code(),
            MouseButton::Right => Key::BTN_RIGHT.code(),
            MouseButton::Middle => Key::BTN_MIDDLE.code(),
            MouseButton::Back => Key::BTN_BACK.code(),
            MouseButton::Forward => Key::BTN_FORWARD.code(),
        }
    }
}

// ── device builders ───────────────────────────────────────────────────────────

fn build_keyboard() -> Result<evdev::uinput::VirtualDevice, Error> {
    let mut keys = AttributeSet::<Key>::new();
    for k in ALL_KEYBOARD_KEYS {
        keys.insert(*k);
    }
    VirtualDeviceBuilder::new()
        .handle_err(location!())?
        .name("WallGuard Virtual Keyboard")
        .with_keys(&keys)
        .handle_err(location!())?
        .build()
        .handle_err(location!())
}

fn build_pointer(w: i32, h: i32) -> Result<evdev::uinput::VirtualDevice, Error> {
    let mut buttons = AttributeSet::<Key>::new();
    for b in [
        Key::BTN_LEFT,
        Key::BTN_RIGHT,
        Key::BTN_MIDDLE,
        Key::BTN_BACK,
        Key::BTN_FORWARD,
    ] {
        buttons.insert(b);
    }

    // AbsInfo::new(value, minimum, maximum, fuzz, flat, resolution)
    let abs_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, AbsInfo::new(0, 0, w, 0, 0, 1));
    let abs_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, AbsInfo::new(0, 0, h, 0, 0, 1));

    VirtualDeviceBuilder::new()
        .handle_err(location!())?
        .name("WallGuard Virtual Pointer")
        .with_absolute_axis(&abs_x)
        .handle_err(location!())?
        .with_absolute_axis(&abs_y)
        .handle_err(location!())?
        .with_keys(&buttons)
        .handle_err(location!())?
        .build()
        .handle_err(location!())
}

// ── key mapping ───────────────────────────────────────────────────────────────

/// Map a key name (as sent by the browser client) to `(needs_shift, evdev_code)`.
///
/// `needs_shift` is true for uppercase letters and shift-modified symbols so
/// that the virtual keyboard sends a syntactically correct key sequence.
fn map_key(name: &str) -> (bool, u16) {
    match name.to_lowercase().as_str() {
        // Named keys
        "backspace" => (false, Key::KEY_BACKSPACE.code()),
        "control" => (false, Key::KEY_LEFTCTRL.code()),
        "meta" => (false, Key::KEY_LEFTMETA.code()),
        "alt" => (false, Key::KEY_LEFTALT.code()),
        "tab" => (false, Key::KEY_TAB.code()),
        "capslock" => (false, Key::KEY_CAPSLOCK.code()),
        "shift" => (false, Key::KEY_LEFTSHIFT.code()),
        "escape" => (false, Key::KEY_ESC.code()),
        "delete" => (false, Key::KEY_DELETE.code()),
        "enter" => (false, Key::KEY_ENTER.code()),
        "arrowup" => (false, Key::KEY_UP.code()),
        "arrowdown" => (false, Key::KEY_DOWN.code()),
        "arrowleft" => (false, Key::KEY_LEFT.code()),
        "arrowright" => (false, Key::KEY_RIGHT.code()),
        "home" => (false, Key::KEY_HOME.code()),
        "end" => (false, Key::KEY_END.code()),
        "pageup" => (false, Key::KEY_PAGEUP.code()),
        "pagedown" => (false, Key::KEY_PAGEDOWN.code()),
        "f1" => (false, Key::KEY_F1.code()),
        "f2" => (false, Key::KEY_F2.code()),
        "f3" => (false, Key::KEY_F3.code()),
        "f4" => (false, Key::KEY_F4.code()),
        "f5" => (false, Key::KEY_F5.code()),
        "f6" => (false, Key::KEY_F6.code()),
        "f7" => (false, Key::KEY_F7.code()),
        "f8" => (false, Key::KEY_F8.code()),
        "f9" => (false, Key::KEY_F9.code()),
        "f10" => (false, Key::KEY_F10.code()),
        "f11" => (false, Key::KEY_F11.code()),
        "f12" => (false, Key::KEY_F12.code()),

        // Single character: use first char of original (pre-lowercase) name
        // to detect uppercase / shifted symbols.
        _ => {
            let c = name.chars().next().unwrap_or('\0');
            char_to_key(c)
        }
    }
}

/// Map a Unicode character to `(needs_shift, evdev_code)` using a US QWERTY layout.
fn char_to_key(c: char) -> (bool, u16) {
    match c {
        'a' => (false, Key::KEY_A.code()),
        'b' => (false, Key::KEY_B.code()),
        'c' => (false, Key::KEY_C.code()),
        'd' => (false, Key::KEY_D.code()),
        'e' => (false, Key::KEY_E.code()),
        'f' => (false, Key::KEY_F.code()),
        'g' => (false, Key::KEY_G.code()),
        'h' => (false, Key::KEY_H.code()),
        'i' => (false, Key::KEY_I.code()),
        'j' => (false, Key::KEY_J.code()),
        'k' => (false, Key::KEY_K.code()),
        'l' => (false, Key::KEY_L.code()),
        'm' => (false, Key::KEY_M.code()),
        'n' => (false, Key::KEY_N.code()),
        'o' => (false, Key::KEY_O.code()),
        'p' => (false, Key::KEY_P.code()),
        'q' => (false, Key::KEY_Q.code()),
        'r' => (false, Key::KEY_R.code()),
        's' => (false, Key::KEY_S.code()),
        't' => (false, Key::KEY_T.code()),
        'u' => (false, Key::KEY_U.code()),
        'v' => (false, Key::KEY_V.code()),
        'w' => (false, Key::KEY_W.code()),
        'x' => (false, Key::KEY_X.code()),
        'y' => (false, Key::KEY_Y.code()),
        'z' => (false, Key::KEY_Z.code()),
        'A' => (true, Key::KEY_A.code()),
        'B' => (true, Key::KEY_B.code()),
        'C' => (true, Key::KEY_C.code()),
        'D' => (true, Key::KEY_D.code()),
        'E' => (true, Key::KEY_E.code()),
        'F' => (true, Key::KEY_F.code()),
        'G' => (true, Key::KEY_G.code()),
        'H' => (true, Key::KEY_H.code()),
        'I' => (true, Key::KEY_I.code()),
        'J' => (true, Key::KEY_J.code()),
        'K' => (true, Key::KEY_K.code()),
        'L' => (true, Key::KEY_L.code()),
        'M' => (true, Key::KEY_M.code()),
        'N' => (true, Key::KEY_N.code()),
        'O' => (true, Key::KEY_O.code()),
        'P' => (true, Key::KEY_P.code()),
        'Q' => (true, Key::KEY_Q.code()),
        'R' => (true, Key::KEY_R.code()),
        'S' => (true, Key::KEY_S.code()),
        'T' => (true, Key::KEY_T.code()),
        'U' => (true, Key::KEY_U.code()),
        'V' => (true, Key::KEY_V.code()),
        'W' => (true, Key::KEY_W.code()),
        'X' => (true, Key::KEY_X.code()),
        'Y' => (true, Key::KEY_Y.code()),
        'Z' => (true, Key::KEY_Z.code()),
        '0' => (false, Key::KEY_0.code()),
        '1' => (false, Key::KEY_1.code()),
        '2' => (false, Key::KEY_2.code()),
        '3' => (false, Key::KEY_3.code()),
        '4' => (false, Key::KEY_4.code()),
        '5' => (false, Key::KEY_5.code()),
        '6' => (false, Key::KEY_6.code()),
        '7' => (false, Key::KEY_7.code()),
        '8' => (false, Key::KEY_8.code()),
        '9' => (false, Key::KEY_9.code()),
        ')' => (true, Key::KEY_0.code()),
        '!' => (true, Key::KEY_1.code()),
        '@' => (true, Key::KEY_2.code()),
        '#' => (true, Key::KEY_3.code()),
        '$' => (true, Key::KEY_4.code()),
        '%' => (true, Key::KEY_5.code()),
        '^' => (true, Key::KEY_6.code()),
        '&' => (true, Key::KEY_7.code()),
        '*' => (true, Key::KEY_8.code()),
        '(' => (true, Key::KEY_9.code()),
        ' ' => (false, Key::KEY_SPACE.code()),
        '-' => (false, Key::KEY_MINUS.code()),
        '_' => (true, Key::KEY_MINUS.code()),
        '=' => (false, Key::KEY_EQUAL.code()),
        '+' => (true, Key::KEY_EQUAL.code()),
        '[' => (false, Key::KEY_LEFTBRACE.code()),
        '{' => (true, Key::KEY_LEFTBRACE.code()),
        ']' => (false, Key::KEY_RIGHTBRACE.code()),
        '}' => (true, Key::KEY_RIGHTBRACE.code()),
        '\\' => (false, Key::KEY_BACKSLASH.code()),
        '|' => (true, Key::KEY_BACKSLASH.code()),
        ';' => (false, Key::KEY_SEMICOLON.code()),
        ':' => (true, Key::KEY_SEMICOLON.code()),
        '\'' => (false, Key::KEY_APOSTROPHE.code()),
        '"' => (true, Key::KEY_APOSTROPHE.code()),
        ',' => (false, Key::KEY_COMMA.code()),
        '<' => (true, Key::KEY_COMMA.code()),
        '.' => (false, Key::KEY_DOT.code()),
        '>' => (true, Key::KEY_DOT.code()),
        '/' => (false, Key::KEY_SLASH.code()),
        '?' => (true, Key::KEY_SLASH.code()),
        '`' => (false, Key::KEY_GRAVE.code()),
        '~' => (true, Key::KEY_GRAVE.code()),
        // Fall back to space for unmapped characters.
        _ => (false, Key::KEY_SPACE.code()),
    }
}

// ── keyboard key set ──────────────────────────────────────────────────────────

/// Every key the virtual keyboard device advertises to the kernel.
///
/// Only keys actually used by `map_key()` / `char_to_key()` are listed — the
/// kernel rejects device creation if you claim a key code that does not exist.
static ALL_KEYBOARD_KEYS: &[Key] = &[
    Key::KEY_A,
    Key::KEY_B,
    Key::KEY_C,
    Key::KEY_D,
    Key::KEY_E,
    Key::KEY_F,
    Key::KEY_G,
    Key::KEY_H,
    Key::KEY_I,
    Key::KEY_J,
    Key::KEY_K,
    Key::KEY_L,
    Key::KEY_M,
    Key::KEY_N,
    Key::KEY_O,
    Key::KEY_P,
    Key::KEY_Q,
    Key::KEY_R,
    Key::KEY_S,
    Key::KEY_T,
    Key::KEY_U,
    Key::KEY_V,
    Key::KEY_W,
    Key::KEY_X,
    Key::KEY_Y,
    Key::KEY_Z,
    Key::KEY_0,
    Key::KEY_1,
    Key::KEY_2,
    Key::KEY_3,
    Key::KEY_4,
    Key::KEY_5,
    Key::KEY_6,
    Key::KEY_7,
    Key::KEY_8,
    Key::KEY_9,
    Key::KEY_SPACE,
    Key::KEY_BACKSPACE,
    Key::KEY_TAB,
    Key::KEY_ENTER,
    Key::KEY_ESC,
    Key::KEY_DELETE,
    Key::KEY_HOME,
    Key::KEY_END,
    Key::KEY_PAGEUP,
    Key::KEY_PAGEDOWN,
    Key::KEY_UP,
    Key::KEY_DOWN,
    Key::KEY_LEFT,
    Key::KEY_RIGHT,
    Key::KEY_F1,
    Key::KEY_F2,
    Key::KEY_F3,
    Key::KEY_F4,
    Key::KEY_F5,
    Key::KEY_F6,
    Key::KEY_F7,
    Key::KEY_F8,
    Key::KEY_F9,
    Key::KEY_F10,
    Key::KEY_F11,
    Key::KEY_F12,
    Key::KEY_LEFTSHIFT,
    Key::KEY_RIGHTSHIFT,
    Key::KEY_LEFTCTRL,
    Key::KEY_RIGHTCTRL,
    Key::KEY_LEFTALT,
    Key::KEY_RIGHTALT,
    Key::KEY_LEFTMETA,
    Key::KEY_RIGHTMETA,
    Key::KEY_CAPSLOCK,
    Key::KEY_MINUS,
    Key::KEY_EQUAL,
    Key::KEY_LEFTBRACE,
    Key::KEY_RIGHTBRACE,
    Key::KEY_BACKSLASH,
    Key::KEY_SEMICOLON,
    Key::KEY_APOSTROPHE,
    Key::KEY_COMMA,
    Key::KEY_DOT,
    Key::KEY_SLASH,
    Key::KEY_GRAVE,
];
