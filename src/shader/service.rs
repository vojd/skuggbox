use log;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::shader::{find_included_files, PreProcessor, ShaderError};
use crate::{PreProcessorConfig, ShaderLocations, ShaderProgram};

/// The SkuggboxShader encapsulates everything we need to load,process and render a shader program
pub struct SkuggboxShader {
    pub shader_program: ShaderProgram,
    pub locations: ShaderLocations,
}

impl SkuggboxShader {
    // TODO(mathias): Pass in the PreProcessorConfig
    pub fn from_files(
        shader_files: Vec<PathBuf>,
        use_camera_integration: bool,
    ) -> anyhow::Result<Vec<SkuggboxShader>, ShaderError> {
        let pre_processor_config = PreProcessorConfig {
            use_camera_integration,
        };
        shader_files
            .iter()
            .map(|path| {
                let mut all_shader_files = vec![];
                let mut pre_processor =
                    PreProcessor::new(path.clone(), pre_processor_config.to_owned());
                pre_processor.pre_process();

                let shader_program =
                    ShaderProgram::from_frag_src(pre_processor.clone().shader_src)?;

                all_shader_files.push(path.clone());
                if let Some(path) = find_included_files(path.clone()) {
                    all_shader_files.extend(path);
                };

                let locations = shader_program.uniform_locations();

                Ok(SkuggboxShader {
                    shader_program,
                    locations,
                })
            })
            .collect()
    }
}

/// The ShaderService handles the inputted shader files, constructs an OpenGL compatible shader
/// as well as builds up a pre-processor for inlining include files etc.
/// It also holds all file data around the used shaders to be used for reloading.
pub struct ShaderService {
    /// All the shader constructs we're using in this setup.
    /// Contains the pre-processor and everything else to build and reload a shader
    pub skuggbox_shaders: Option<Vec<SkuggboxShader>>,
    /// initial set of files used to construct these shaders.
    /// What we initially construct the shaders from
    initial_shader_files: Vec<PathBuf>,
    /// All files that makes up the shaders, with its included files.
    /// This property is what we'll watch for file changes to trigger a reload.
    pub all_shader_files: Vec<PathBuf>,
    pub use_camera_integration: bool,
    /// Two way channels for listening and reacting to changes in our shader files
    receiver: Option<Receiver<PathBuf>>,
}

impl ShaderService {
    pub fn new(shader_files: Vec<PathBuf>) -> anyhow::Result<Self, ShaderError> {
        // Construct a vector of all used shader files
        let mut all_shader_files: Vec<PathBuf> = vec![];
        for file_path in shader_files.iter() {
            all_shader_files.push(file_path.clone());
            if let Some(file_path) = find_included_files(file_path.clone()) {
                all_shader_files.extend(file_path);
            };
        }

        // The actual shader objects we want to use in this demo/intro
        let skuggbox_shaders =
            if let Ok(skuggbox_shaders) = SkuggboxShader::from_files(shader_files.clone(), false) {
                skuggbox_shaders.into()
            } else {
                None
            };

        Ok(Self {
            skuggbox_shaders,
            initial_shader_files: shader_files,
            all_shader_files,
            use_camera_integration: false,
            receiver: None,
        })
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
                match self.reload() {
                    Ok(_) => log::info!("Shader reloaded"),
                    Err(err) => {
                        log::error!("{:?}", &err);
                    }
                }
            }
        };
    }

    /// Reloading re-constructs the shaders.
    pub fn reload(&mut self) -> anyhow::Result<(), ShaderError> {
        let use_cam = self.use_camera_integration;
        if let Ok(skuggbox_shaders) =
            SkuggboxShader::from_files(self.initial_shader_files.to_owned(), use_cam)
        {
            self.skuggbox_shaders = skuggbox_shaders.into()
        };

        Ok(())
    }
}
