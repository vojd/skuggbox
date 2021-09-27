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

use crate::buffer::Buffer;
use crate::shader::{ShaderProgram, ShaderService};
use simple_logger::SimpleLogger;
use winit::event::{ElementState, VirtualKeyCode};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;

use crate::state::{seek, PlayMode, PlaybackControl};
use crate::timer::Timer;
use glam::{Vec2, Vec3};
use glutin::event::{MouseButton, MouseScrollDelta};
use log::debug;

mod buffer;
mod shader;
mod state;
mod timer;
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

#[derive(Debug)]
struct Camera {
    pos: Vec3,
    target: Vec3,
    speed: f32,
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vec3::new(0.0, 1.0, -1.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            speed: 1.0,
            zoom: 0.0,
        }
    }
}

#[derive(Debug)]
struct WorldState {
    width: i32,
    height: i32,
    /// App state - is the application running?
    is_running: bool,

    delta_time: f32,
    playback_time: f32,

    mouse_pos: Vec2,
    mouse_delta: Vec2,
    zoom_level: f32,

    /// Running or paused?
    play_mode: PlayMode,

    is_lmb_down: bool,
    is_rmb_down: bool,
    camera: Camera,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let mut timer = Timer::new();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Skuggbox rs");

    let window_context = ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    let context = unsafe { window_context.make_current().unwrap() };

    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let mut state = WorldState {
        width: 1024,
        height: 768,
        is_running: true,
        delta_time: 0.0,
        playback_time: 0.0,
        mouse_pos: Vec2::new(0.0, 0.0),
        mouse_delta: Default::default(),
        zoom_level: 0.0,
        play_mode: PlayMode::Playing,
        is_lmb_down: false,
        is_rmb_down: false,
        camera: Camera::default(),
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

    let vertex_buffer = Buffer::new_vertex_buffer();

    while state.is_running {
        if receiver.try_recv().is_ok() {
            shaders.reload();
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
            );
        });

        render(&context, &mut state, &shaders, &vertex_buffer);

        timer.stop();
    }
}

#[allow(temporary_cstring_as_ptr)]
fn get_uniform_location(program: &ShaderProgram, uniform_name: &str) -> i32 {
    unsafe { gl::GetUniformLocation(program.id, CString::new(uniform_name).unwrap().as_ptr()) }
}

fn handle_events<T>(
    event: Event<'_, T>,
    control_flow: &mut ControlFlow,
    world_state: &mut WorldState,
    timer: &mut Timer,
    context: &ContextWrapper<PossiblyCurrent, Window>,
    buffer: &Buffer,
) {
    *control_flow = ControlFlow::Poll;
    context.swap_buffers().unwrap();

    match event {
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::CloseRequested => {
                    println!("Bye now...");
                    world_state.is_running = false;
                    buffer.delete();
                    *control_flow = ControlFlow::Exit
                }

                WindowEvent::Resized(size) => {
                    let size = size.to_logical::<i32>(1.0);
                    // bind size
                    world_state.width = size.width;
                    world_state.height = size.height;
                }

                WindowEvent::CursorMoved { position, .. } => {
                    if world_state.is_rmb_down {
                        world_state.mouse_delta = Vec2::new(
                            position.x as f32 - world_state.mouse_pos.x,
                            world_state.mouse_pos.y - position.y as f32,
                        );
                    }
                }

                WindowEvent::MouseInput { button, state, .. } => {
                    world_state.is_lmb_down =
                        button == MouseButton::Left && state == ElementState::Pressed;
                    world_state.is_rmb_down =
                        button == MouseButton::Right && state == ElementState::Pressed;
                    debug!(
                        "l: {:?}, r: {:?}",
                        world_state.is_lmb_down, world_state.is_rmb_down
                    );
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    if let MouseScrollDelta::LineDelta(_x, y) = delta {
                        world_state.camera.zoom += y;
                    }
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {
                            if keycode == VirtualKeyCode::Space {
                                debug!("toggle playmode");
                                world_state.play_mode = match world_state.play_mode {
                                    PlayMode::Playing => PlayMode::Paused,
                                    PlayMode::Paused => {
                                        timer.start();
                                        PlayMode::Playing
                                    }
                                }
                            }

                            if keycode == VirtualKeyCode::Right {
                                world_state.playback_time =
                                    seek(world_state.playback_time, PlaybackControl::Forward(1.0))
                            }

                            if keycode == VirtualKeyCode::Left {
                                world_state.playback_time =
                                    seek(world_state.playback_time, PlaybackControl::Rewind(1.0))
                            }

                            if keycode == VirtualKeyCode::Key0 {
                                world_state.playback_time = 0.0;
                            }

                            if keycode == VirtualKeyCode::A {
                                world_state.camera.pos.x -= 0.5;
                            }

                            if keycode == VirtualKeyCode::D {
                                world_state.camera.pos.x += 0.5;
                            }

                            if keycode == VirtualKeyCode::W {
                                world_state.camera.pos.y -= 0.5;
                            }

                            if keycode == VirtualKeyCode::S {
                                world_state.camera.pos.y += 0.5;
                            }

                            if keycode == VirtualKeyCode::Period {
                                // reset all camera settings
                                world_state.camera = Camera::default();
                                world_state.mouse_delta = Vec2::new(0.0, 0.0);
                            }
                        }
                    }

                    if let Some(keycode) = input.virtual_keycode {
                        debug!("pressed {:?}", keycode)
                    }
                }

                _ => {}
            }
        }

        Event::MainEventsCleared => {
            if matches!(world_state.play_mode, PlayMode::Playing) {
                world_state.playback_time += world_state.delta_time;
            }

            *control_flow = ControlFlow::Exit;
        }

        Event::RedrawRequested(_) => {}
        _ => (),
    }
}

fn render(
    context: &ContextWrapper<PossiblyCurrent, Window>,
    state: &mut WorldState,
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

        // push uniform values to shader

        let location = get_uniform_location(program, "iTime");
        gl::Uniform1f(location, state.playback_time);

        let location = get_uniform_location(program, "iResolution");
        gl::Uniform2f(location, state.width as f32, state.height as f32);

        // NOTE: Passing in the delta for now as it's used for cam control in the shader
        let location = get_uniform_location(program, "iMouse");
        gl::Uniform3f(
            location,
            state.mouse_delta.x,
            state.mouse_delta.y,
            state.camera.zoom,
        );

        let location = get_uniform_location(program, "camPos");
        gl::Uniform3f(
            location,
            state.camera.pos.x,
            state.camera.pos.y,
            state.camera.pos.z,
        );

        gl::Clear(gl::COLOR_BUFFER_BIT);
        buffer.bind();
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

        gl::UseProgram(0);
    }

    unsafe { gl::UseProgram(0) };
    context.swap_buffers().unwrap();
}
