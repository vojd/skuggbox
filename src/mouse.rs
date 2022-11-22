use glam::{Vec2, Vec3};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

use crate::event::WindowEventHandler;

#[derive(Debug)]
pub struct Mouse {
    pub pos: Vec3,
    pub last_pos: Vec2,
    pub delta: Vec2,

    pub is_lmb_down: bool,
    pub is_mmb_down: bool,
    pub is_rmb_down: bool,
    pub is_first_rmb_click: bool,
}

impl Default for Mouse {
    fn default() -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            last_pos: Vec2::ZERO,
            delta: Vec2::new(0.0, 0.0),

            is_lmb_down: false,
            is_mmb_down: false,
            is_rmb_down: false,
            is_first_rmb_click: false,
        }
    }
}

impl WindowEventHandler for Mouse {
    fn handle_window_events(&mut self, event: &WindowEvent<'_>) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if self.is_rmb_down {
                    self.delta = Vec2::new(
                        position.x as f32 - self.pos.x,
                        self.pos.y - position.y as f32,
                    );

                    // reset mouse delta so we don't get jumps when we press down the mouse button
                    if self.is_first_rmb_click {
                        self.delta = Vec2::ZERO;
                        self.is_first_rmb_click = false;
                    }

                    self.pos.x = position.x as f32;
                    self.pos.y = position.y as f32;
                }
                true
            }

            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                modifiers,
            } => {
                let dir = match delta {
                    // vertical_scroll => mouse-wheel up/down
                    MouseScrollDelta::LineDelta(_, vertical_scroll) => vertical_scroll,
                    MouseScrollDelta::PixelDelta(_) => &0.,
                };
                self.pos.z += dir;
                true
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if *button == MouseButton::Right && *state == ElementState::Released {
                    self.is_first_rmb_click = true;
                }

                self.is_lmb_down = *button == MouseButton::Left && *state == ElementState::Pressed;
                self.is_mmb_down =
                    *button == MouseButton::Middle && *state == ElementState::Pressed;
                self.is_rmb_down = *button == MouseButton::Right && *state == ElementState::Pressed;
                self.is_first_rmb_click = self.is_rmb_down;
                true
            }

            _ => false,
        }
    }
}
