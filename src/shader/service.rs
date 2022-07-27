use log::{error, info};
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::shader::VERTEX_SHADER;
use crate::shader::{find_included_files, PreProcessor, Shader, ShaderError};
use crate::uniforms::{read_uniforms, Uniform};
use crate::utils::cstr_with_len;

pub fn create_program(fragment_src: String) -> Result<ShaderProgram, ShaderError> {
    let vertex_shader = Shader::from_source(String::from(VERTEX_SHADER), gl::VERTEX_SHADER)?;
    let frag_shader = Shader::from_source(fragment_src, gl::FRAGMENT_SHADER)?;
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

#[allow(temporary_cstring_as_ptr)]
pub fn get_uniform_location(program: &ShaderProgram, uniform_name: &str) -> i32 {
    unsafe { gl::GetUniformLocation(program.id, CString::new(uniform_name).unwrap().as_ptr()) }
}

fn get_uniform_locations(program: &ShaderProgram) -> ShaderLocations {
    ShaderLocations {
        resolution: get_uniform_location(program, "iResolution"),
        time: get_uniform_location(program, "iTime"),
        time_delta: get_uniform_location(program, "iTimeDelta"),
        mouse: get_uniform_location(program, "iMouse"),
    }
}

#[derive(Default)]
pub struct ShaderLocations {
    pub resolution: i32,
    pub time: i32,
    pub time_delta: i32,
    pub mouse: i32,
}

pub struct ShaderService {
    pre_processor: Box<PreProcessor>,
    fs: PathBuf,
    pub program: Option<ShaderProgram>,
    pub files: Vec<PathBuf>,
    pub use_camera_integration: bool,
    uniforms: Vec<Uniform>, // TODO: Unused
    pub locations: ShaderLocations,

    receiver: Option<Receiver<PathBuf>>,
}

impl ShaderService {
    pub fn new(fs: PathBuf) -> Self {
        let mut pre_processor = PreProcessor::new(fs.clone());
        pre_processor.reload();
        // NOTE: unwrapping here until we have UI
        // Need to be able to recreate the shader service when user fixes the error
        let program = create_program(pre_processor.shader_src.clone()).unwrap();
        let files = if let Some(f) = find_included_files(fs.clone()) {
            vec![fs.clone(), f.iter().collect()]
        } else {
            vec![fs.clone()]
        };
        let locations = get_uniform_locations(&program);

        Self {
            pre_processor: pre_processor.into(),
            fs,
            program: Some(program),
            files,
            use_camera_integration: false,
            uniforms: vec![],
            locations,
            receiver: None,
        }
    }

    pub fn watch(&mut self) {
        let (sender, receiver): (Sender<PathBuf>, Receiver<PathBuf>) = channel();

        self.receiver = Some(receiver);
        let files = self.files.clone();

        let _ = thread::spawn(move || {
            glsl_watcher::watch_all(sender, files);
        });
    }

    pub fn run(&mut self) {
        if let Some(recv) = &self.receiver {
            if recv.try_recv().is_ok() {
                self.reload()
            }
        };
    }

    pub fn reload(&mut self) {
        self.pre_processor.use_camera_integration = self.use_camera_integration;

        self.pre_processor.reload();
        match create_program(self.pre_processor.shader_src.clone()) {
            Ok(new_program) => {
                self.locations = get_uniform_locations(&new_program);
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
