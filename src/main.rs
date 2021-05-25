extern crate gl;
extern crate glutin;
extern crate winit;

use std::sync::mpsc::channel;
use std::thread;

use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::buffer::{Buffer, BufferType};
use crate::shader::ShaderService;
use simple_logger::SimpleLogger;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;

mod buffer;
mod shader;
mod uniforms;
mod utils;

#[allow(unused_macros)]
macro_rules! gl_error {
    ($($s:stmt;)*) => {
        $(
            $s;
            if cfg!(debug_assertions) {
                let err = gl::GetError();
                if err != gl::NO_ERROR {
                    let err_str = match err {
                        gl::INVALID_ENUM => "GL_INVALID_ENUM",
                        gl::INVALID_VALUE => "GL_INVALID_VALUE",
                        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
                        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                        gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
                        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
                        _ => "unknown error"
                    };
                    println!("{}:{} - {} caused {}",
                             file!(),
                             line!(),
                             stringify!($s),
                             err_str);
                }
            }
        )*
    }
}

struct State {
    width: i32,
    height: i32,
    is_running: bool,
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Skuggbox rs");

    let window_context = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    let context = unsafe { window_context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let mut state = State {
        width: 1024,
        height: 768,
        is_running: true,
    };

    // shader compiler channel
    let (sender, receiver) = channel();

    let mut shaders = ShaderService::new(
        "shaders".to_string(),
        "base.vert".to_string(),
        "base.frag".to_string(),
    );

    let _ = thread::spawn(move || {
        glsl_watcher::watch(sender, "shaders", "base.vert", "base.frag");
    });

    let vertices: Vec<f32> = vec![
        1.0, 1.0, 0.0, // 0
        -1.0, 1.0, 0.0, // 1
        1.0, -1.0, 0.0, // 2
        -1.0, -1.0, 0.0, // 3
    ];

    let vertex_buffer = Buffer::vertex_buffer(BufferType::VertexBuffer, &vertices);

    while state.is_running {
        if receiver.try_recv().is_ok() {
            shaders.reload();
        }

        event_loop.run_return(|event, _, control_flow| {
            handle_events(
                event,
                control_flow,
                &mut state,
                &shaders,
                &context,
                &vertex_buffer,
            );
        });

        render(&context, &state, &shaders, &vertex_buffer);
    }
}

fn handle_events<T>(
    event: Event<'_, T>,
    control_flow: &mut ControlFlow,
    state: &mut State,
    shaders: &ShaderService,
    context: &ContextWrapper<PossiblyCurrent, Window>,
    buffer: &Buffer,
) {
    *control_flow = ControlFlow::Poll;
    *control_flow = ControlFlow::Wait;

    context.swap_buffers().unwrap();

    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            println!("Bye now...");
            state.is_running = false;
            buffer.delete();
            *control_flow = ControlFlow::Exit
        }

        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            let size = size.to_logical::<i32>(1.0);
            // bind size
            state.width = size.width;
            state.height = size.height;
        }

        Event::MainEventsCleared => {
            unsafe {
                if !&shaders.program.is_some() {
                    eprintln!("Found no shader program");
                    return;
                }

                let program = shaders.program.as_ref().unwrap();

                gl::UseProgram(program.id);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                buffer.bind();
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

                gl::UseProgram(0);
            }

            context.swap_buffers().unwrap();

            *control_flow = ControlFlow::Exit;
        }

        Event::RedrawRequested(_) => {}
        _ => (),
    }
}

fn render(
    context: &ContextWrapper<PossiblyCurrent, Window>,
    state: &State,
    shaders: &ShaderService,
    buffer: &Buffer,
) {
    unsafe {
        gl::Viewport(0, 0, state.width, state.height);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    if let Some(program) = &shaders.program {
        unsafe { gl::UseProgram(program.id) };
    } else {
        unsafe { gl::UseProgram(0) };
    }

    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
        buffer.bind();
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    }

    unsafe { gl::UseProgram(0) };
    context.swap_buffers().unwrap();
}
