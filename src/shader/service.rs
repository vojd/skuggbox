use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::{fs, thread};
use time::format_description;

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
        puffin::profile_function!();

        // pull file updates from the channel
        if let Some(recv) = &self.receiver {
            if let Ok(changed_path_buf) = recv.try_recv() {
                for shader in self.shaders.iter_mut() {
                    if shader.uses_file(&changed_path_buf) {
                        let main_shader_path = shader.get_main_shader_path();
                        log::debug!("Reloading shader {:?}", main_shader_path);

                        let reloaded_sahder = self.pre_processor.load_file(main_shader_path);
                        shader.mark_for_recompilation(reloaded_sahder);
                    }
                }
            }
        };

        for shader in self.shaders.iter_mut() {
            if shader.ready_to_compile {
                match shader.try_to_compile() {
                    Ok(_) => {
                        shader.bind_uniform_locations(gl);
                        self.last_error = None;
                        log::info!("Shader compiled");
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

    /// Saves the current state to disk
    /// Make a `snapshots` dir inside the current directory and save the file with
    /// datetime.glsl
    /// TODO(mathias): Save diff instead of the whole file and allow going back-forth between diffs
    pub fn save_snapshot(&self) {
        if self.last_error.is_none() {
            log::debug!("Snapshot: save as");

            let source = self
                .shaders
                .iter()
                .map(|shader| shader.content.shader_src.clone())
                .collect::<Vec<String>>()
                .join("");

            // create dir if it doesn't exist yet
            let base_shader = self.shaders.first().unwrap();

            if let Some(path) = base_shader.content.main_shader_path.parent() {
                let snapshot_dir = path.join("snapshots");
                dbg!(&snapshot_dir);

                // Create directory if it doesn't exist
                if !snapshot_dir.is_dir() {
                    fs::create_dir(snapshot_dir.clone()).expect("Failed to create snapshot dir");
                    log::debug!("Created snapshot dir {:?}", snapshot_dir);
                }

                // Save snapshot into snapshot dir
                let format =
                    format_description::parse("[year][month][day]_[hour][minute][second]").unwrap();

                let datetime = time::OffsetDateTime::now_local()
                    .unwrap()
                    .format(&format)
                    .unwrap();

                let snapshot_filepath = snapshot_dir.join(format!("snapshot-{}.glsl", datetime));
                log::info!("Snapshot: Saving to {:?}", &snapshot_filepath.clone());
                fs::write(snapshot_filepath, source).unwrap();
            };
        }
    }
}
