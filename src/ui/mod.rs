//! [`egui`] bindings for [`glow`](https://github.com/grovesNL/glow).
//!
//! The main types you want to look are are [`Painter`] and [`EguiGlow`].
//!
//! If you are writing an app, you may want to look at [`eframe`](https://docs.rs/eframe) instead.
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!

#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]

mod window;

pub use painter::*;
pub use window::*;

pub mod painter;
pub use glow;
pub use painter::{CallbackFn, Painter};
mod misc_util;
mod post_process;
mod shader_version;
pub mod vao;
