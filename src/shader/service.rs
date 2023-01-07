use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use crate::shader::PreProcessor;
use crate::{PreProcessorConfig, ShaderError, SkuggboxShader};

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
    pub last_error: Option<ShaderError>,
}

impl ShaderService {
    pub fn new(gl: Arc<glow::Context>, shader_files: Vec<PathBuf>) -> Self {
        let pre_processor_config = PreProcessorConfig {
            use_camera_integration: false,
        };

        let pre_processor = PreProcessor::new(pre_processor_config);
        let shaders = SkuggboxShader::from_files(gl, &pre_processor, shader_files);

        Self {
            pre_processor,
            shaders,
            use_camera_integration: false,
            receiver: None,
            last_error: None,
        }
    }

    pub fn watch(&mut self) {
        let (sender, receiver): (Sender<PathBuf>, Receiver<PathBuf>) = channel();

        self.receiver = Some(receiver);

        let all_shader_files = self
            .shaders
            .iter()
            .flat_map(|shader| shader.get_all_files())
            .cloned()
            .collect();

        let _ = thread::spawn(move || {
            glsl_watcher::watch_all(sender, all_shader_files);
        });
    }

    /// This method should be called from the GL-thread.
    /// It is basically the same as watching for file changes and the
    /// reload the shaders whenever that happens.
    pub fn run(&mut self, gl: &glow::Context) -> Result<(), ShaderError> {
        // pull file updates from the channel
        if let Some(recv) = &self.receiver {
            let value = recv.try_recv();
            if value.is_ok() {
                let changed_file_path = value.ok().unwrap();

                for shader in self.shaders.iter_mut() {
                    if shader.uses_file(&changed_file_path) {
                        let reloaded_shader =
                            self.pre_processor.load_file(shader.get_main_shader_path());
                        shader.mark_for_recompilation(reloaded_shader);

                        // TODO: If the included files change in the shader, they won't be watched
                    }
                }
            }
        };

        for shader in self.shaders.iter_mut() {
            if shader.ready_to_compile {
                match shader.try_to_compile() {
                    Ok(_) => {
                        log::debug!("Shader compiled");
                        shader.find_shader_uniforms(gl);
                        self.last_error = None;
                    }
                    Err(e) => {
                        self.last_error = Some(e.clone());
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Reloading re-constructs all shaders.
    pub fn reload(&mut self, config: PreProcessorConfig) {
        self.pre_processor.config = config;
        for shader in self.shaders.iter_mut() {
            let reloaded_shader = self.pre_processor.load_file(shader.get_main_shader_path());
            shader.mark_for_recompilation(reloaded_shader);
        }
    }

    pub fn source(&self) {
        for shader in &self.shaders {
            log::info!("{}", shader.content.shader_id);
            log::info!("{}", shader.content.shader_src);
        }
    }
}
