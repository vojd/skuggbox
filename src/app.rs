use glow::HasContext;
use glow::VertexArray;
use std::sync::Arc;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;

use crate::{
    handle_actions, handle_events, top_bar, Action, AppConfig, AppState, AppWindow, PlayMode,
    ShaderService,
};
use ui_backend::Ui;

pub struct App {
    pub event_loop: EventLoop<()>,
    pub app_window: AppWindow,
    pub app_state: AppState,
    pub ui: Option<Ui>,
    pub gl: Option<Arc<glow::Context>>,
}

impl App {
    pub fn from_config(config: AppConfig) -> Self {
        let app_state = AppState::default();
        let (app_window, event_loop) = AppWindow::new(config, &app_state);
        let ui = None;
        Self {
            event_loop,
            app_window,
            app_state,
            ui,
            gl: None,
        }
    }

    pub fn run(&mut self, config: AppConfig) {
        let App {
            event_loop,
            app_window,
            app_state,
            gl: _,
            ui: _,
        } = self;

        let mut actions: Vec<Action> = vec![];

        let gl = app_window.create_window_context();
        let mut ui = Ui::new(event_loop, gl.clone());

        let shader_files = config.files.unwrap();
        log::debug!("Shader files: {:?}", shader_files);
        let mut shader_service = ShaderService::new(gl.clone(), shader_files);
        shader_service.watch();
        let _ = shader_service.run(gl.as_ref());

        let vertex_array = unsafe {
            gl.create_vertex_array()
                .expect("Cannot create vertex array")
        };

        log::debug!("EventLoop: Starting");

        while app_state.is_running {
            let _ = shader_service.run(gl.as_ref());
            app_state.shader_error = shader_service.last_error.clone();

            // force UI open if we have a shader error
            if app_state.shader_error.is_some() {
                app_state.ui_visible = true;
            }

            if matches!(app_state.play_mode, PlayMode::Playing) {
                app_state.timer.start();
                // TODO(mathias): Remove this. Only use `app_state.timer.delta_time`
                app_state.delta_time = app_state.timer.delta_time;
            }

            event_loop.run_return(|event, _window_target, control_flow| {
                *control_flow = ControlFlow::Wait;

                // TODO: No unwrap on the window object

                let _repaint_after = ui.run(app_window.window.as_ref().unwrap(), |egui_ctx| {
                    egui::TopBottomPanel::top("view_top").show(egui_ctx, |ui| {
                        top_bar(ui, app_state, &mut actions, &shader_service);
                    });

                    if let Some(error) = &app_state.shader_error {
                        let mut error = format!("{}", error);
                        egui::TopBottomPanel::bottom("view_bottom").show(egui_ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut error)
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_rows(4)
                                        .desired_width(f32::INFINITY)
                                        .lock_focus(true),
                                )
                            });
                        });
                    }
                });

                handle_events(&event, control_flow, &mut ui, app_state, &mut actions);

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

            if let Some(mouse_dir) = shader.locations.mouse_dir {
                gl.uniform_3_f32(
                    Some(&mouse_dir),
                    state.mouse.dir.x,
                    state.mouse.dir.y,
                    state.mouse.dir.z,
                );
            }

            if let Some(sb_camera_transform) = shader.locations.sb_camera_transform {
                let camera = state.camera.calculate_uniform_data();
                let f32_arr = camera.to_cols_array();
                gl.uniform_matrix_4_f32_slice(Some(&sb_camera_transform), false, &f32_arr);
            }

            if let Some(cam_pos) = shader.locations.cam_pos {
                let pos = state.camera_pos;
                gl.uniform_3_f32(Some(&cam_pos), pos.x, pos.y, pos.z);
            }

            if let Some(sb_color_a) = shader.locations.sb_color_a {
                let col = state.scene_vars.color_a;
                gl.uniform_3_f32(Some(&sb_color_a), col[0], col[1], col[2]);
            }

            // actually render
            gl.clear(glow::COLOR_BUFFER_BIT);
            macros::check_for_gl_error!(&gl, "clear");
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 3);
            macros::check_for_gl_error!(&gl, "draw_arrays");

            if state.ui_visible {
                if let Some(window) = &app_window.window {
                    egui_glow.paint(window);
                }
            }
        }
    }

    app_window.swap_buffers();
}
