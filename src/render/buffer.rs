use glow::{HasContext, NativeVertexArray};
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum BufferType {
    VertexBuffer,
    IndexBuffer,
    FrameBuffer,
}

#[derive(Clone, Debug)]
pub struct Buffer {
    pub gl_buffer: Option<NativeVertexArray>,
    // TODO(mathias): Add buffer_type and examine if we need to expose buffer size or not
}

fn get_buffer_type(buffer_type: BufferType) -> gl::types::GLenum {
    match buffer_type {
        BufferType::VertexBuffer => gl::ARRAY_BUFFER,
        BufferType::IndexBuffer => gl::ELEMENT_ARRAY_BUFFER,
        BufferType::FrameBuffer => gl::FRAMEBUFFER,
    }
}

impl Buffer {
    pub unsafe fn new_vertex_buffer(gl: &glow::Context) -> Self {
        let vertices: Vec<f32> = vec![
            1.0, 1.0, 0.0, // 0
            -1.0, 1.0, 0.0, // 1
            1.0, -1.0, 0.0, // 2
            -1.0, -1.0, 0.0, // 3
        ];

        // let buffer_type = get_buffer_type(BufferType::VertexBuffer);
        // let size = (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr;

        // unsafe {
        //     // let mut vbo: gl::types::GLuint = 0;
        //     let vbo = gl.create_buffer()?;
        //     // gl::GenBuffers(1, &mut vbo);
        //
        //     gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        //     gl::BufferData(
        //         buffer_type,                                   // target
        //         size,                                          // size of data in bytes
        //         vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
        //         gl::STATIC_DRAW,                               // usage
        //     );
        //     gl::BindBuffer(buffer_type, 0);
        // }
        //
        // // set up vertex array object
        // let mut vao: gl::types::GLuint = 0;
        // unsafe {
        //     gl::GenVertexArrays(1, &mut vao);
        //     gl::BindVertexArray(vao);
        //     gl::BindBuffer(buffer_type, vbo);
        //     gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        //     gl::VertexAttribPointer(
        //         0,         // index of the generic vertex attribute ("layout (location = 0)")
        //         3,         // the number of components per generic vertex attribute
        //         gl::FLOAT, // data type
        //         gl::FALSE, // normalized (int-to-float conversion)
        //         (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        //         std::ptr::null(), // offset of the first component
        //     );
        //     gl::BindBuffer(buffer_type, 0);
        //     gl::BindVertexArray(0);
        // }

        let vao = {
            let vbo = gl.create_buffer().unwrap();
            let vao = gl.create_vertex_array().unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            gl.bind_vertex_array(None);

            vao
        };

        Self {
            gl_buffer: Some(vao),
        }
    }

    pub unsafe fn bind(&self, gl: &glow::Context) {
        // unsafe {
        //     gl::BindVertexArray(self.gl_buffer);
        // }

        if let Some(vao) = self.gl_buffer {
            gl.bind_vertex_array(Some(vao));
        } else {
            log::info!("no vao to bind");
        }
    }

    pub unsafe fn unbind(&self, gl: &glow::Context) {
        if let Some(vao) = self.gl_buffer {
            gl.delete_vertex_array(vao);
        }
    }
}
