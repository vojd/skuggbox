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

/// Check for OpenGL error and report it using `tracing::error`.
///
/// WARNING: slow! Only use during setup!
///
/// ``` no_run
/// # let glow_context = todo!();
/// use egui_glow::check_for_gl_error_even_in_release;
/// check_for_gl_error_even_in_release!(glow_context);
/// check_for_gl_error_even_in_release!(glow_context, "during painting");
/// ```
#[macro_export]
macro_rules! check_for_gl_error_even_in_release {
    ($gl: expr) => {{
        $crate::check_for_gl_error_impl($gl, file!(), line!(), "")
    }};
    ($gl: expr, $context: literal) => {{
        $crate::check_for_gl_error_impl($gl, file!(), line!(), $context)
    }};
}

/// Profiling macro for feature "puffin"
macro_rules! profile_function {
    ($($arg: tt)*) => {
        #[cfg(feature = "puffin")]
        #[cfg(not(target_arch = "wasm32"))]
        puffin::profile_function!($($arg)*);
    };
}
pub(crate) use profile_function;

/// Profiling macro for feature "puffin"
macro_rules! profile_scope {
    ($($arg: tt)*) => {
        #[cfg(feature = "puffin")]
        #[cfg(not(target_arch = "wasm32"))]
        puffin::profile_scope!($($arg)*);
    };
}
pub(crate) use profile_scope;
