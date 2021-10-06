use std::ffi::CString;
use std::path::PathBuf;

use log::error;

use crate::shader::{read_from_file, Includer};
use crate::utils::{cstr_to_str, cstr_with_len};

#[derive(Debug)]
pub enum ShaderError {
    CompilationError { error: String },
    FileError { error: String },
}

#[derive(Debug)]
pub struct Shader {
    pub(crate) id: gl::types::GLuint,
}

impl Shader {
    pub fn from_file(
        source_file: PathBuf,
        shader_type: gl::types::GLuint,
    ) -> anyhow::Result<Shader, ShaderError> {
        // TODO: move to root
        let mut includer = Includer::new(source_file);
        includer.process_includes();
        let shader_src = includer.shader_src;
        let id = shader_from_string(shader_src, shader_type)?;
        Ok(Shader { id })
    }

    pub fn from_source(
        source: String,
        shader_type: gl::types::GLuint,
    ) -> anyhow::Result<Shader, ShaderError> {
        let id = shader_from_string(source, shader_type)?;
        Ok(Shader { id })
    }
}

/// Perform the required OpenGL dance to create an OpenGL shader
fn shader_from_string(
    source: String,
    shader_type: gl::types::GLuint,
) -> anyhow::Result<gl::types::GLuint, ShaderError> {
    let c_src = CString::new(source).expect("Could not convert source to CString");

    // check for includes

    let id = unsafe { gl::CreateShader(shader_type) };

    unsafe {
        gl::ShaderSource(id, 1, &c_src.as_ptr(), std::ptr::null());
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

        // Prints the compilation error to console
        error!("{}", error.to_string_lossy());
        return Err(ShaderError::CompilationError {
            error: cstr_to_str(&error),
        });
    }

    Ok(id)
}
