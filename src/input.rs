use crate::{
    buffer::Buffer,
    state::{seek, AppState, PlayMode, PlaybackControl},
    timer::Timer,
    OrbitCamera, ShaderService, WindowEventHandler,
};
use glam::Vec2;
use glutin::{ContextWrapper, PossiblyCurrent};
use log::info;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};
pub fn handle_events<T>(
    event: Event<'_, T>,
    control_flow: &mut ControlFlow,
    world_state: &mut AppState,
    timer: &mut Timer,
    context: &ContextWrapper<PossiblyCurrent, Window>,
    buffer: &Buffer,
    shader: &mut ShaderService,
) {
    *control_flow = ControlFlow::Poll;
    context.swap_buffers().unwrap();

    match event {
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Bye now...");
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

                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                VirtualKeyCode::Escape => {
                                    info!("Bye now...");
                                    world_state.is_running = false;
                                    buffer.delete();
                                    *control_flow = ControlFlow::Exit;
                                }

                                // Timeline controls
                                VirtualKeyCode::Space => {
                                    world_state.play_mode = match world_state.play_mode {
                                        PlayMode::Playing => {
                                            log::info!("Paused");
                                            PlayMode::Paused
                                        }
                                        PlayMode::Paused => {
                                            log::info!("Playing");
                                            timer.start();
                                            PlayMode::Playing
                                        }
                                    }
                                }
                                VirtualKeyCode::Right => {
                                    world_state.playback_time = seek(
                                        world_state.playback_time,
                                        PlaybackControl::Forward(1.0),
                                    )
                                }
                                VirtualKeyCode::Left => {
                                    world_state.playback_time = seek(
                                        world_state.playback_time,
                                        PlaybackControl::Rewind(1.0),
                                    )
                                }
                                VirtualKeyCode::Key0 => {
                                    world_state.playback_time =
                                        seek(world_state.playback_time, PlaybackControl::Stop)
                                }
                                // Feature controls
                                VirtualKeyCode::Key1 => {
                                    if shader.use_camera_integration {
                                        info!("Disabling camera integration");
                                        shader.use_camera_integration = false;
                                        shader.reload();
                                    }
                                }
                                VirtualKeyCode::Key2 => {
                                    if !shader.use_camera_integration {
                                        info!("Enabling camera integration. Please use '#pragma skuggbox(camera)' in your shader");
                                        shader.use_camera_integration = true;
                                        shader.reload();
                                    }
                                }
                                VirtualKeyCode::Period => {
                                    // reset all camera settings
                                    world_state.camera = Box::from(OrbitCamera::default());
                                    world_state.mouse.delta = Vec2::new(0.0, 0.0);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                _ => {}
            }

            world_state.mouse.handle_window_events(&event);
            world_state.camera.handle_window_events(&event);
            world_state
                .camera
                .handle_mouse(&world_state.mouse, world_state.delta_time);
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
