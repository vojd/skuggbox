use crate::{Config, EguiGlow};
use glow::Context;
use glutin::PossiblyCurrent;
use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct AppWindow {
    // pub window_context: ContextWrapper<PossiblyCurrent, Window>,
    pub window_context: glutin::WindowedContext<PossiblyCurrent>,
}

impl AppWindow {
    pub fn new(config: Config) -> (Self, Arc<Context>, EventLoop<()>) {
        let event_loop = glutin::event_loop::EventLoopBuilder::with_user_event().build();
        let (gl_window, gl) = create_display(&event_loop);

        (
            Self {
                window_context: gl_window,
            },
            std::sync::Arc::new(gl),
            event_loop,
        )
    }

    pub fn new_old(config: Config) -> (Self, std::sync::Arc<glow::Context>, EventLoop<()>) {
        let event_loop = EventLoop::new();

        let window_builder = WindowBuilder::new()
            .with_title("Skuggbox")
            .with_always_on_top(config.always_on_top);

        // Using the ContextBuilder instead of window_builder.build() to get an OpenGL context
        let window_context = glutin::ContextBuilder::new()
            .build_windowed(window_builder, &event_loop)
            .unwrap();

        // Load the OpenGL context
        let window_context = unsafe { window_context.make_current().unwrap() };
        gl::load_with(|s| window_context.get_proc_address(s) as *const _);

        let gl_context =
            unsafe { glow::Context::from_loader_function(|s| window_context.get_proc_address(s)) };

        (
            Self { window_context },
            std::sync::Arc::new(gl_context),
            event_loop,
        )
    }
}

fn create_display(
    event_loop: &glutin::event_loop::EventLoop<()>,
) -> (
    glutin::WindowedContext<glutin::PossiblyCurrent>,
    glow::Context,
) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("Skuggbox");

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };
    (gl_window, gl)
}
