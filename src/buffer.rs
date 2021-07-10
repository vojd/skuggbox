#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum BufferType {
    VertexBuffer,
    IndexBuffer,
}

#[derive(Clone, Debug)]
pub struct Buffer {
    pub gl_buffer: gl::types::GLuint,
    pub size: gl::types::GLsizeiptr,
}

fn get_buffer_type(buffer_type: BufferType) -> gl::types::GLenum {
    match buffer_type {
        BufferType::VertexBuffer => gl::ARRAY_BUFFER,
        BufferType::IndexBuffer => gl::ELEMENT_ARRAY_BUFFER,
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
        Buffer::vertex_buffer(BufferType::VertexBuffer, &vertices)
    }

    pub fn vertex_buffer(buffer_type: BufferType, vertices: &[f32]) -> Self {
        let buf_type = get_buffer_type(buffer_type);
        let size = (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr;

        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                buf_type,                                      // target
                size,                                          // size of data in bytes
                vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW,                               // usage
            );
            gl::BindBuffer(buf_type, 0);
        }

        // set up vertex array object
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::BindBuffer(buf_type, vbo);
            gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
            gl::VertexAttribPointer(
                0,         // index of the generic vertex attribute ("layout (location = 0)")
                3,         // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );
            gl::BindBuffer(buf_type, 0);
            gl::BindVertexArray(0);
        }

        Self {
            gl_buffer: vao,
            size,
        }
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
