use glam::{Mat4, Vec2, Vec3, Vec4};
use glutin::event::WindowEvent;
use std::f32::consts::PI;
use winit::event::{ElementState, MouseScrollDelta, VirtualKeyCode};

use crate::event::WindowEventHandler;
use crate::mouse::Mouse;

pub trait CameraModel: WindowEventHandler {
    fn handle_mouse(&mut self, mouse: &Mouse, delta_time: f32);

    fn calculate_uniform_data(&mut self) -> Mat4;
}

pub struct OrbitCamera {
    pos: Vec3,
    target: Vec3,
    angle: Vec2,
    #[allow(dead_code)]
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
            zoom: 5.0,
        }
    }
}

impl CameraModel for OrbitCamera {
    fn handle_mouse(&mut self, mouse: &Mouse, delta_time: f32) {
        if mouse.is_rmb_down {
            // scale the x and y differently since the movement range is different
            self.angle += mouse.delta * Vec2::new(0.75, -1.5) * delta_time;

            if self.angle.x < -1.0 {
                self.angle.x = -1.0
            }
            if self.angle.y < -1.0 {
                self.angle.y = -1.0
            }
        }
    }

    fn calculate_uniform_data(&mut self) -> Mat4 {
        self.pos.x = (self.angle.x * PI).sin() * self.zoom;
        self.pos.y = (self.angle.y * 1.53).sin() * self.zoom;
        self.pos.z = (self.angle.x * PI).cos() * self.zoom;

        let up = Vec3::new(0.0, 1.0, 0.0);
        let forward = (self.target - self.pos).normalize();
        let side = Vec3::cross(up, forward);

        Mat4::from_cols(
            Vec4::new(side.x, side.y, side.z, 0.0),
            Vec4::new(up.x, up.y, up.z, 0.0),
            Vec4::new(forward.x, forward.y, forward.z, 0.0),
            Vec4::new(self.pos.x, self.pos.y, self.pos.z, 1.0),
        )
    }
}

impl WindowEventHandler for OrbitCamera {
    fn handle_window_events(&mut self, event: &WindowEvent<'_>) -> bool {
        match event {
            WindowEvent::MouseWheel { delta, .. } => {
                self.zoom -= match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };

                if self.zoom <= 0.25 {
                    self.zoom = 0.25;
                }

                true
            }

            WindowEvent::KeyboardInput { input, .. } => {
                if input.state == ElementState::Pressed {
                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            VirtualKeyCode::A => {
                                self.target.x += 0.5;
                            }
                            VirtualKeyCode::D => {
                                self.target.x -= 0.5;
                            }
                            VirtualKeyCode::W => {
                                self.target.y += 0.5;
                            }
                            VirtualKeyCode::S => {
                                self.target.y -= 0.5;
                            }

                            _ => (),
                        }
                    }
                }

                true
            }

            _ => false,
        }
    }
}
