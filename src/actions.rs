use crate::camera::OrbitCamera;
use crate::{seek, AppState, Mouse, PlayMode, PlaybackControl, PreProcessorConfig, ShaderService};
use winit::event_loop::ControlFlow;

/// First person camera movement
pub enum CameraMovement {
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    Reset,
}

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
    CameraMove(CameraMovement),
    UIToggleVisible,
    ToggleFullscreen,
    Screenshot,
    PrintSource,
    TakeSnapshot,
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
                app_state.playback_time = seek(
                    app_state.playback_time,
                    &app_state.modifier,
                    PlaybackControl::Stop,
                );
                app_state.play_mode = PlayMode::Paused;
                log::debug!("Stopped");
            }
            Action::TogglePlayPause => match app_state.play_mode {
                PlayMode::Playing => {
                    app_state.play_mode = PlayMode::Paused;
                    log::debug!("Paused");
                }
                PlayMode::Paused => {
                    app_state.timer.start();
                    app_state.play_mode = PlayMode::Playing;
                    log::debug!("Playing");
                }
            },
            Action::TimeForward(time) => {
                app_state.playback_time = seek(
                    app_state.playback_time,
                    &app_state.modifier,
                    PlaybackControl::Forward(time),
                )
            }
            Action::TimeRewind(time) => {
                app_state.playback_time = seek(
                    app_state.playback_time,
                    &app_state.modifier,
                    PlaybackControl::Rewind(time),
                )
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
                        shader_service.reload(PreProcessorConfig {
                            use_camera_integration: true,
                        });
                        //.expect("Expected successful shader reload");
                    }
                }
                false => {
                    if shader_service.use_camera_integration {
                        log::info!("Disabling camera integration");
                        shader_service.use_camera_integration = false;
                        shader_service.reload(PreProcessorConfig {
                            use_camera_integration: false,
                        });
                        //.expect("Expected successful shader reload");
                    }
                }
            },
            Action::CameraReset => {
                app_state.camera = Box::from(OrbitCamera::default());
                app_state.mouse = Mouse::default();
            }
            Action::UIToggleVisible => {
                app_state.ui_visible = !app_state.ui_visible;
                log::debug!("Action::UIToggleVisible {:?}", app_state.ui_visible);
            }
            Action::ToggleFullscreen => {
                app_state.is_fullscreen = !app_state.is_fullscreen;
            }
            Action::Screenshot => {
                log::debug!("Take screenshot. To be implemented.");
            }
            Action::PrintSource => {
                shader_service.source();
            }
            Action::CameraMove(camera_movement) => match camera_movement {
                CameraMovement::MoveForward => {
                    app_state.camera_pos.z += 0.2;
                }
                CameraMovement::MoveBackward => {
                    app_state.camera_pos.z -= 0.2;
                }
                CameraMovement::StrafeLeft => {
                    app_state.camera_pos.x -= 0.2;
                }
                CameraMovement::StrafeRight => {
                    app_state.camera_pos.x += 0.2;
                }
                CameraMovement::Reset => {
                    app_state.camera_pos.x = 0.0;
                    app_state.camera_pos.y = 0.0;
                    app_state.camera_pos.z = 0.0;
                }
            },

            Action::TakeSnapshot => shader_service.save_snapshot(),
        }
    }
}
