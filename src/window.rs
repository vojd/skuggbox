use crate::Config;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct AppWindow {
    pub window_context: ContextWrapper<PossiblyCurrent, Window>,
}

impl AppWindow {
    pub fn new(config: Config) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();

        let window_builder = WindowBuilder::new()
            .with_title("Skuggbox")
            .with_always_on_top(config.always_on_top);

        // Using the ContextBuilder instead of window_builder.build() to get an OpenGL context
        let window_context = ContextBuilder::new()
            .build_windowed(window_builder, &event_loop)
            .unwrap();

        // Load the OpenGL context
        let window_context = unsafe { window_context.make_current().unwrap() };
        gl::load_with(|s| window_context.get_proc_address(s) as *const _);

        (Self { window_context }, event_loop)
    }
}
