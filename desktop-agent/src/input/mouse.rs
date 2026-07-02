//! Mouse control via the `enigo` crate (0.6 API).

use enigo::{Axis, Button, Coordinate, Direction, Enigo, Mouse};

use super::InputError;

/// Simulates mouse movement, clicks and scrolling on the host machine.
pub struct MouseHandler<'a> {
    enigo: &'a mut Enigo,
}

impl<'a> MouseHandler<'a> {
    pub fn new(enigo: &'a mut Enigo) -> Self {
        Self { enigo }
    }

    /// Move the cursor to an absolute screen coordinate.
    pub fn move_to(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| InputError::Simulation(e.to_string()))
    }

    /// Press and release a mouse button (a full click).
    pub fn click(&mut self, button: Button) -> Result<(), InputError> {
        self.enigo
            .button(button, Direction::Click)
            .map_err(|e| InputError::Simulation(e.to_string()))
    }

    /// Hold or release a button (used for drag operations).
    pub fn button(&mut self, button: Button, direction: Direction) -> Result<(), InputError> {
        self.enigo
            .button(button, direction)
            .map_err(|e| InputError::Simulation(e.to_string()))
    }

    /// Scroll vertically (positive = down) or horizontally.
    pub fn scroll(&mut self, delta: i32, axis: Axis) -> Result<(), InputError> {
        self.enigo
            .scroll(delta, axis)
            .map_err(|e| InputError::Simulation(e.to_string()))
    }
}
