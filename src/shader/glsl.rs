use crate::shader::{shader_from_file, shader_from_string, ShaderError};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Shader {
    pub(crate) id: gl::types::GLuint,
}

impl Shader {
    pub fn from_file(
        source_file: PathBuf,
        shader_type: gl::types::GLuint,
    ) -> anyhow::Result<Shader, ShaderError> {
        let id = shader_from_file(source_file, shader_type)?;
        Ok(Shader { id })
    }

    pub fn from_source(
        source: String,
        shader_type: gl::types::GLuint,
    ) -> anyhow::Result<Shader, ShaderError> {
        let id = shader_from_string(source, shader_type)?;
        Ok(Shader { id })
    }
}
