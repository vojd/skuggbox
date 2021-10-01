use glam::Vec2;
use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::event::WindowEventHandler;

#[derive(Debug)]
pub struct Mouse {
    pub pos: Vec2,
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
            pos: Vec2::new(0.0, 0.0),
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
    fn handle_window_events(&mut self, event: &WindowEvent) -> bool {
        let mouse = self;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if mouse.is_rmb_down {
                    mouse.delta = Vec2::new(
                        position.x as f32 - mouse.pos.x,
                        mouse.pos.y - position.y as f32,
                    );

                    // reset mouse delta so we don't get jumps when we
                    if mouse.is_first_rmb_click {
                        mouse.delta = Vec2::ZERO;
                        mouse.is_first_rmb_click = false;
                    }
                    mouse.pos = Vec2::new(position.x as f32, position.y as f32);
                }
                true
            }

            WindowEvent::MouseInput { button, state, .. } => {
                mouse.is_lmb_down = *button == MouseButton::Left && *state == ElementState::Pressed;
                mouse.is_mmb_down = *button == MouseButton::Middle && *state == ElementState::Pressed;
                mouse.is_rmb_down = *button == MouseButton::Right && *state == ElementState::Pressed;
                mouse.is_first_rmb_click = mouse.is_rmb_down;
                true
            }

            _ => false
        }
    }
}