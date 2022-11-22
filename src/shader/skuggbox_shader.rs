use crate::{PreProcessor, Shader, ShaderLocations, ShaderProgram};
use std::path::PathBuf;
use std::sync::Arc;

/// The SkuggboxShader encapsulates everything we need to load, process and render a shader program
pub struct SkuggboxShader {
    shader: Shader,
    pub shader_program: ShaderProgram,
    pub locations: ShaderLocations,
    pub ready_to_compile: bool,
}

impl SkuggboxShader {
    /// Creates a Vec of SkuggboxShaders from a Vec of file paths.
    pub fn from_files(
        gl: Arc<glow::Context>,
        pre_processor: &PreProcessor,
        shader_files: Vec<PathBuf>,
    ) -> Vec<SkuggboxShader> {
        shader_files
            .iter()
            .map(|path| {
                log::info!("starting here");
                let shader = pre_processor.load_file(path);
                let ready_to_compile = shader.ready_to_compile;
                SkuggboxShader {
                    shader,
                    shader_program: ShaderProgram::default(),
                    locations: ShaderLocations::default(),
                    ready_to_compile,
                }
            })
            .collect()
    }

    /// Returns all files that are part of this shader due to inclusion
    pub fn get_all_files(&self) -> Vec<&PathBuf> {
        self.shader.parts.keys().collect()
    }

    /// Returns true if a file is used by the shader
    pub fn uses_file(&self, path: &PathBuf) -> bool {
        self.shader.parts.keys().any(|p| p.eq(path))
    }

    /// Return the main shader path from where the inclusion tree starts
    pub fn get_main_shader_path(&self) -> &PathBuf {
        &self.shader.main_shader_path
    }

    /// Mark the shader so that it's recompiled during the next frame
    pub fn mark_for_recompilation(&mut self, shader: Shader) {
        self.ready_to_compile = shader.ready_to_compile;
        self.shader = shader;
    }

    /// Attempt to recompile the shader. Returns true if the shader was recompiled.
    pub fn try_to_compile(&mut self) -> bool {
        if !self.ready_to_compile {
            return false;
        }

        self.shader_program.free();
        self.ready_to_compile = false;

        match ShaderProgram::from_frag_src(self.shader.shader_src.clone()) {
            Ok(program) => {
                self.shader_program = program;
                true
            }
            Err(_) => false,
        }
    }

    /// Detected uniforms in the shader source
    pub fn find_shader_uniforms(&mut self) {
        self.locations = self.shader_program.uniform_locations();
    }
}
