use crate::{AppState, ShaderService};
use glow::{HasContext, VertexArray};
use std::sync::Arc;

pub struct Renderer {
    gl: Arc<glow::Context>,
    vertex_array: VertexArray,
}

impl Renderer {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let vertex_array = unsafe {
            gl.create_vertex_array()
                .expect("Cannot create vertex array")
        };
        Self { gl, vertex_array }
    }

    pub fn draw(&self, state: &mut AppState, shader_service: &ShaderService) {
        let gl = self.gl.clone();
        unsafe {
            gl.bind_vertex_array(Some(self.vertex_array));

            gl.clear_color(0.1, 0.2, 0.1, 1.0);

            if let Some(shader) = shader_service.shaders.get(0) {
                // kick shader to gpu
                gl.use_program(shader.program);

                // set uniforms
                if let Some(resolution) = shader.locations.resolution {
                    gl.uniform_2_f32(Some(&resolution), state.width as f32, state.height as f32)
                }

                if let Some(time) = shader.locations.time {
                    gl.uniform_1_f32(Some(&time), state.playback_time)
                }

                if let Some(delta_time) = shader.locations.time_delta {
                    gl.uniform_1_f32(Some(&delta_time), state.delta_time)
                }

                // Mouse uniforms
                if let Some(mouse) = shader.locations.mouse {
                    let x = state.mouse.pos.x;
                    let y = state.mouse.pos.y;

                    let left_mouse = if state.mouse.is_lmb_down { 1.0 } else { 0.0 };
                    let right_mouse = if state.mouse.is_rmb_down { 1.0 } else { 0.0 };

                    gl.uniform_4_f32(Some(&mouse), x, y, left_mouse, right_mouse);
                };

                if let Some(mouse_dir) = shader.locations.mouse_dir {
                    gl.uniform_3_f32(
                        Some(&mouse_dir),
                        state.mouse.dir.x,
                        state.mouse.dir.y,
                        state.mouse.dir.z,
                    );
                }

                if let Some(sb_camera_transform) = shader.locations.sb_camera_transform {
                    let camera = state.camera.calculate_uniform_data();
                    let f32_arr = camera.to_cols_array();
                    gl.uniform_matrix_4_f32_slice(Some(&sb_camera_transform), false, &f32_arr);
                }

                if let Some(cam_pos) = shader.locations.cam_pos {
                    let pos = state.camera_pos;
                    gl.uniform_3_f32(Some(&cam_pos), pos.x, pos.y, pos.z);
                }

                if let Some(sb_color_a) = shader.locations.sb_color_a {
                    let col = state.scene_vars.color_a;
                    gl.uniform_3_f32(Some(&sb_color_a), col[0], col[1], col[2]);
                }

                // actually render
                gl.clear(glow::COLOR_BUFFER_BIT);
                macros::check_for_gl_error!(&gl, "clear");
                gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 3);
                macros::check_for_gl_error!(&gl, "draw_arrays");
            }
        }
    }
}
