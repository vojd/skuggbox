extern crate gl;
extern crate glutin;
extern crate winit;

use glutin::ContextBuilder;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new();
    let gl_ctx = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();
    let gl_window = unsafe { gl_ctx.make_current().unwrap() };

    gl::load_with(|s| gl_window.get_proc_address(s) as *const _);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Bye now...");
                *control_flow = ControlFlow::Exit
            }
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.2, 0.5, 0.2, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                gl_window.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
