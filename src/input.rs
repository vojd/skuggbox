use ui_backend::Ui;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::{
    state::{AppState, PlayMode},
    Action, ActionModifier, CameraMovement, WindowEventHandler,
};

pub fn handle_events<T>(
    event: &Event<'_, T>,
    control_flow: &mut ControlFlow,
    ui: &mut Ui,
    app_state: &mut AppState,
    actions: &mut Vec<Action>,
) {
    *control_flow = ControlFlow::Poll;

    match event {
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::CloseRequested => {
                    actions.push(Action::AppExit);
                }

                WindowEvent::Resized(size) => {
                    let size = size.to_logical::<i32>(1.0);
                    actions.push(Action::WindowResize((size.width, size.height)))
                }

                WindowEvent::ModifiersChanged(modifier_state) => {
                    let internal =
                        i32::from(modifier_state.shift()) + 2 * i32::from(modifier_state.ctrl());
                    app_state.modifier = match internal {
                        1 => ActionModifier::Slow,
                        2 => ActionModifier::Fast,
                        3 => ActionModifier::SuperSlow,
                        _ => ActionModifier::Normal,
                    }
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                VirtualKeyCode::Escape => {
                                    actions.push(Action::AppExit);
                                }

                                // Timeline controls
                                VirtualKeyCode::Space => {
                                    actions.push(Action::TogglePlayPause);
                                }
                                VirtualKeyCode::Right => {
                                    actions.push(Action::TimeForward(1.0));
                                }
                                VirtualKeyCode::Left => {
                                    actions.push(Action::TimeRewind(1.0));
                                }
                                VirtualKeyCode::Key0 => {
                                    actions.push(Action::TimeStop);
                                }

                                // Movement controls
                                VirtualKeyCode::A => {
                                    actions.push(Action::CameraMove(CameraMovement::StrafeLeft));
                                }
                                VirtualKeyCode::D => {
                                    actions.push(Action::CameraMove(CameraMovement::StrafeRight));
                                }
                                VirtualKeyCode::S => {
                                    actions.push(Action::CameraMove(CameraMovement::MoveBackward));
                                }
                                VirtualKeyCode::W => {
                                    actions.push(Action::CameraMove(CameraMovement::MoveForward));
                                }
                                VirtualKeyCode::X => {
                                    actions.push(Action::CameraMove(CameraMovement::Reset));
                                }

                                // Feature controls
                                VirtualKeyCode::Key1 => {
                                    actions.push(Action::CameraToggleIntegration(false));
                                }
                                VirtualKeyCode::Key2 => {
                                    actions.push(Action::CameraToggleIntegration(true));
                                }
                                VirtualKeyCode::Period => {
                                    // reset all camera settings
                                    actions.push(Action::CameraReset);
                                }

                                // UI controls
                                VirtualKeyCode::Tab => actions.push(Action::UIToggleVisible),
                                VirtualKeyCode::F11 => actions.push(Action::ToggleFullscreen),

                                VirtualKeyCode::P => actions.push(Action::PrintSource),
                                VirtualKeyCode::F12 => actions.push(Action::TakeSnapshot),
                                _ => {}
                            }
                        }
                    }
                }

                _ => {}
            }

            app_state.mouse.handle_window_events(event);
            app_state.camera.handle_window_events(event);
            app_state
                .camera
                .handle_mouse(&app_state.mouse, app_state.delta_time);

            let _event_response = ui.on_event(event);
        }

        Event::MainEventsCleared => {
            if matches!(app_state.play_mode, PlayMode::Playing) {
                app_state.playback_time += app_state.delta_time;
            }

            *control_flow = ControlFlow::Exit;
        }

        Event::RedrawRequested(_) => {}
        _ => (),
    }
}
