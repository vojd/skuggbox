use winit::event_loop::EventLoop;
use winit::platform::run_return::EventLoopExtRunReturn;

use crate::{
    handle_events, AppState, AppWindow, Buffer, Config, PlayMode, ShaderService, SkuggboxShader,
    Timer,
};

pub struct App {
    pub event_loop: EventLoop<()>,
    pub app_window: AppWindow,
    pub state: AppState,
}

impl App {
    pub fn from_config(config: Config) -> Self {
        let (app_window, event_loop) = AppWindow::new(config);
        let state = crate::AppState::default();

        Self {
            event_loop,
            app_window,
            state,
        }
    }

    pub fn run(&mut self, config: Config) -> anyhow::Result<(), anyhow::Error> {
        let App {
            event_loop,
            app_window,
            state,
        } = self;

        let mut timer = Timer::new();

        let shader_files = config.files.unwrap();

        let mut shader_service =
            ShaderService::new(shader_files).expect("Could not compile initial shaders");
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
                        &app_window.window_context,
                        &vertex_buffer,
                        &mut shader_service,
                    );
                });
            }

            if let Some(skuggbox_shaders) = &shader_service.skuggbox_shaders {
                render(app_window, state, skuggbox_shaders, &vertex_buffer);
            }

            timer.stop();
        }

        Ok(())
    }
}

fn render(
    app_window: &mut AppWindow,
    // window_context: &ContextWrapper<PossiblyCurrent, Window>,
    state: &mut AppState,
    skuggbox_shaders: &[SkuggboxShader],
    buffer: &Buffer,
) {
    unsafe {
        gl::Viewport(0, 0, state.width, state.height);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    // TODO: This only pulls the first one at the moment until we have multi-buffer support
    let shader = skuggbox_shaders.get(0).unwrap();
    let program = &shader.shader_program;

    unsafe {
        gl::UseProgram(program.id);

        // viewport resolution in pixels
        gl::Uniform2f(
            shader.locations.resolution,
            state.width as f32,
            state.height as f32,
        );

        gl::Uniform1f(shader.locations.time, state.playback_time);
        gl::Uniform1f(shader.locations.time_delta, state.delta_time);

        // push mouse location to the shader
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

        let location = program.uniform_location("sbCameraPosition");
        gl::Uniform3f(location, position.x, position.y, position.z);

        let location = program.uniform_location("sbCameraTransform");
        gl::UniformMatrix4fv(location, 1, gl::FALSE, &transform.to_cols_array()[0]);

        gl::Clear(gl::COLOR_BUFFER_BIT);
        buffer.bind();
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

        gl::UseProgram(0);
    }

    unsafe { gl::UseProgram(0) };
    app_window.window_context.swap_buffers().unwrap();
}
