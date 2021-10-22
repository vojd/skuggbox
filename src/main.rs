extern crate gl;
extern crate glutin;
extern crate winit;

use clap::Parser;
use std::ffi::CString;
use std::sync::mpsc::channel;
use std::thread;

use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

use simple_logger::SimpleLogger;
use winit::{
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

use skuggbox::{
    buffer::Buffer,
    config::Config,
    handle_events,
    shader::{ShaderProgram, ShaderService},
    state::{AppState, PlayMode},
    timer::Timer
};

fn main() {
    SimpleLogger::new().init().unwrap();

    // Parse command line arguments using `structopt`
    let config = Config::parse();

    // verify that all specified file does exist
    let mut timer = Timer::new();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Skuggbox");

    let window_context = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    let context = unsafe { window_context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let mut state = AppState::default();

    // shader compiler channel
    let (sender, receiver) = channel();

    let mut shader = ShaderService::new(config.file);

    // TODO: Ensure we only watch the files currently in the shader
    let files = shader.files.clone();
    let _ = thread::spawn(move || {
        glsl_watcher::watch_all(sender, files);
    });

    let vertex_buffer = Buffer::new_vertex_buffer();

    while state.is_running {
        if receiver.try_recv().is_ok() {
            shader.reload();
        }

        if matches!(state.play_mode, PlayMode::Playing) {
            timer.start();
            state.delta_time = timer.delta_time;
        }

        event_loop.run_return(|event, _, control_flow| {
            handle_events(
                event,
                control_flow,
                &mut state,
                &mut timer,
                &context,
                &vertex_buffer,
                &mut shader,
            );
        });

        render(&context, &mut state, &shader, &vertex_buffer);

        timer.stop();
    }
}

#[allow(temporary_cstring_as_ptr)]
fn get_uniform_location(program: &ShaderProgram, uniform_name: &str) -> i32 {
    unsafe { gl::GetUniformLocation(program.id, CString::new(uniform_name).unwrap().as_ptr()) }
}

fn render(
    context: &ContextWrapper<PossiblyCurrent, Window>,
    state: &mut AppState,
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
        let program = shaders.program.as_ref().unwrap();

        gl::UseProgram(program.id);

        // viewport resolution in pixels
        let location = get_uniform_location(program, "iResolution");
        gl::Uniform2f(location, state.width as f32, state.height as f32);

        let location = get_uniform_location(program, "iTime");
        gl::Uniform1f(location, state.playback_time);
        let location = get_uniform_location(program, "iTimeDelta");
        gl::Uniform1f(location, state.delta_time);

        // push mouse location to the shader
        let location = get_uniform_location(program, "iMouse");
        gl::Uniform4f(
            location,
            state.mouse.pos.x,
            state.mouse.pos.y,
            if state.mouse.is_lmb_down { 1.0 } else { 0.0 },
            if state.mouse.is_rmb_down { 1.0 } else { 0.0 },
        );

        // push the camera transform to the shader
        let transform = state.camera.calculate_uniform_data();
        let position = transform.w_axis;

        let location = get_uniform_location(program, "sbCameraPosition");
        gl::Uniform3f(location, position.x, position.y, position.z);

        let location = get_uniform_location(program, "sbCameraTransform");
        gl::UniformMatrix4fv(location, 1, gl::FALSE, &transform.to_cols_array()[0]);

        gl::Clear(gl::COLOR_BUFFER_BIT);
        buffer.bind();
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

        gl::UseProgram(0);
    }

    unsafe { gl::UseProgram(0) };
    context.swap_buffers().unwrap();
}
