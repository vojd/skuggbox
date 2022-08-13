#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum BufferType {
    VertexBuffer,
    IndexBuffer,
    FrameBuffer,
}

#[derive(Clone, Debug)]
pub struct Buffer {
    pub gl_buffer: gl::types::GLuint,
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
    pub fn new_vertex_buffer() -> Self {
        let vertices: Vec<f32> = vec![
            1.0, 1.0, 0.0, // 0
            -1.0, 1.0, 0.0, // 1
            1.0, -1.0, 0.0, // 2
            -1.0, -1.0, 0.0, // 3
        ];

        let buffer_type = get_buffer_type(BufferType::VertexBuffer);
        let size = (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr;

        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                buffer_type,                                   // target
                size,                                          // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW,                               // usage
            );
            gl::BindBuffer(buffer_type, 0);
        }

        // set up vertex array object
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::BindBuffer(buffer_type, vbo);
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0,         // index of the generic vertex attribute ("layout (location = 0)")
                3,         // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );
            gl::BindBuffer(buffer_type, 0);
            gl::BindVertexArray(0);
        }

        Self { gl_buffer: vao }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.gl_buffer);
        }
    }

    pub fn delete(&self) {
        unsafe { gl::DeleteBuffers(0, &self.gl_buffer as *const _) }
    }
}
