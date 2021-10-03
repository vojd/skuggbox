use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use log::info;

use crate::shader::Shader;
use crate::uniforms::{read_uniforms, Uniform};
use crate::utils::{cstr_to_str, cstr_with_len, pragma_shader_name};

#[derive(Debug)]
pub enum ShaderError {
    CompilationError { error: String },
    FileError { error: String },
}

const VERTEX_SHADER: &str = "#version 330 core
    layout (location = 0) in vec3 position;
    void main() {
        gl_Position = vec4(position, 1.0);
    }
    ";

fn read_shader_src(source_file: PathBuf) -> anyhow::Result<String, ShaderError> {
    let mut file = File::open(source_file.clone()).map_err(|e| ShaderError::FileError {
        error: format!(
            "Err: {:?}, {:?} is invalid or does not exit",
            e, source_file
        ),
    })?;

    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    Ok(s)
}

fn is_include_line(s: &str) -> bool {
    s.starts_with("#pragma") && s.contains("include")
}

/// Read a shader file from disk and include any shaders in #pragma include(shadername.ext)
/// TODO: Allow reading from multiple directories.
/// 1. Read from system dir
/// 2. Read from project dir
fn read_from_file(source_file: PathBuf) -> anyhow::Result<String, ShaderError> {
    let s = read_shader_src(source_file.clone())?;

    // Find potential include files
    let mut includes = HashMap::<&str, String>::new();
    for line in s.lines() {
        if is_include_line(line.trim_start()) {
            let shader_name = pragma_shader_name(line);
            let path = source_file
                .parent()
                .expect("Could not read path from shader source file");
            let import_source_file = path.join(Path::new(shader_name.as_str()));
            let k = read_shader_src(import_source_file)?;
            includes.insert(line, k);
        }
    }

    if includes.is_empty() {
        return Ok(s);
    }

    // Do the actual inclusion
    let k: String = s
        .lines()
        .map(|line| {
            if includes.contains_key(line) {
                info!("including {:?}", line);
                format!("{}\n", includes.get(line).unwrap())
            } else {
                format!("{}\n", line)
            }
        })
        .collect();

    Ok(k)
}

pub(crate) fn shader_from_string(
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
        eprintln!("{}", error.to_string_lossy());
        return Err(ShaderError::CompilationError {
            error: cstr_to_str(&error),
        });
    }

    Ok(id)
}

pub(crate) fn shader_from_file(
    source_file: PathBuf,
    shader_type: gl::types::GLuint,
) -> anyhow::Result<gl::types::GLuint, ShaderError> {
    let source = read_from_file(source_file)?;
    shader_from_string(source, shader_type)
}

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
                eprintln!("Compilation failed - not binding failed program");
            }
        };
    }
}

// Search for included files from the supplied `shader`
fn find_included_files(shader: PathBuf) -> Option<Vec<PathBuf>> {
    let s = match read_shader_src(shader.clone()) {
        Ok(src) => src,
        _ => return None,
    };

    // Find potential include files
    let mut includes = Vec::new();
    for line in s.lines() {
        if is_include_line(line.trim_start()) {
            let shader_name = pragma_shader_name(line);
            let path = shader
                .parent()
                .expect("Could not read path from shader source file");
            let s = path.join(Path::new(shader_name.as_str()));
            includes.push(s);
        }
    }
    Some(includes)
}
