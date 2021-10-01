use glam::{Vec2, Vec3};
use glutin::event::WindowEvent;
use winit::event::{ElementState, MouseScrollDelta, VirtualKeyCode};

use crate::event::WindowEventHandler;
use crate::mouse::Mouse;

pub trait CameraModel: WindowEventHandler {
    fn handle_mouse(&mut self, mouse: &Mouse);
}


#[derive(Debug)]
pub struct OrbitCamera {
    pos: Vec3,
    target: Vec3,
    angle: Vec2,
    speed: f32,
    zoom: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            pos: Vec3::new(02.0, 2.0, -2.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            angle: Vec2::ZERO,
            speed: 1.0,
            zoom: 0.0,
        }
    }
}

impl CameraModel for OrbitCamera {
    fn handle_mouse(&mut self, mouse: &Mouse) {
        self.angle += mouse.delta;
    }
}

impl WindowEventHandler for OrbitCamera {
    fn handle_window_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel {delta, ..} => {
                self.zoom += match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                true
            },

            WindowEvent::KeyboardInput { input, .. } => {
                if input.state == ElementState::Pressed {
                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            VirtualKeyCode::A => {
                                self.pos.x -= 0.5;
                                self.target.x += 0.5;
                            }
                            VirtualKeyCode::D => {
                                self.pos.x += 0.5;
                                self.target.x -= 0.5;
                            }
                            VirtualKeyCode::W => {
                                self.pos.y -= 0.5;
                                self.target.y += 0.5;
                            }
                            VirtualKeyCode::S => {
                                self.pos.y += 0.5;
                                self.target.y -= 0.5;
                            }

                            _ => ()
                        }
                    }
                }

                true
            }

            _ => false
        }
    }
}