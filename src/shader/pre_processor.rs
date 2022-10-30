use std::collections::{BTreeMap, HashSet};
/// Utility functions to read shader content
/// and produce the necessary pieces to construct a
use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::shader::ShaderError;
use crate::utils::pragma_shader_name;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PragmaDirective {
    Camera(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Part {
    pub shader_path: PathBuf,
    pub shader_src: String,
    pub shader_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Shader {
    // <shader_id>.glsl
    pub shader_id: String,
    pub main_shader_path: PathBuf,
    pub parts: BTreeMap<PathBuf, Part>,
    // contains the final shader
    pub shader_src: String,
    pub ready_to_compile: bool,
}

#[derive(Clone)]
pub struct PreProcessorConfig {
    pub use_camera_integration: bool,
}

#[derive(Clone)]
pub struct PreProcessor {
    pub config: PreProcessorConfig,
}

impl PreProcessor {
    pub fn new(config: PreProcessorConfig) -> Self {
        Self { config }
    }

    pub fn load_file(&self, shader_path: PathBuf) -> Shader {
        let shader_name = shader_path.file_name().unwrap().to_str().unwrap();
        let shader_id = match shader_name.rsplit_once('.') {
            Some((left, _)) => left.to_string(),
            None => shader_name.to_string(),
        };

        let mut shader = Shader {
            shader_id,
            main_shader_path: shader_path.to_owned(),
            parts: Default::default(),
            shader_src: String::new(),
            ready_to_compile: false,
        };

        let mut loaded_files: HashSet<PathBuf> = HashSet::new();

        match self.process_part(&mut shader, &mut loaded_files, shader_path.clone()) {
            Ok(main_part) => {
                let path = match shader_path.canonicalize() {
                    Ok(x) => x,
                    Err(_) => shader_path.to_owned(),
                };
                shader.parts.insert(path, main_part.clone());
                shader.shader_src = main_part.shader_src;
                shader.ready_to_compile = true;
            }
            Err(e) => {
                log::error!("Error reading shader {:?}: {:?}", shader_path, e);
            }
        }

        shader
    }

    fn process_part(
        &self,
        shader: &mut Shader,
        loaded_files: &mut HashSet<PathBuf>,
        shader_path: PathBuf,
    ) -> anyhow::Result<Part, ShaderError> {
        let result = read_file(shader_path.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let file_contents = result.ok().unwrap();

        // mark the file as read
        loaded_files.insert(shader_path.clone());

        let shader_name = shader_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let shader_source =
            self.process_includes(shader, loaded_files, &shader_path, file_contents);

        Ok(Part {
            shader_path,
            shader_src: shader_source,
            shader_name,
        })
    }

    fn process_includes(
        &self,
        shader: &mut Shader,
        loaded_files: &mut HashSet<PathBuf>,
        shader_path: &Path,
        source: String,
    ) -> String {
        let blocks: Vec<String> = source
            .lines()
            .map(|line| {
                if !is_include_line(line.trim_start()) {
                    return line.to_string();
                }

                let shader_name = pragma_shader_name(line);
                let base_dir = shader_path.parent().unwrap();
                let path = base_dir.join(shader_name);

                if loaded_files.contains(&path) {
                    log::warn!("multiple includes of shader: {:?}", path);
                    return format!("// {}", line);
                }

                match self.process_part(shader, loaded_files, path.clone()) {
                    Ok(part) => {
                        let source = part.shader_src.clone();
                        shader.parts.insert(path, part);
                        source
                    }
                    Err(e) => {
                        log::warn!("failed to load file: {:?}: {:?}", path, e);
                        format!("// {}", line)
                    }
                }
            })
            .collect();

        blocks.join("\n")
    }
}

fn is_include_line(s: &str) -> bool {
    s.starts_with("#pragma") && s.contains("include")
}

fn read_file(shader_path: PathBuf) -> anyhow::Result<String, ShaderError> {
    let mut file = File::open(shader_path.clone()).map_err(|e| ShaderError::FileError {
        error: format!(
            "Err: {:?}, {:?} is invalid or does not exit",
            e, shader_path
        ),
    })?;

    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    Ok(s)
}
