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

use crate::buffer::gen_array_buffer;
use crate::shader::ShaderService;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;

mod buffer;
mod shader;
mod utils;

#[allow(unused_macros)]
macro_rules! glchk {
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
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Metazoa rs");

    let window_context = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    let context = unsafe { window_context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let mut state = State {
        width: 1092,
        height: 768,
        is_running: true,
    };

    // shader compiler channel
    let (tx, rx) = channel();

    let mut shaders = ShaderService::new(
        "shaders".to_string(),
        "base.vert".to_string(),
        "base.frag".to_string(),
    );

    let _ = thread::spawn(move || {
        glsl_watcher::watch(tx, "shaders", "base.vert", "base.frag");
    });

    let vertices: Vec<f32> = vec![
        1.0, 1.0, 0.0, // 0
        -1.0, 1.0, 0.0, // 1
        1.0, -1.0, 0.0, // 2
        -1.0, -1.0, 0.0, // 3
    ];

    let vao = gen_array_buffer(&vertices);

    while state.is_running {
        if rx.try_recv().is_ok() {
            shaders.reload();
        }

        event_loop.run_return(|event, _, control_flow| {
            handle_events(event, control_flow, &mut state, &shaders, &context, vao);
        });

        render(&context, &state, &shaders, vao);
    }
}

fn handle_events<T>(
    event: Event<'_, T>,
    control_flow: &mut ControlFlow,
    state: &mut State,
    shaders: &ShaderService,
    context: &ContextWrapper<PossiblyCurrent, Window>,
    vao: gl::types::GLuint,
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
            *control_flow = ControlFlow::Exit
        }

        Event::WindowEvent { event, .. } => {
            match event {
                // WindowEvent::CloseRequested => *is_running = false,
                WindowEvent::Resized(size) => {
                    let size = size.to_logical::<i32>(1.0);
                    // bind size
                    state.width = size.width;
                    state.height = size.height;
                }
                _ => {}
            }
        }

        Event::MainEventsCleared => {
            if let Some(program) = &shaders.program {
                unsafe { gl::UseProgram(program.id) };
            } else {
                unsafe { gl::UseProgram(0) };
            }

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);

                gl::BindVertexArray(vao);
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            }

            unsafe { gl::UseProgram(0) };
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
    vao: gl::types::GLuint,
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

        gl::BindVertexArray(vao);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    }

    unsafe { gl::UseProgram(0) };
    context.swap_buffers().unwrap();
}
