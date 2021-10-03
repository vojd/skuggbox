use std::path::{PathBuf};
use log::{error, info};

use crate::shader::{Shader, ShaderError, find_included_files};
use crate::uniforms::{read_uniforms, Uniform};
use crate::utils::{cstr_with_len};

const VERTEX_SHADER: &str = "#version 330 core
    layout (location = 0) in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
    ";

pub fn create_program(fragment_path: PathBuf) -> Result<ShaderProgram, ShaderError> {
    let vertex_shader = Shader::from_source(String::from(VERTEX_SHADER), gl::VERTEX_SHADER)?;
    let frag_shader = Shader::from_file(fragment_path, gl::FRAGMENT_SHADER)?;
    info!(
        "Creating shader program: {} {}",
        vertex_shader.id, frag_shader.id
    );
    Ok(ShaderProgram::new(vertex_shader, frag_shader))
}

pub struct ShaderProgram {
    pub id: gl::types::GLuint,
}

impl ShaderProgram {
    pub fn new(vert_shader: Shader, frag_shader: Shader) -> Self {
        let id = unsafe { gl::CreateProgram() };
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
            error!("linker error {}", error.to_string_lossy());
            panic!("linker error");
        }

        unsafe {
            gl::DetachShader(id, vert_shader.id);
            gl::DetachShader(id, frag_shader.id);
        }

        Self { id }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct ShaderService {
    fs: PathBuf,
    pub program: Option<ShaderProgram>,
    pub files: Vec<PathBuf>,
    uniforms: Vec<Uniform>, // TODO: Unused
}

impl ShaderService {
    pub fn new(fs: PathBuf) -> Self {
        // NOTE: unwrapping here until we have UI
        // Need to be able to recreate the shader service when user fixes the error
        let program = create_program(fs.clone()).unwrap();
        let files = if let Some(f) = find_included_files(fs.clone()) {
            vec![fs.clone(), f.iter().collect()]
        } else {
            vec![fs.clone()]
        };

        Self {
            fs,
            program: Some(program),
            files,
            uniforms: vec![],
        }
    }

    pub fn reload(&mut self) {
        match create_program(self.fs.clone()) {
            Ok(new_program) => {
                self.program = Some(new_program);
                self.uniforms = read_uniforms(self.fs.clone());
                info!("Shader recreated without errors")
            }
            _ => {
                error!("Compilation failed - not binding failed program");
            }
        };
    }
}
