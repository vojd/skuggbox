#![warn(clippy::all, future_incompatible, nonstandard_style, rust_2018_idioms)]

pub mod actions;
pub mod app;

pub mod config;
pub mod event;
pub mod input;
pub mod macros;
pub mod minime;
pub mod mouse;
pub mod render;
pub mod shader;
pub mod state;
pub mod timer;
pub mod utils;
pub mod window;

pub use actions::*;
pub use app::*;
pub use buffer::*;
pub use config::*;
pub use event::*;
pub use input::*;
pub use macros::*;
pub use minime::*;
pub use mouse::*;
pub use render::*;
pub use shader::*;
pub use state::*;
pub use timer::*;
pub use uniforms::*;
pub use utils::*;
pub use window::*;
