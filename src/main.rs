extern crate gl;
extern crate glutin;
extern crate winit;

use glutin::ContextBuilder;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::buffer::gen_array_buffer;
use crate::opengl::{Shader, ShaderProgram};

mod buffer;
mod opengl;
mod utils;

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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Metazoa rs");

    let window_context = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    let context = unsafe { window_context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let vertex_shader =
        Shader::from_source("shaders/base.vert", gl::VERTEX_SHADER).expect("shader error");
    let frag_shader =
        Shader::from_source("shaders/base.frag", gl::FRAGMENT_SHADER).expect("shader error");

    let shader_program = ShaderProgram::new(vertex_shader, frag_shader);

    let vertices: Vec<f32> = vec![
        1.0, 1.0, 0.0, // 0
        -1.0, 1.0, 0.0, // 1
        1.0, -1.0, 0.0, // 2
        -1.0, -1.0, 0.0, // 3
    ];

    let vao = gen_array_buffer(&vertices);

    unsafe {
        gl::Viewport(0, 0, 900, 700);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

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
                shader_program.activate();

                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);

                    gl::BindVertexArray(vao);
                    gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                }

                context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
