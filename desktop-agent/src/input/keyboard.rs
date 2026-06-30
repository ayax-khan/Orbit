use enigo::{Enigo, KeyboardControllable};

pub struct KeyboardHandler {
    enigo: Enigo,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self { enigo: Enigo::new() }
    }

    pub fn key_sequence(&mut self, sequence: &str) {
        self.enigo.key_sequence(sequence);
    }

    pub fn key_click(&mut self, key: enigo::Key) {
        self.enigo.key_click(key);
    }
}
