// Encoder abstraction for H.264
pub trait Encoder {
    fn encode(&self, frame: &[u8]) -> Result<Vec<u8>, String>;
}

pub struct H264Encoder {
    // encoder_type: Hardware/Software,
}

impl H264Encoder {
    pub fn new() -> Self {
        println!("Initializing H.264 Encoder...");
        Self {}
    }
}
