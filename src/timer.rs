use std::time::Instant;

pub struct Timer {
    /// current frame
    time: Instant,
    /// last frame
    last_time: Instant,

    pub delta_time: f32,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    pub fn new() -> Self {
        Self {
            time: Instant::now(),
            last_time: Instant::now(),
            delta_time: 0.0,
        }
    }

    /// Start of frame
    pub fn start(&mut self) {
        self.time = Instant::now();
        self.delta_time = (self.time - self.last_time).as_secs_f32();
    }

    /// End of frame
    pub fn stop(&mut self) {
        self.last_time = self.time;
    }
}
