extern crate gl;
extern crate glutin;
extern crate winit;

use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::channel;
use std::thread;

use clap::Parser;
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
    get_uniform_location, handle_events,
    shader::ShaderService,
    state::{AppState, PlayMode},
    timer::Timer,
};

fn create_new_shader(path: PathBuf) -> std::io::Result<u64> {
    if path.exists() {
        log::warn!("File already exists. Not overwriting it to save your life :D");
        exit(1);
    }
    fs::copy("shaders/init.glsl", path)
}

fn main() {
    SimpleLogger::new().init().unwrap();

    // Parse command line arguments using `structopt`
    let config = Config::parse();
    if let Some(new_file) = config.clone().new {
        log::info!("creating new shader at {:?}", new_file);
        match create_new_shader(new_file) {
            Ok(_) => log::info!("Done"),
            Err(err) => {
                log::error!("{:?}", err);
                exit(1);
            }
        }
    }

    if config.file.is_some() {
        log::info!("loading existing shader");
        run(config);
    }
}

fn run(config: Config) {
    assert!(config.file.is_some());

    // verify that all specified file does exist
    let mut timer = Timer::new();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Skuggbox")
        .with_always_on_top(config.always_on_top);

    let window_context = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    let context = unsafe { window_context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let mut state = AppState::default();

    // shader compiler channel
    let (sender, receiver) = channel();
    let shader_file = config.file.unwrap();
    let mut shader = ShaderService::new(shader_file);

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
            // let location = get_uniform_location(program, "iResolution");
            gl::Uniform2f(
                shaders.locations.resolution,
                state.width as f32,
                state.height as f32,
            );

            // let location = get_uniform_location(program, "iTime");
            gl::Uniform1f(shaders.locations.time, state.playback_time);
            // let location = get_uniform_location(program, "iTimeDelta");
            gl::Uniform1f(shaders.locations.time_delta, state.delta_time);

            // push mouse location to the shader
            // let location = get_uniform_location(program, "iMouse");
            gl::Uniform4f(
                shaders.locations.mouse,
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
}
