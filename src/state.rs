use crate::{CameraModel, Mouse, OrbitCamera};
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub width: i32,
    pub height: i32,
    /// App state - is the application running?
    pub is_running: bool,
    pub delta_time: f32,
    pub playback_time: f32,
    pub mouse: Mouse,
    /// Running or paused?
    pub play_mode: PlayMode,
    pub camera: Box<dyn CameraModel>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            is_running: true,
            delta_time: 0.0,
            playback_time: 0.0,
            mouse: Mouse::default(),
            play_mode: PlayMode::Playing,
            camera: Box::from(OrbitCamera::default()),
        }
    }
}
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
    Stop,
}

pub fn seek(playback_time: f32, playback_control: PlaybackControl) -> f32 {
    match playback_control {
        PlaybackControl::Forward(t) => playback_time + t,
        PlaybackControl::Rewind(t) => playback_time - t,
        PlaybackControl::Stop => 0.0,
    }
}
