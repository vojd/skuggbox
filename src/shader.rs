use std::ffi::CString;
use std::fs::File;
use std::io::Read;

use crate::shader::ShaderError::CompilationError;
use crate::utils::{cstr_to_str, cstr_with_len};

#[derive(Debug)]
pub enum ShaderError {
    CompilationError { error: String },
}

pub struct Shader {
    pub(crate) id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(
        source_file: String,
        shader_type: gl::types::GLuint,
    ) -> anyhow::Result<Shader, ShaderError> {
        let id = shader_from_source(source_file, shader_type)?;
        Ok(Shader { id })
    }
}

fn read_from_file(source_file: String) -> CString {
    let mut file = File::open(source_file).expect("Failed to read file");
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    CString::new(s).unwrap()
}

fn shader_from_source(
    source_file: String,
    shader_type: gl::types::GLuint,
) -> anyhow::Result<gl::types::GLuint, ShaderError> {
    let src = read_from_file(source_file);

    let id = unsafe { gl::CreateShader(shader_type) };

    unsafe {
        gl::ShaderSource(id, 1, &src.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    // in case something failed we want to extract the error and return it
    if success == 0 {
        // fetch the required buffer length
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = cstr_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            )
        }
        eprintln!("{}", error.to_string_lossy());
        return Err(CompilationError {
            error: cstr_to_str(&error),
        });
    }
    println!("2");
    Ok(id)
}

pub fn create_program(dir: &str, vs: &str, fs: &str) -> Result<ShaderProgram, ShaderError> {
    let vertex_shader_path = format!("{}/{}", dir, vs);
    let frag_shader_path = format!("{}/{}", dir, fs);
    let vertex_shader = Shader::from_source(vertex_shader_path, gl::VERTEX_SHADER)?;
    let frag_shader = Shader::from_source(frag_shader_path, gl::FRAGMENT_SHADER)?;
    println!("{} {} {}", dir, vs, fs);
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
            eprintln!("linker error {}", error.to_string_lossy());
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

#[allow(dead_code)]
pub struct ShaderService {
    dir: String,
    vs: String,
    fs: String,
    pub program: Option<ShaderProgram>,
}

impl ShaderService {
    pub fn new(dir: String, vs: String, fs: String) -> Self {
        let program = match create_program("shaders", "base.vert", "base.frag") {
            Ok(p) => Some(p),
            _ => None,
        };

        Self {
            dir,
            vs,
            fs,
            program,
        }
    }

    pub fn reload(&mut self) {
        match create_program("shaders", "base.vert", "base.frag") {
            Ok(new_program) => {
                self.program = Some(new_program);
            }
            _ => {
                println!("Compilation failed - not binding failed program");
            }
        };
    }
}
