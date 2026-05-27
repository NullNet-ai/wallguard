use copypasta::{ClipboardContext, ClipboardProvider};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseMessage {
    message_type: String,
    button: String,
    x: i32,
    y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyboardMessage {
    message_type: String,
    key: String,
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardMessage {
    message_type: String,
    content: String,
}

// ── input backend ─────────────────────────────────────────────────────────────

/// Abstraction over X11 (Enigo) and kernel uinput (Wayland) input backends.
///
/// On session open the agent tries Enigo first — it works on X11 and on
/// Wayland sessions where XWayland is running.  If Enigo fails (pure Wayland
/// without XWayland), it falls back to the Linux uinput backend which injects
/// events at kernel level and therefore works regardless of the display server.
enum InputBackend {
    Enigo(enigo::Enigo),
    #[cfg(target_os = "linux")]
    Uinput(super::uinput_handler::UinputHandler),
}

impl fmt::Debug for InputBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputBackend::Enigo(_) => write!(f, "InputBackend::Enigo"),
            #[cfg(target_os = "linux")]
            InputBackend::Uinput(_) => write!(f, "InputBackend::Uinput"),
        }
    }
}

impl InputBackend {
    fn new() -> Result<Self, Error> {
        use enigo::{Enigo, Settings};

        match Enigo::new(&Settings::default()) {
            Ok(e) => {
                log::info!("Input backend: Enigo (X11/XWayland)");
                Ok(InputBackend::Enigo(e))
            }
            Err(enigo_err) => {
                #[cfg(target_os = "linux")]
                {
                    log::info!(
                        "Enigo unavailable ({enigo_err:?}); \
                         using uinput (Wayland / kernel) backend"
                    );
                    let handler = super::uinput_handler::UinputHandler::new(None, None)?;
                    Ok(InputBackend::Uinput(handler))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    Err(format!("No input backend available: {enigo_err:?}"))
                        .handle_err(location!())
                }
            }
        }
    }

    fn move_mouse(&mut self, x: i32, y: i32) -> Result<(), Error> {
        match self {
            InputBackend::Enigo(e) => {
                use enigo::{Coordinate, Mouse};
                e.move_mouse(x, y, Coordinate::Abs).handle_err(location!())
            }
            #[cfg(target_os = "linux")]
            InputBackend::Uinput(u) => u.move_abs(x, y),
        }
    }

    fn button(&mut self, btn: &str, press: bool) -> Result<(), Error> {
        match self {
            InputBackend::Enigo(e) => {
                use enigo::{Button, Direction, Mouse};
                let button = parse_enigo_button(btn)?;
                let dir = if press {
                    Direction::Press
                } else {
                    Direction::Release
                };
                e.button(button, dir).handle_err(location!())
            }
            #[cfg(target_os = "linux")]
            InputBackend::Uinput(u) => {
                use super::uinput_handler::MouseButton;
                let btn = parse_uinput_button(btn)?;
                if press {
                    u.button_press(btn)
                } else {
                    u.button_release(btn)
                }
            }
        }
    }

    fn key(&mut self, key: &str, direction: KeyDir) -> Result<(), Error> {
        match self {
            InputBackend::Enigo(e) => {
                use enigo::{Direction, Key, Keyboard};
                let k = parse_enigo_key(key);
                let dir = match direction {
                    KeyDir::Press => Direction::Press,
                    KeyDir::Release => Direction::Release,
                    KeyDir::Click => Direction::Click,
                };
                e.key(k, dir).handle_err(location!())
            }
            #[cfg(target_os = "linux")]
            InputBackend::Uinput(u) => match direction {
                KeyDir::Press => u.key_press(key),
                KeyDir::Release => u.key_release(key),
                KeyDir::Click => u.key_click(key),
            },
        }
    }
}

#[derive(Clone, Copy)]
enum KeyDir {
    Press,
    Release,
    Click,
}

// ── MessageHandler ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MessageHandler {
    input: Arc<Mutex<InputBackend>>,
    clctx: Arc<Mutex<ClipboardContext>>,
}

impl fmt::Debug for MessageHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageHandler")
            .field("input", &self.input)
            .finish()
    }
}

impl MessageHandler {
    pub fn new() -> Result<Self, Error> {
        let input = InputBackend::new()?;
        let clctx = ClipboardContext::new().handle_err(location!())?;
        Ok(Self {
            input: Arc::new(Mutex::new(input)),
            clctx: Arc::new(Mutex::new(clctx)),
        })
    }

    pub async fn on_message(&self, message: Vec<u8>) -> Result<(), Error> {
        let json = serde_json::from_slice::<serde_json::Value>(&message).handle_err(location!())?;

        let message_type = json
            .get("message_type")
            .ok_or("Message Type is missing")
            .handle_err(location!())?
            .as_str()
            .ok_or("Wrong type")
            .handle_err(location!())?;

        match message_type.to_lowercase().as_str() {
            "mousemove" | "mousedown" | "mouseup" => {
                let msg = serde_json::from_value::<MouseMessage>(json).handle_err(location!())?;
                self.on_mouse_message(msg).await?;
            }
            "keyup" | "keydown" | "keypress" => {
                let msg =
                    serde_json::from_value::<KeyboardMessage>(json).handle_err(location!())?;
                self.on_keyboard_message(msg).await?;
            }
            "clipboard" => {
                let msg =
                    serde_json::from_value::<ClipboardMessage>(json).handle_err(location!())?;
                self.on_clipboard_message(msg).await?;
            }
            mt => {
                return Err(format!("{mt} message type is not supported")).handle_err(location!());
            }
        };

        Ok(())
    }

    async fn on_mouse_message(&self, message: MouseMessage) -> Result<(), Error> {
        let mut input = self.input.lock().await;
        match message.message_type.to_lowercase().as_str() {
            "mousemove" => input.move_mouse(message.x, message.y),
            "mousedown" => input.button(&message.button, true),
            "mouseup" => input.button(&message.button, false),
            _ => Err(format!(
                "Unsupported mouse message type {}",
                message.message_type
            ))
            .handle_err(location!()),
        }
    }

    async fn on_keyboard_message(&self, message: KeyboardMessage) -> Result<(), Error> {
        let direction = match message.message_type.to_lowercase().as_str() {
            "keydown" => KeyDir::Press,
            "keyup" => KeyDir::Release,
            _ => KeyDir::Click,
        };
        self.input.lock().await.key(&message.key, direction)
    }

    async fn on_clipboard_message(&self, message: ClipboardMessage) -> Result<(), Error> {
        self.clctx
            .lock()
            .await
            .set_contents(message.content)
            .handle_err(location!())
    }
}

// ── key / button parsers (Enigo) ──────────────────────────────────────────────

fn parse_enigo_button(s: &str) -> Result<enigo::Button, Error> {
    use enigo::Button;
    match s.to_lowercase().as_str() {
        "left" => Ok(Button::Left),
        "middle" => Ok(Button::Middle),
        "right" | "rigth" => Ok(Button::Right),
        "back" => Ok(Button::Back),
        "forward" => Ok(Button::Forward),
        _ => Err(format!("Unsupported mouse button {s}")).handle_err(location!()),
    }
}

fn parse_enigo_key(key: &str) -> enigo::Key {
    use enigo::Key;
    match key.to_lowercase().as_str() {
        "backspace" => Key::Backspace,
        "control" => Key::Control,
        "meta" => Key::Meta,
        "alt" => Key::Alt,
        "tab" => Key::Tab,
        "capslock" => Key::CapsLock,
        "shift" => Key::Shift,
        "escape" => Key::Escape,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        "delete" => Key::Delete,
        "enter" => Key::Return,
        "arrowup" => Key::UpArrow,
        "arrowdown" => Key::DownArrow,
        "arrowleft" => Key::LeftArrow,
        "arrowright" => Key::RightArrow,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        k => Key::Unicode(k.chars().next().unwrap_or('\0')),
    }
}

// ── button parser (uinput) ────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn parse_uinput_button(s: &str) -> Result<super::uinput_handler::MouseButton, Error> {
    use super::uinput_handler::MouseButton;
    match s.to_lowercase().as_str() {
        "left" => Ok(MouseButton::Left),
        "middle" => Ok(MouseButton::Middle),
        "right" | "rigth" => Ok(MouseButton::Right),
        "back" => Ok(MouseButton::Back),
        "forward" => Ok(MouseButton::Forward),
        _ => Err(format!("Unsupported mouse button {s}")).handle_err(location!()),
    }
}
