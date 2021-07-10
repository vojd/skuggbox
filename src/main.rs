extern crate gl;
extern crate glutin;
extern crate winit;
use std::ffi::CString;
use std::sync::mpsc::channel;
use std::thread;

use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::buffer::{Buffer, BufferType};
use crate::shader::{ShaderService, ShaderProgram};
use simple_logger::SimpleLogger;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;
use winit::event::{ScanCode, ElementState};

use log::info;

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

#[derive(Default)]
struct State {
    width: i32,
    height: i32,
    is_running: bool,

    mouse_x: i32,
    mouse_y: i32,

    mouse_left_pressed: bool,
    mouse_right_pressed: bool,

    dev_mode: bool,
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
        mouse_x: 0,
        mouse_y: 0,
        mouse_left_pressed: false,
        mouse_right_pressed: false,
        dev_mode: false,
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

#[allow(temporary_cstring_as_ptr)]
fn get_uniform_location(program: &ShaderProgram, uniform_name: &str) -> i32 {
    unsafe {
        gl::GetUniformLocation(program.id, CString::new(uniform_name).unwrap().as_ptr())
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

        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::CloseRequested => {
                    println!("Bye now...");
                    state.is_running = false;
                    buffer.delete();
                    *control_flow = ControlFlow::Exit
                }

                WindowEvent::Resized(size) => {
                    let size = size.to_logical::<i32>(1.0);
                    // bind size
                    state.width = size.width;
                    state.height = size.height;
                }

                WindowEvent::Moved(pos) => {
                    state.mouse_x = pos.x;
                    state.mouse_y = pos.y;
                }

                WindowEvent::MouseInput {button, state, ..} => {
                    info!("mouse input {:?} {:?}", button, state);
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {

                        }

                    }
                    if let Some(keycode) = input.virtual_keycode {
                        info!("pressed {:?}", keycode)


                    }
                }

                _ => {}
            }
        }

        Event::MainEventsCleared => {
            unsafe {
                if !&shaders.program.is_some() {
                    eprintln!("Found no shader program");
                    return;
                }

                let program = shaders.program.as_ref().unwrap();

                gl::UseProgram(program.id);

                // push uniform values to shader
                let location = get_uniform_location(program, "iTime");
                gl::Uniform1f(location, 0.0);

                let location = get_uniform_location(program, "iResolution");
                gl::Uniform2f(location, state.width as f32, state.height as f32);

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
