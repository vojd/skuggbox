use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use winit::event_loop::EventLoop;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

use crate::{
    get_uniform_location, handle_events, AppState, Buffer, Config, PlayMode, ShaderService, Timer,
};

pub struct App {
    pub event_loop: EventLoop<()>,
    pub window_context: ContextWrapper<PossiblyCurrent, Window>,
    pub state: AppState,
}

impl App {
    pub fn from_config(config: Config) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Skuggbox")
            .with_always_on_top(config.always_on_top);

        let window_context = ContextBuilder::new()
            .build_windowed(window, &event_loop)
            .unwrap();

        let window_context = unsafe { window_context.make_current().unwrap() };

        let state = crate::AppState::default();

        Self {
            event_loop,
            window_context,
            state,
        }
    }

    pub fn run(&mut self, config: Config) -> anyhow::Result<(), anyhow::Error> {
        let App {
            event_loop,
            window_context,
            state,
        } = self;

        let mut timer = Timer::new();

        gl::load_with(|s| window_context.get_proc_address(s) as *const _);
        let shader_files = config.files.unwrap();
        let mut shader_service = ShaderService::new(shader_files);

        shader_service.watch();
        let vertex_buffer = Buffer::new_vertex_buffer();

        while state.is_running {
            shader_service.run();

            if matches!(state.play_mode, PlayMode::Playing) {
                timer.start();
                state.delta_time = timer.delta_time;
            }

            {
                event_loop.run_return(|event, _, control_flow| {
                    handle_events(
                        &event,
                        control_flow,
                        state,
                        &mut timer,
                        window_context,
                        &vertex_buffer,
                        &mut shader_service,
                    );
                });
            }

            render(window_context, state, &shader_service, &vertex_buffer);

            timer.stop();
        }

        Ok(())
    }
}

fn render(
    window_context: &ContextWrapper<PossiblyCurrent, Window>,
    state: &mut AppState,
    shader_service: &ShaderService,
    buffer: &Buffer,
) {
    unsafe {
        gl::Viewport(0, 0, state.width, state.height);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    unsafe {
        // TODO: This only pulls the first one at the moment until we have multi-buffer support
        // let program = shader_service.skuggbox_shaders.get(0).unwrap();
        let shader = shader_service.skuggbox_shaders.get(0).unwrap();
        // let program = &shader_service.skuggbox_shaders.get(0).unwrap().shader_program;
        let program = &shader.shader_program;

        gl::UseProgram(program.id);

        // viewport resolution in pixels
        // let location = get_uniform_location(program, "iResolution");
        gl::Uniform2f(
            shader.locations.resolution,
            state.width as f32,
            state.height as f32,
        );

        // let location = get_uniform_location(program, "iTime");
        gl::Uniform1f(shader.locations.time, state.playback_time);
        // let location = get_uniform_location(program, "iTimeDelta");
        gl::Uniform1f(shader.locations.time_delta, state.delta_time);

        // push mouse location to the shader
        // let location = get_uniform_location(program, "iMouse");
        gl::Uniform4f(
            shader.locations.mouse,
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
    window_context.swap_buffers().unwrap();
}
