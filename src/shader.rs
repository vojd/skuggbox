use std::ffi::CString;
use std::fs::File;
use std::io::Read;

use anyhow::Error;

use crate::utils::cstr_with_len;
use crate::shader::ShaderError::CompilationError;

#[derive(Debug)]
pub enum ShaderError {
    CompilationError,
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
        return Err(CompilationError {});
    }
    println!("2");
    Ok(id)
}

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
    }
}
