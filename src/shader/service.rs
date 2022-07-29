use log;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::shader::VERTEX_SHADER;
use crate::shader::{find_included_files, PreProcessor, Shader, ShaderError};
use crate::ShaderProgram;

/// Construct an OpenGL compatible shader program
pub fn create_program(fragment_src: String) -> Result<ShaderProgram, ShaderError> {
    let vertex_shader = Shader::from_source(String::from(VERTEX_SHADER), gl::VERTEX_SHADER)?;
    let frag_shader = Shader::from_source(fragment_src, gl::FRAGMENT_SHADER)?;
    log::info!(
        "Creating shader program: {} {}",
        vertex_shader.id,
        frag_shader.id
    );
    Ok(ShaderProgram::new(vertex_shader, frag_shader))
}

/// Simple struct to hold all that's necessary to build and read the contents of a shader
pub struct SkuggboxShader {
    pub pre_processor: PreProcessor,
    pub shader_program: ShaderProgram,
    pub locations: ShaderLocations,
    pub files: Vec<PathBuf>,
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

/// Given a Vec of paths, create the OpenGL shaders to be used
fn create_shaders(shader_files: Vec<PathBuf>, use_cam_integration: bool) -> Vec<SkuggboxShader> {
    let skuggbox_shaders: Vec<SkuggboxShader> = shader_files
        .iter()
        .map(|path| {
            let mut all_shader_files = vec![];
            let mut pre_processor = PreProcessor::new(path.clone());
            pre_processor.use_camera_integration = use_cam_integration;
            pre_processor.reload();

            let shader_program = create_program(pre_processor.clone().shader_src).unwrap();

            all_shader_files.push(path.clone());
            if let Some(path) = find_included_files(path.clone()) {
                all_shader_files.extend(path);
            };

            let locations = get_uniform_locations(&shader_program);

            SkuggboxShader {
                pre_processor,
                shader_program,
                locations,
                files: all_shader_files,
            }
        })
        .collect();
    skuggbox_shaders
}

/// The ShaderService handles the inputted shader files, constructs an OpenGL compatible shader
/// as well as builds up a pre-processor for inlining include files etc.
/// It also holds all file data around the used shaders to be used for reloading.
pub struct ShaderService {
    /// All the shader constructs we're using in this setup.
    /// Contains the pre-processor and everything else to build and reload a shader
    pub skuggbox_shaders: Vec<SkuggboxShader>,
    /// initial set of files used to construct these shaders
    initial_shader_files: Vec<PathBuf>,
    /// all files that makes up these shaders, with included files
    pub all_shader_files: Vec<PathBuf>,
    pub use_camera_integration: bool,
    /// Two way channels for listening and reacting to changes in our shader files
    receiver: Option<Receiver<PathBuf>>,
}

impl ShaderService {
    pub fn new(shader_files: Vec<PathBuf>) -> Self {
        // Construct a vector of all used shader files
        let mut all_shader_files: Vec<PathBuf> = vec![];
        for f in shader_files.iter() {
            all_shader_files.push(f.clone());
            if let Some(f) = find_included_files(f.clone()) {
                all_shader_files.extend(f);
            };
        }

        // The actual shader objects we want to use in this demo/intro
        let skuggbox_shaders = create_shaders(shader_files.clone(), false);

        Self {
            skuggbox_shaders,
            initial_shader_files: shader_files,
            all_shader_files,
            use_camera_integration: false,
            receiver: None,
        }
    }

    pub fn watch(&mut self) {
        let (sender, receiver): (Sender<PathBuf>, Receiver<PathBuf>) = channel();

        self.receiver = Some(receiver);
        let files = self.all_shader_files.clone();

        let _ = thread::spawn(move || {
            glsl_watcher::watch_all(sender, files);
        });
    }

    /// Running is basically the same as listening and reacting to changes.
    /// We reload the shaders whenever we spot a file change.
    pub fn run(&mut self) {
        if let Some(recv) = &self.receiver {
            if recv.try_recv().is_ok() {
                self.reload()
            }
        };
    }

    /// Reloading re-constructs the shaders.
    pub fn reload(&mut self) {
        let use_cam = self.use_camera_integration;
        self.skuggbox_shaders = create_shaders(self.initial_shader_files.clone(), use_cam);
    }
}
