use glow::Context;
use glutin::PossiblyCurrent;
use winit::event_loop::EventLoop;

use crate::{AppState, Config};

pub struct AppWindow {
    pub window_context: glutin::WindowedContext<PossiblyCurrent>,
}

impl AppWindow {
    pub fn new(config: Config, app_state: &AppState) -> (Self, Context, EventLoop<()>) {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Skuggbox")
            .with_always_on_top(config.always_on_top)
            .with_inner_size(glutin::dpi::LogicalSize::new(
                app_state.width,
                app_state.height,
            ));

        let window_context = unsafe {
            match glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
            {
                Ok(ctx) => ctx.make_current().unwrap(),
                Err(creation_error) => {
                    eprintln!("Context creation failed with error: {}", creation_error);
                    panic!("Error creating context")
                }
            }
        };
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window_context.get_proc_address(s) as *const _)
        };

        (Self { window_context }, gl, event_loop)
    }
}
