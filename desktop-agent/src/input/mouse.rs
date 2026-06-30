use enigo::{Enigo, MouseControllable};

pub struct MouseHandler {
    enigo: Enigo,
}

impl MouseHandler {
    pub fn new() -> Self {
        Self { enigo: Enigo::new() }
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.enigo.mouse_move_to(x, y);
    }

    pub fn click(&mut self) {
        self.enigo.mouse_click(enigo::MouseButton::Left);
    }
}
