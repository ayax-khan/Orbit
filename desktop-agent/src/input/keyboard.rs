//! Keyboard control via the `enigo` crate (0.6 API).

use enigo::{Direction, Enigo, Key, Keyboard};

use super::InputError;

/// Simulates keyboard input (Unicode text and named keys) on the host.
pub struct KeyboardHandler<'a> {
    enigo: &'a mut Enigo,
}

impl<'a> KeyboardHandler<'a> {
    pub fn new(enigo: &'a mut Enigo) -> Self {
        Self { enigo }
    }

    /// Type a Unicode string as-is (supports arbitrary characters).
    pub fn type_text(&mut self, text: &str) -> Result<(), InputError> {
        self.enigo
            .text(text)
            .map_err(|e| InputError::Simulation(e.to_string()))
    }

    /// Press and release a single named key (Enter, Backspace, arrows, ...).
    #[allow(dead_code)] // convenience wrapper; the processor uses `key` directly
    pub fn key_click(&mut self, key: Key) -> Result<(), InputError> {
        self.enigo
            .key(key, Direction::Click)
            .map_err(|e| InputError::Simulation(e.to_string()))
    }

    /// Hold or release a key (used for modifier combinations / shortcuts).
    pub fn key(&mut self, key: Key, direction: Direction) -> Result<(), InputError> {
        self.enigo
            .key(key, direction)
            .map_err(|e| InputError::Simulation(e.to_string()))
    }
}
