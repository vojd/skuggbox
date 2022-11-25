use crate::{PreProcessor, ShaderProgram, ShaderUniformLocations};
use glow::Program;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Part {
    pub shader_path: PathBuf,
    pub shader_src: String,
    pub shader_name: String,
}

/// The textual components that makes up what we need to process and build an OpenGL shader
/// The `ShaderContent` shall never have anything to do with actual OpenGL calls but provide what the
/// shader is called, its text content and where it resides on disk.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ShaderContent {
    /// The filename constitutes the `shader_id`
    /// <shader_id>.glsl
    pub shader_id: String,
    pub main_shader_path: PathBuf,
    pub parts: BTreeMap<PathBuf, Part>,
    // contains the final shader
    pub shader_src: String,
    pub ready_to_compile: bool,
}

/// The SkuggboxShader encapsulates an OpenGL shader program with its uniform locations and
/// metadata around where the shader code comes from etc.
pub struct SkuggboxShader {
    gl: Arc<glow::Context>,
    pub content: ShaderContent,
    pub program: Option<Program>,
    pub locations: ShaderUniformLocations,
    pub ready_to_compile: bool,
}

impl SkuggboxShader {
    /// Creates a Vec of SkuggboxShaders from a Vec of file paths.
    /// This is only to build up the text/code structure of what will become OpenGL shaders
    pub fn from_files(
        gl: Arc<glow::Context>,
        pre_processor: &PreProcessor,
        shader_files: Vec<PathBuf>,
    ) -> Vec<Self> {
        shader_files
            .iter()
            .map(|path| {
                log::info!("starting here");
                let shader = pre_processor.load_file(path);
                let ready_to_compile = shader.ready_to_compile;
                Self {
                    gl: gl.clone(),
                    content: shader,
                    program: None,
                    locations: ShaderUniformLocations::default(),
                    ready_to_compile,
                }
            })
            .collect()
    }

    /// Returns all files that are part of this shader due to inclusion
    pub fn get_all_files(&self) -> Vec<&PathBuf> {
        self.content.parts.keys().collect()
    }

    /// Returns true if a file is used by the shader
    pub fn uses_file(&self, path: &PathBuf) -> bool {
        self.content.parts.keys().any(|p| p.eq(path))
    }

    /// Return the main shader path from where the inclusion tree starts
    pub fn get_main_shader_path(&self) -> &PathBuf {
        &self.content.main_shader_path
    }

    /// Mark the shader so that it's recompiled during the next frame
    pub fn mark_for_recompilation(&mut self, shader: ShaderContent) {
        self.ready_to_compile = shader.ready_to_compile;
        self.content = shader;
    }

    /// Attempt to recompile the shader. Returns true if the shader was recompiled.
    pub fn try_to_compile(&mut self) -> bool {
        if !self.ready_to_compile {
            return false;
        }

        // TODO(mathias): Free / delete the old programs
        // self.shader_program.free();
        self.ready_to_compile = false;

        match ShaderProgram::from_frag_src(&self.gl, self.content.shader_src.clone()) {
            Ok(program) => {
                self.program = Some(program);
                true
            }
            Err(err) => {
                log::warn!("{:?}", err);
                false
            }
        }
    }

    /// Detected uniforms in the shader source
    pub fn find_shader_uniforms(&mut self, gl: &glow::Context) {
        if let Some(program) = self.program {
            self.locations = unsafe { ShaderProgram::uniform_locations(gl, program) };
            log::debug!("time: {:?}", self.locations.time);
            log::debug!("resolution: {:?}", self.locations.resolution);
        }
    }
}
