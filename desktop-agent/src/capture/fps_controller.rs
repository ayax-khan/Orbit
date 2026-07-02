use std::time::Duration;

/// Adaptive frame-rate controller (spec: "Adaptive FPS").
///
/// Reduces FPS when network latency is high and increases it again when the
/// link recovers, staying within a configured min/max range.
pub struct FpsController {
    min_fps: u32,
    max_fps: u32,
    current_fps: u32,
}

impl FpsController {
    pub fn new(min_fps: u32, max_fps: u32) -> Self {
        Self {
            min_fps,
            max_fps,
            current_fps: max_fps,
        }
    }

    /// Adjust the target FPS based on the most recent measured latency (ms).
    /// Thresholds follow the spec: back off above 100ms, ramp up below 50ms.
    pub fn adjust(&mut self, latency_ms: u32) {
        if latency_ms > 100 && self.current_fps > self.min_fps {
            self.current_fps = (self.current_fps.saturating_sub(5)).max(self.min_fps);
        } else if latency_ms < 50 && self.current_fps < self.max_fps {
            self.current_fps = (self.current_fps + 5).min(self.max_fps);
        }
    }

    /// Current target FPS (exposed for metrics / telemetry).
    #[allow(dead_code)]
    pub fn current_fps(&self) -> u32 {
        self.current_fps
    }

    /// Time to wait between frames at the current FPS.
    pub fn frame_interval(&self) -> Duration {
        Duration::from_micros(1_000_000 / self.current_fps.max(1) as u64)
    }
}
