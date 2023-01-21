use glam::{Vec2, Vec3};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

use crate::event::WindowEventHandler;

#[derive(Debug)]
pub struct Mouse {
    pub pos: Vec2,
    pub last_pos: Vec2,
    pub delta: Vec2,
    /// Only keep track of the direction the mouse is going in range -1 to 1
    /// Useful for when you want to rotate the camera based on the mouse direction
    /// and avoid camera jumps when cursor enters far from the previous hit.
    /// Remember to scale this value in the shader to fit your liking.
    ///
    /// There is also the `Camera` and its `sb_camera_transform` which you should use
    /// for a pre-made camera which injects into the shader.
    /// x: horizontal rotation
    /// y: vertical rotation
    /// z: zoom level
    pub dir: Vec3,

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

            dir: Vec3::default(),
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

                    self.pos = Vec2::new(position.x as f32, position.y as f32);

                    let v = vec2_to_dir(self.delta);
                    self.dir.x += v.x;
                    self.dir.y += v.y;
                }
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

            WindowEvent::MouseWheel { delta, .. } => {
                self.dir.z -= match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                true
            }

            _ => false,
        }
    }
}

/// Simply return a numerical direction in -1, 0 and 1 on which direction on the number line
/// the value points
fn vec2_to_dir(v: Vec2) -> Vec2 {
    Vec2::new(
        if v.x < 0. {
            -1.0
        } else if v.x > 0. {
            1.
        } else {
            0.
        },
        if v.y < 0. {
            -1.0
        } else if v.y > 0. {
            1.
        } else {
            0.
        },
    )
}
