use std::sync::Arc;

use glow::{HasContext, VertexArray};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;

use crate::{
    handle_actions, handle_events, Action, AppState, AppWindow, Config, PlayMode, ShaderService, Ui,
};

pub struct App {
    pub event_loop: EventLoop<()>,
    pub app_window: AppWindow,
    pub app_state: AppState,
    pub gl: Arc<glow::Context>,
}

impl App {
    pub fn from_config(config: Config) -> Self {
        let app_state = AppState::default();
        let (app_window, gl, event_loop) = AppWindow::new(config, &app_state);

        Self {
            event_loop,
            app_window,
            app_state,
            gl: Arc::new(gl),
        }
    }

    pub fn run(&mut self, config: Config) {
        let App {
            event_loop,
            app_window,
            app_state,
            gl,
        } = self;

        let mut ui = Ui::new(event_loop, gl.clone());

        let mut actions: Vec<Action> = vec![];

        let shader_files = config.files.unwrap();
        log::debug!("Shader files: {:?}", shader_files);
        let mut shader_service = ShaderService::new(gl.clone(), shader_files);
        shader_service.watch();
        shader_service.run(gl);

        let vertex_array = unsafe {
            gl.create_vertex_array()
                .expect("Cannot create vertex array")
        };

        while app_state.is_running {
            shader_service.run(gl);

            if matches!(app_state.play_mode, PlayMode::Playing) {
                app_state.timer.start();
                // TODO(mathias): Remove this. Only use `app_state.timer.delta_time`
                app_state.delta_time = app_state.timer.delta_time;
            }

            event_loop.run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Wait;

                let _repaint_after = ui.run(app_window.window_context.window(), |egui_ctx| {
                    egui::TopBottomPanel::top("view_top").show(egui_ctx, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("⏹").clicked() {
                                actions.push(Action::TogglePlayPause);
                                actions.push(Action::TimeStop);
                            }
                            // rewind
                            if ui.button("⏪").clicked() {
                                actions.push(Action::TimeRewind(1.0))
                            }
                            // play/pause
                            let play_mode_label = match app_state.play_mode {
                                PlayMode::Playing => "⏸",
                                PlayMode::Paused => "▶",
                            };
                            if ui.button(play_mode_label).clicked() {
                                actions.push(Action::TogglePlayPause)
                            }

                            // fast forward
                            if ui.button("⏩").clicked() {
                                actions.push(Action::TimeForward(1.0))
                            }
                        });
                    });
                });

                handle_events(
                    &event,
                    control_flow,
                    &mut ui,
                    app_state,
                    // &app_window.window_context,
                    &mut actions,
                );

                handle_actions(&mut actions, app_state, &mut shader_service, control_flow);
            });

            render(
                gl.clone(),
                vertex_array,
                &mut ui,
                app_window,
                app_state,
                &shader_service,
            );

            app_state.timer.stop();
        }
    }
}

fn render(
    gl: Arc<glow::Context>,
    vertex_array: VertexArray,
    egui_glow: &mut Ui,
    app_window: &AppWindow,
    state: &mut AppState,
    shader_service: &ShaderService,
) {
    unsafe {
        gl.bind_vertex_array(Some(vertex_array));

        gl.clear_color(0.1, 0.2, 0.1, 1.0);

        if let Some(shader) = shader_service.shaders.get(0) {
            // kick shader to gpu
            gl.use_program(shader.program);

            // set uniforms
            if let Some(resolution) = shader.locations.resolution {
                gl.uniform_2_f32(Some(&resolution), state.width as f32, state.height as f32)
            }

            if let Some(time) = shader.locations.time {
                gl.uniform_1_f32(Some(&time), state.playback_time)
            }

            if let Some(delta_time) = shader.locations.time_delta {
                gl.uniform_1_f32(Some(&delta_time), state.delta_time)
            }

            // Mouse uniforms
            if let Some(mouse) = shader.locations.mouse {
                let x = state.mouse.pos.x;
                let y = state.mouse.pos.y;

                let left_mouse = if state.mouse.is_lmb_down { 1.0 } else { 0.0 };
                let right_mouse = if state.mouse.is_rmb_down { 1.0 } else { 0.0 };

                gl.uniform_4_f32(Some(&mouse), x, y, left_mouse, right_mouse);
            };

            // actually render
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 3);

            if state.ui_visible {
                egui_glow.paint(app_window.window_context.window());
            }
        }
    }
    app_window.window_context.swap_buffers().unwrap();
}
