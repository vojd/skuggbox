use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::shader::{PreProcessor};
use crate::{PreProcessorConfig, Shader, ShaderLocations, ShaderProgram};

/// The SkuggboxShader encapsulates everything we need to load,process and render a shader program
pub struct SkuggboxShader {
    shader: Shader,
    pub shader_program: ShaderProgram,
    pub locations: ShaderLocations,
    pub ready_to_compile: bool
}

impl SkuggboxShader {
    pub fn from_files(
        pre_processor: &PreProcessor,
        shader_files: Vec<PathBuf>,
    ) -> Vec<SkuggboxShader> {
        shader_files
            .iter()
            .map(|path| {
                let shader = pre_processor.load_file(path);
                let ready = shader.ready_to_compile.clone();
                SkuggboxShader {
                    shader,
                    shader_program: ShaderProgram::new(),
                    locations: ShaderLocations::new(),
                    ready_to_compile: ready
                }
            })
            .collect()
    }

    pub fn get_all_files(&self) -> Vec<&PathBuf> {
        self.shader.parts.keys().collect()
    }

    pub fn uses_file(&self, path: &PathBuf) -> bool {
        self.shader.parts.keys().any(|p| p.eq(path))
    }

    pub fn get_main_shader_path(&self) -> &PathBuf {
        &self.shader.main_shader_path
    }

    pub fn replace_shader(&mut self, shader: Shader) {
        self.ready_to_compile = shader.ready_to_compile;
        self.shader = shader;
    }

    pub fn try_to_compile(&mut self) -> bool  {
        if !self.ready_to_compile {
            return false;
        }

        self.shader_program.free();

        return true;
    }
}

/// The ShaderService handles the inputted shader files, constructs an OpenGL compatible shader
/// as well as builds up a pre-processor for inlining include files etc.
/// It also holds all file data around the used shaders to be used for reloading.
pub struct ShaderService {
    /// All the shader constructs we're using in this setup.
    /// Contains the pre-processor and everything else to build and reload a shader
    pub shaders: Vec<SkuggboxShader>,
    pub use_camera_integration: bool,
    /// Two way channels for listening and reacting to changes in our shader files
    pre_processor: PreProcessor,
    receiver: Option<Receiver<PathBuf>>,
}

impl ShaderService {
    pub fn new(shader_files: Vec<PathBuf>) -> Self {
        let pre_processor_config = PreProcessorConfig {
          use_camera_integration: false
        };

        let pre_processor = PreProcessor::new(pre_processor_config);
        let shaders = SkuggboxShader::from_files(&pre_processor, shader_files.clone());

        Self {
            pre_processor,
            shaders,
            use_camera_integration: false,
            receiver: None,
        }
    }

    pub fn watch(&mut self) {
        let (sender, receiver): (Sender<PathBuf>, Receiver<PathBuf>) = channel();

        self.receiver = Some(receiver);

        let all_shader_files = self.shaders
            .iter()
            .flat_map(|shader| shader.get_all_files().clone())
            .map(|path| path.to_owned().clone())
            .collect();

        let _ = thread::spawn(move || {
            glsl_watcher::watch_all(sender, all_shader_files);
        });
    }

    /// Running is basically the same as listening and reacting to changes.
    /// We reload the shaders whenever we spot a file change.
    pub fn run(&mut self) {
        if let Some(recv) = &self.receiver {
            let value = recv.try_recv();
            if value.is_ok() {
                let changed_file_path = value.ok().unwrap();

                for mut shader in self.shaders.iter() {
                    if !shader.uses_file(&changed_file_path) {
                        continue;
                    }
                    self.reload_shader(shader);
                }
            }
        };

        for mut shader in self.shaders.iter() {
            if shader.try_to_compile() {
                // TODO: shader was compiled
            }
        }
    }

    fn reload_shader(&self, shader: &mut SkuggboxShader) {
        let reloaded_shader = self.pre_processor.load_file(shader.get_main_shader_path());
        shader.replace_shader(reloaded_shader);
    }

    /// Reloading re-constructs the shaders.
    pub fn reload(&mut self) {
       // TODO: reload everything
    }
}
