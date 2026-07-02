//! Remote input protocol and the processor that applies events to the host.
//!
//! Events arrive as JSON over the WebRTC data channel from the Chrome
//! extension and are decoded into [`InputEvent`]s, then applied through the
//! `enigo` backend (mouse, keyboard, clipboard-adjacent text entry).

use enigo::{Axis, Button, Direction, Enigo, Key, Settings};
use serde::Deserialize;

use super::keyboard::KeyboardHandler;
use super::mouse::MouseHandler;
use super::InputError;

/// A single input command sent by the client. The `type` tag matches the
/// messages produced by the extension's `useInput` hook.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputEvent {
    /// Absolute cursor move.
    MouseMove { x: i32, y: i32 },
    /// Move then perform a full click with the given button (default: left).
    MouseClick {
        x: i32,
        y: i32,
        #[serde(default)]
        button: MouseButton,
    },
    /// Button press or release, for drag operations.
    MouseButton {
        #[serde(default)]
        button: MouseButton,
        pressed: bool,
    },
    /// Wheel scroll. Positive `delta_y` scrolls down.
    MouseScroll {
        #[serde(default)]
        delta_x: i32,
        #[serde(default)]
        delta_y: i32,
    },
    /// A named key press (Enter, Backspace, arrows, modifiers, ...).
    KeyDown { key: String },
    /// A named key release (for held modifiers/shortcuts).
    KeyUp { key: String },
    /// Type a Unicode string verbatim (clipboard paste / IME text).
    Text { text: String },
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

impl From<MouseButton> for Button {
    fn from(b: MouseButton) -> Self {
        match b {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            MouseButton::Middle => Button::Middle,
        }
    }
}

/// Owns a single `enigo` instance and applies decoded input events to it.
pub struct InputProcessor {
    enigo: Enigo,
}

impl InputProcessor {
    pub fn new() -> Result<Self, InputError> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| InputError::Init(e.to_string()))?;
        Ok(Self { enigo })
    }

    /// Decode a JSON payload and apply it. Errors are returned to the caller
    /// so they can be logged without tearing down the connection.
    pub fn process_json(&mut self, json: &str) -> Result<(), InputError> {
        let event: InputEvent = serde_json::from_str(json)
            .map_err(|e| InputError::Simulation(format!("bad input payload: {e}")))?;
        self.apply(event)
    }

    fn apply(&mut self, event: InputEvent) -> Result<(), InputError> {
        match event {
            InputEvent::MouseMove { x, y } => {
                MouseHandler::new(&mut self.enigo).move_to(x, y)
            }
            InputEvent::MouseClick { x, y, button } => {
                let mut mouse = MouseHandler::new(&mut self.enigo);
                mouse.move_to(x, y)?;
                mouse.click(button.into())
            }
            InputEvent::MouseButton { button, pressed } => {
                let dir = if pressed { Direction::Press } else { Direction::Release };
                MouseHandler::new(&mut self.enigo).button(button.into(), dir)
            }
            InputEvent::MouseScroll { delta_x, delta_y } => {
                let mut mouse = MouseHandler::new(&mut self.enigo);
                if delta_y != 0 {
                    mouse.scroll(delta_y, Axis::Vertical)?;
                }
                if delta_x != 0 {
                    mouse.scroll(delta_x, Axis::Horizontal)?;
                }
                Ok(())
            }
            InputEvent::KeyDown { key } => self.apply_key(&key, Direction::Click),
            InputEvent::KeyUp { key } => self.apply_key(&key, Direction::Release),
            InputEvent::Text { text } => {
                KeyboardHandler::new(&mut self.enigo).type_text(&text)
            }
        }
    }

    fn apply_key(&mut self, key: &str, direction: Direction) -> Result<(), InputError> {
        let mut keyboard = KeyboardHandler::new(&mut self.enigo);
        match map_key(key) {
            Some(k) => keyboard.key(k, direction),
            // Unknown single-character keys are typed as text (covers letters,
            // digits and punctuation coming from `KeyboardEvent.key`).
            None if key.chars().count() == 1 && direction != Direction::Release => {
                keyboard.type_text(key)
            }
            None => Ok(()),
        }
    }
}

/// Map a web `KeyboardEvent.key` name to an `enigo::Key`.
fn map_key(key: &str) -> Option<Key> {
    let mapped = match key {
        "Enter" => Key::Return,
        "Backspace" => Key::Backspace,
        "Tab" => Key::Tab,
        "Escape" | "Esc" => Key::Escape,
        "Delete" | "Del" => Key::Delete,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "ArrowUp" => Key::UpArrow,
        "ArrowDown" => Key::DownArrow,
        "ArrowLeft" => Key::LeftArrow,
        "ArrowRight" => Key::RightArrow,
        " " | "Space" | "Spacebar" => Key::Space,
        "Control" => Key::Control,
        "Shift" => Key::Shift,
        "Alt" => Key::Alt,
        "Meta" | "OS" => Key::Meta,
        _ => return None,
    };
    Some(mapped)
}
