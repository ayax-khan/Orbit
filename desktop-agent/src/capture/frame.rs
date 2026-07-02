/// A single captured screen frame in BGRA8 (top-down) layout, matching the
/// output of the Windows Desktop Duplication API.
#[derive(Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// Raw pixel data, 4 bytes per pixel (BGRA).
    pub data: Vec<u8>,
}

impl Frame {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self { width, height, data }
    }

    /// Expected byte length for the frame's dimensions.
    pub fn expected_len(&self) -> usize {
        (self.width as usize) * (self.height as usize) * 4
    }
}
