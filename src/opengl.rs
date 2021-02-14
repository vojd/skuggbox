use std::ffi::CString;
use std::fs::File;
use std::io::Read;

use anyhow::Error;

use crate::utils::cstr_with_len;

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(source_file: &str, shader_type: gl::types::GLuint) -> Result<Shader, Error> {
        let id = shader_from_source(source_file, shader_type).expect("Shader compilation");
        Ok(Shader { id })
    }
}

fn read_from_file(source_file: &str) -> CString {
    let mut file = File::open(source_file).expect("Failed to read file");
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let src = CString::new(s).unwrap();
    src
}

fn shader_from_source(
    source_file: &str,
    shader_type: gl::types::GLuint,
) -> Result<gl::types::GLuint, String> {
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
        return Err(error.to_string_lossy().into_owned());
    }
    println!("2");
    Ok(id)
}

pub struct ShaderProgram {
    id: gl::types::GLuint,
    link_status: gl::types::GLint,
}

impl ShaderProgram {
    pub fn new(vert_shader: Shader, frag_shader: Shader) -> Self {
        let id = unsafe { gl::CreateProgram() };
        let mut link_status: gl::types::GLint = 1;
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
            // return Err(error.to_string_lossy().into_owned());
        }

        unsafe {
            gl::DetachShader(id, vert_shader.id);
            gl::DetachShader(id, frag_shader.id);
        }

        Self { id, link_status }
    }

    /// Stupid `use` to be reserved word `e
    pub fn activate(&self) {
        unsafe {
            gl::UseProgram(self.id);
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
