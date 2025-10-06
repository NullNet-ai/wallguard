use copypasta::{ClipboardContext, ClipboardProvider};
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
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

#[derive(Clone)]
pub struct MessageHandler {
    enigo: Arc<Mutex<Enigo>>,
    clctx: Arc<Mutex<ClipboardContext>>,
}

impl fmt::Debug for MessageHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageHandler")
            .field("enigo", &self.enigo)
            // .field("clctx", &self.clctx)
            .finish()
    }
}

impl MessageHandler {
    pub fn new() -> Result<Self, Error> {
        let enigo = Enigo::new(&Settings::default()).handle_err(location!())?;
        let clctx = ClipboardContext::new().handle_err(location!())?;

        Ok(Self {
            enigo: Arc::new(Mutex::new(enigo)),
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
                let message =
                    serde_json::from_value::<MouseMessage>(json).handle_err(location!())?;

                self.on_mouse_message(message).await?;
            }
            "keyup" | "keydown" | "keypress" => {
                let message =
                    serde_json::from_value::<KeyboardMessage>(json).handle_err(location!())?;

                self.on_keyboard_message(message).await?;
            }
            "clipboard" => {
                let message =
                    serde_json::from_value::<ClipboardMessage>(json).handle_err(location!())?;

                self.on_clipboard_message(message).await?;
            }
            mt => {
                return Err(format!("{mt} message type is not supported")).handle_err(location!());
            }
        };

        Ok(())
    }

    async fn on_mouse_message(&self, message: MouseMessage) -> Result<(), Error> {
        let button = match message.button.to_lowercase().as_str() {
            "left" => Button::Left,
            "middle" => Button::Middle,
            "rigth" => Button::Right,
            "back" => Button::Back,
            "forward" => Button::Forward,
            _ => Err(format!("Unsupported mouse button {}", message.button))
                .handle_err(location!())?,
        };

        match message.message_type.to_lowercase().as_str() {
            "mousemove" => self
                .enigo
                .lock()
                .await
                .move_mouse(message.x, message.y, Coordinate::Abs)
                .handle_err(location!()),
            "mousedown" => self
                .enigo
                .lock()
                .await
                .button(button, Direction::Press)
                .handle_err(location!()),
            "mouseup" => self
                .enigo
                .lock()
                .await
                .button(button, Direction::Release)
                .handle_err(location!()),
            _ => Err(format!("Unsupported mouse message type {}", message.button))
                .handle_err(location!())?,
        }
    }

    async fn on_keyboard_message(&self, message: KeyboardMessage) -> Result<(), Error> {
        let key = match message.key.to_lowercase().as_str() {
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

            key => Key::Unicode(key.chars().next().unwrap_or('\0')),
        };

        let direction = match message.message_type.to_lowercase().as_str() {
            "keyup" => Direction::Release,
            "keydown" => Direction::Press,
            _ => Direction::Click,
        };

        self.enigo
            .lock()
            .await
            .key(key, direction)
            .handle_err(location!())
    }

    async fn on_clipboard_message(&self, message: ClipboardMessage) -> Result<(), Error> {
        self.clctx
            .lock()
            .await
            .set_contents(message.content)
            .handle_err(location!())
    }
}
