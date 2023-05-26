use crate::camera::{CameraModel, OrbitCamera};
use crate::{Mouse, ShaderError, Timer};
use glam::Vec3;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub width: i32,
    pub height: i32,
    /// App state - is the application running?
    pub is_running: bool,
    pub timer: Timer,
    pub delta_time: f32,
    pub playback_time: f32,
    pub mouse: Mouse,
    pub modifier: ActionModifier,
    /// Running or paused?
    pub play_mode: PlayMode,
    pub ui_visible: bool,
    pub is_fullscreen: bool,
    pub camera: Box<dyn CameraModel>,
    // TODO(mathias): Move the camera pos into the camera model
    pub camera_pos: Vec3,
    pub shader_error: Option<ShaderError>,

    pub scene_vars: SceneVars,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            is_running: true,
            timer: Timer::default(),
            delta_time: 0.0,
            playback_time: 0.0,
            mouse: Mouse::default(),
            modifier: ActionModifier::Normal,
            play_mode: PlayMode::Playing,
            ui_visible: true,
            is_fullscreen: false,
            camera: Box::from(OrbitCamera::default()),
            camera_pos: Vec3::default(),
            shader_error: None,
            scene_vars: Default::default(),
        }
    }
}

/// Bad naming but these are the values we can set from within skuggbox like colors
#[derive(Default)]
pub struct SceneVars {
    pub color_a: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActionModifier {
    SuperSlow, // shift + ctrl
    Slow,      // shift
    Normal,
    Fast, // ctrl
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

pub fn seek(
    playback_time: f32,
    modifier: &ActionModifier,
    playback_control: PlaybackControl,
) -> f32 {
    let factor = match modifier {
        ActionModifier::SuperSlow => 0.03125,
        ActionModifier::Slow => 0.125,
        ActionModifier::Normal => 1.0,
        ActionModifier::Fast => 8.0,
    };

    match playback_control {
        PlaybackControl::Forward(t) => playback_time + t * factor,
        PlaybackControl::Rewind(t) => f32::max(playback_time - t * factor, 0.0),
        PlaybackControl::Stop => 0.0,
    }
}
