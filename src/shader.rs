use std::ffi::CString;
use std::fs::File;
use std::io::Read;

use crate::shader::ShaderError::CompilationError;
use crate::uniforms::{read_uniforms, Uniform};
use crate::utils::{cstr_to_str, cstr_with_len, pragma_shader_name};
use log::info;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ShaderError {
    CompilationError { error: String },
}

#[derive(Debug)]
pub struct Shader {
    pub(crate) id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(
        source_file: PathBuf,
        shader_type: gl::types::GLuint,
    ) -> anyhow::Result<Shader, ShaderError> {
        let id = shader_from_source(source_file, shader_type)?;
        Ok(Shader { id })
    }
}

fn read_from_file(source_file: PathBuf) -> CString {
    let mut file = File::open(source_file.clone()).expect("Failed to read file");
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    // Find potential include files
    let mut includes = HashMap::<&str, String>::new();
    for line in s.lines() {
        let l = line.trim_start();
        // find import line
        if l.starts_with("#pragma") && l.contains("include") {
            let shader_name = pragma_shader_name(l);
            let path = source_file
                .parent()
                .expect("Could not read path from shader source file");
            let import_shader_file = path.join(Path::new(shader_name.as_str()));

            let mut imported_file = File::open(import_shader_file).expect("Failed to import");
            let mut k = String::new();
            imported_file.read_to_string(&mut k).unwrap();

            includes.insert(line, k);
        }
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

    CString::new(k).unwrap()
}

fn shader_from_source(
    source_file: PathBuf,
    shader_type: gl::types::GLuint,
) -> anyhow::Result<gl::types::GLuint, ShaderError> {
    let src = read_from_file(source_file);

    // check for includes

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

        // Prints the compilation error to console
        eprintln!("{}", error.to_string_lossy());
        return Err(CompilationError {
            error: cstr_to_str(&error),
        });
    }

    Ok(id)
}

pub fn create_program(
    vertex_path: PathBuf,
    fragment_path: PathBuf,
) -> Result<ShaderProgram, ShaderError> {
    // let vertex_shader_path = format!("{}/{}", dir, vs);
    // let frag_shader_path = format!("{}/{}", dir, fs);
    let vertex_shader = Shader::from_source(vertex_path, gl::VERTEX_SHADER)?;
    let frag_shader = Shader::from_source(fragment_path, gl::FRAGMENT_SHADER)?;
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

#[allow(dead_code)]
pub struct ShaderService {
    vs: PathBuf,
    fs: PathBuf,
    pub program: Option<ShaderProgram>,
    uniforms: Vec<Uniform>, // TODO: Unused
}

impl ShaderService {
    pub fn new(shader_dir: String, vertex_src: String, frag_src: String) -> Self {
        let vs = PathBuf::from(format!("./{}/{}", shader_dir, vertex_src));
        let fs = PathBuf::from(format!("./{}/{}", shader_dir, frag_src));

        let program = match create_program(vs.clone(), fs.clone()) {
            Ok(p) => Some(p),
            _ => None,
        };

        Self {
            vs,
            fs,
            program,
            uniforms: vec![],
        }
    }

    pub fn reload(&mut self) {
        match create_program(self.vs.clone(), self.fs.clone()) {
            Ok(new_program) => {
                self.program = Some(new_program);
                let uniforms = read_uniforms(self.fs.clone());
                info!("Found uniforms");
                dbg!(&uniforms);
            }
            _ => {
                eprintln!("Compilation failed - not binding failed program");
            }
        };
    }
}
