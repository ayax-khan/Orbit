use serde::Deserialize;
use super::mouse::MouseHandler;
use super::keyboard::KeyboardHandler;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum InputEvent {
    #[serde(rename = "mouse_click")]
    MouseClick { x: i32, y: i32 },
    #[serde(rename = "key_down")]
    KeyDown { key: String },
}

pub struct InputProcessor {
    mouse: MouseHandler,
    keyboard: KeyboardHandler,
}

impl InputProcessor {
    pub fn new() -> Self {
        Self {
            mouse: MouseHandler::new(),
            keyboard: KeyboardHandler::new(),
        }
    }

    pub fn process_event(&mut self, json_data: &str) {
        if let Ok(event) = serde_json::from_str::<InputEvent>(json_data) {
            match event {
                InputEvent::MouseClick { x, y } => {
                    self.mouse.move_to(x, y);
                    self.mouse.click();
                    println!("Simulated mouse click at {}, {}", x, y);
                }
                InputEvent::KeyDown { key } => {
                    println!("Simulated key down: {}", key);
                    // Map string key to enigo::Key if necessary
                }
            }
        }
    }
}
