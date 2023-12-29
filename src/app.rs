use std::sync::Arc;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;

use ui_backend::UiWindow;

use crate::renderer::Renderer;
use crate::{
    handle_actions, handle_events, Action, AppConfig, AppState, AppWindow, PlayMode, ShaderService,
    SkuggboxUi,
};

pub struct App {
    pub event_loop: EventLoop<()>,
    pub app_window: AppWindow,
    pub app_state: AppState,
    pub ui_window: Option<UiWindow>,
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
            ui_window: ui,
            gl: None,
        }
    }

    pub fn run(&mut self, config: AppConfig) {
        let App {
            event_loop,
            app_window,
            app_state,
            gl: _,
            ui_window: _,
        } = self;

        let mut actions: Vec<Action> = vec![];

        let gl = app_window.create_window_context();
        let mut ui_window = UiWindow::new(event_loop, gl.clone());
        let mut app_ui = SkuggboxUi::new();

        let shader_files = config.files.unwrap();
        log::debug!("Shader files: {:?}", shader_files);
        let mut shader_service = ShaderService::new(gl.clone(), shader_files);
        shader_service.watch();
        let _ = shader_service.run(gl.as_ref());

        let renderer = Renderer::new(gl.clone());

        log::debug!("MainLoop: Start");

        while app_state.is_running {
            let _ = shader_service.run(gl.as_ref());
            app_state.shader_error = shader_service.last_error.clone();

            // force UI open if we have a shader error
            if app_state.shader_error.is_some() {
                app_state.is_ui_visible = true;
            }

            if matches!(app_state.play_mode, PlayMode::Playing) {
                app_state.timer.start();
                // TODO(mathias): Remove this. Only use `app_state.timer.delta_time`
                app_state.delta_time = app_state.timer.delta_time;
            }

            event_loop.run_return(|event, _window_target, control_flow| {
                *control_flow = ControlFlow::Wait;

                // TODO: No unwrap on the window object

                let _repaint_after =
                    ui_window.run(app_window.window.as_ref().unwrap(), |egui_ctx| {
                        egui::TopBottomPanel::top("view_top").show(egui_ctx, |ui| {
                            app_ui.top_bar(ui, app_state, &mut actions, &shader_service);
                        });

                        if let Some(error) = &app_state.shader_error {
                            let error = format!("{}", error);
                            egui::TopBottomPanel::bottom("view_bottom").show(egui_ctx, |ui| {
                                app_ui.shader_error_panel(ui, error);
                            });
                        }
                    });

                handle_events(
                    &event,
                    control_flow,
                    &mut ui_window,
                    app_state,
                    &mut actions,
                );

                handle_actions(&mut actions, app_state, &mut shader_service, control_flow);
            });

            // Render the OpenGL scene
            renderer.draw(app_state, &shader_service);

            // Render UI on top of OpenGL scene
            if app_state.is_ui_visible && app_window.window.is_some() {
                if let Some(window) = &app_window.window {
                    ui_window.paint(window);
                }
            }

            app_window.swap_buffers();

            app_state.timer.stop();
        }

        log::debug!("MainLoop: Exit");
    }
}
