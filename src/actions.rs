use crate::{seek, AppState, OrbitCamera, PlayMode, PlaybackControl, ShaderService};
use glam::Vec2;
use winit::event_loop::ControlFlow;

pub enum Action {
    AppExit,
    TimePlay,
    TimePause,
    TogglePlayPause,
    TimeStop,
    TimeForward(f32),
    TimeRewind(f32),
    WindowClose,
    WindowResize((i32, i32)),
    // size
    CameraToggleIntegration(bool),
    CameraReset,
}

pub fn handle_actions(
    actions: &mut Vec<Action>,
    app_state: &mut AppState,
    shader_service: &mut ShaderService,
    control_flow: &mut ControlFlow,
) {
    for action in actions.drain(..) {
        match action {
            Action::AppExit => {
                log::info!("Bye now...");
                app_state.is_running = false;
                *control_flow = ControlFlow::Exit
            }
            Action::TimePlay => {
                app_state.timer.start();
                app_state.play_mode = PlayMode::Playing;
            }
            Action::TimePause => {
                app_state.play_mode = PlayMode::Paused;
            }
            Action::TimeStop => {
                app_state.playback_time = seek(app_state.playback_time, PlaybackControl::Stop)
            }
            Action::TogglePlayPause => match app_state.play_mode {
                PlayMode::Playing => {
                    app_state.play_mode = PlayMode::Paused;
                }
                PlayMode::Paused => {
                    app_state.timer.start();
                    app_state.play_mode = PlayMode::Playing;
                }
            },
            Action::TimeForward(time) => {
                app_state.playback_time =
                    seek(app_state.playback_time, PlaybackControl::Forward(time))
            }
            Action::TimeRewind(time) => {
                app_state.playback_time =
                    seek(app_state.playback_time, PlaybackControl::Rewind(time))
            }
            Action::WindowClose => {}
            Action::WindowResize((width, height)) => {
                app_state.width = width;
                app_state.height = height;
            }
            Action::CameraToggleIntegration(use_camera_integration) => match use_camera_integration
            {
                true => {
                    if !shader_service.use_camera_integration {
                        log::info!("Enabling camera integration. Please use '#pragma skuggbox(camera)' in your shader");
                        shader_service.use_camera_integration = true;
                        shader_service
                            .reload();
                            //.expect("Expected successful shader reload");
                    }
                }
                false => {
                    if shader_service.use_camera_integration {
                        log::info!("Disabling camera integration");
                        shader_service.use_camera_integration = false;
                        shader_service
                            .reload();
                            //.expect("Expected successful shader reload");
                    }
                }
            },
            Action::CameraReset => {
                app_state.camera = Box::from(OrbitCamera::default());
                app_state.mouse.delta = Vec2::new(0.0, 0.0);
            }
        }
    }
}
