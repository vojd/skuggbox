use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayMode {
    Playing,
    Paused,
}

impl Default for PlayMode {
    fn default() -> Self {
        Self::Paused
    }
}

pub enum PlaybackControl {
    Forward(f32),
    Rewind(f32),
}

pub fn seek(playback_time: f32, playback_control: PlaybackControl) -> f32 {
    match playback_control {
        PlaybackControl::Forward(t) => playback_time + t,
        PlaybackControl::Rewind(t) => playback_time - t,
    }
}
