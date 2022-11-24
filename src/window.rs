use glow::Context;
use glutin::PossiblyCurrent;
use winit::event_loop::EventLoop;

use crate::Config;

pub struct AppWindow {
    pub window_context: glutin::WindowedContext<PossiblyCurrent>,
}

impl AppWindow {
    pub fn new(config: Config) -> (Self, Context, EventLoop<()>) {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Skuggbox")
            .with_always_on_top(config.always_on_top)
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

        let window_context = unsafe {
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap()
        };
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window_context.get_proc_address(s) as *const _)
        };

        (Self { window_context }, gl, event_loop)
    }
}
