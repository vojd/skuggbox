use crate::opengl::Shader;
use crate::utils::cstr_with_len;

pub fn create_program(dir: &str, vs: &str, fs: &str) -> ShaderProgram {
    let vertex_shader_path = format!("{}/{}", dir, vs);
    let frag_shader_path = format!("{}/{}", dir, fs);
    let vertex_shader =
        Shader::from_source(vertex_shader_path, gl::VERTEX_SHADER).expect("shader error");
    let frag_shader =
        Shader::from_source(frag_shader_path, gl::FRAGMENT_SHADER).expect("shader error");
    println!("{} {} {}", dir, vs, fs);
    ShaderProgram::new(vertex_shader, frag_shader)
}

pub struct ShaderProgram {
    pub id: gl::types::GLuint,
    link_status: gl::types::GLint,
}

impl ShaderProgram {
    pub fn new(vert_shader: Shader, frag_shader: Shader) -> Self {
        let id = unsafe { gl::CreateProgram() };
        println!("created new shader program {}", id);

        unsafe {
            gl::AttachShader(id, vert_shader.id);
            gl::AttachShader(id, frag_shader.id);
            gl::LinkProgram(id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = cstr_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }
            eprintln!("linker error {}", error.to_string_lossy());
        }

        unsafe {
            gl::DetachShader(id, vert_shader.id);
            gl::DetachShader(id, frag_shader.id);
        }
        println!("Shader program created successfully {}", id);
        Self {
            id,
            link_status: success,
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

#[allow(dead_code)]
pub struct ShaderService {
    dir: String,
    vs: String,
    fs: String,
    pub program: ShaderProgram,
}

impl ShaderService {
    pub fn new(dir: String, vs: String, fs: String) -> Self {
        let program = create_program("shaders", "base.vert", "base.frag");
        println!("new {}", program.id);
        Self {
            dir,
            vs,
            fs,
            program,
        }
    }

    pub fn reload(&mut self) {
        self.program = create_program("shaders", "base.vert", "base.frag");
        println!("reloading {}", self.program.id);
    }
}
