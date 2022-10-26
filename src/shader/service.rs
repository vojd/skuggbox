use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::shader::{PreProcessor};
use crate::{PreProcessorConfig, SkuggboxShader};


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

    /// This method should be called from the GL-thread.
    /// It is basically the same as watching for file changes and the
    /// reload the shaders whenever that happens.
    pub fn run(&mut self) {
        // pull file updates from the channel
        if let Some(recv) = &self.receiver {
            let value = recv.try_recv();
            if value.is_ok() {
                let changed_file_path = value.ok().unwrap();

                for shader in self.shaders.iter_mut() {
                    if shader.uses_file(&changed_file_path) {
                        let reloaded_shader = self.pre_processor.load_file(shader.get_main_shader_path());
                        shader.mark_for_recompilation(reloaded_shader);

                        // TODO: If the included files change in the shader, they won't be watched
                    }
                }
            }
        };

        for shader in self.shaders.iter_mut() {
            if shader.try_to_compile() {
                // Hurray! The shader was compiled
                shader.find_shader_uniforms();
            }
        }
    }

    fn reload_shader(&mut self, shader: &mut SkuggboxShader) {
        let reloaded_shader = self.pre_processor.load_file(shader.get_main_shader_path());
        shader.mark_for_recompilation(reloaded_shader);
    }

    /// Reloading re-constructs all shaders.
    pub fn reload(&mut self) {
        for shader in self.shaders.iter_mut() {
            let reloaded_shader = self.pre_processor.load_file(shader.get_main_shader_path());
            shader.mark_for_recompilation(reloaded_shader);
        }
    }
}
