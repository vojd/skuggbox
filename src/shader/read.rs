use log::{debug, error};
use regex::Regex;
/// Utility functions to read shader content
/// and produce the necessary pieces to construct a
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::shader::ShaderError;
use crate::utils::{include_statement_from_string, pragma_shader_name};
use crate::SKUGGBOX_CAMERA;

/// Read a shader from disk, return String or ShaderError
pub fn read_shader_src(shader_path: PathBuf) -> anyhow::Result<String, ShaderError> {
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

fn is_include_line(s: &str) -> bool {
    s.starts_with("#pragma") && s.contains("include")
}

/// Search for included files from the supplied `shader`
/// TODO: Replace with the new includer
pub fn find_included_files(shader: PathBuf) -> Option<Vec<PathBuf>> {
    let s = match read_shader_src(shader.clone()) {
        Ok(src) => src,
        _ => return None,
    };

    // Find potential include files
    let mut includes = Vec::new();
    for line in s.lines() {
        if is_include_line(line.trim_start()) {
            let shader_name = pragma_shader_name(line);
            let path = shader
                .parent()
                .expect("Could not read path from shader source file");
            let s = path.join(Path::new(shader_name.as_str()));
            includes.push(s);
        }
    }
    Some(includes)
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PragmaDirective {
    Camera(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Part {
    pub shader_path: PathBuf,
    pub shader_src: String,
    pub shader_name: String,
    pub parent_path: PathBuf,
    pub parent_src: String,
}

pub struct PreProcessor {
    main_shader_path: PathBuf,
    main_shader_src: String,
    parts: BTreeMap<String, Part>,

    // if the camera integration should be used or not
    pub use_camera_integration: bool,

    // contains the final shader
    pub shader_src: String,

    // The files which makes up the content of this shader
    pub files: Vec<PathBuf>,
}

impl PreProcessor {
    pub fn new(shader_path: PathBuf) -> Self {
        let shader_src = read_shader_src(shader_path.clone()).unwrap();
        Self {
            main_shader_src: shader_src,
            main_shader_path: shader_path,
            parts: Default::default(),
            use_camera_integration: false,
            shader_src: Default::default(),
            files: vec![],
        }
    }

    pub fn reload(&mut self) {
        match read_shader_src(self.main_shader_path.clone()) {
            Ok(src) => self.process(src),
            Err(e) => error!("Could not re-compile shader {:?}", e),
        };
    }

    pub fn process(&mut self, shader_src: String) {
        self.main_shader_src = shader_src;
        self.process_includes();
        self.process_integrations();
        self.recreate_file_list();
    }

    /// Handle pragma directives which are not
    fn process_pragma(&self, line: &str) -> Option<PragmaDirective> {
        let camera_regex = Regex::new(r"^\s*#pragma\s+skuggbox\s*\(\s*camera\s*\)\s*$").unwrap();

        if line.contains("#pragma") && camera_regex.is_match(line) {
            debug!("Found camera integration: {:?}", line);

            let pragma_content = match self.use_camera_integration {
                true => "#define USE_SKUGGBOX_CAMERA\n".to_string() + SKUGGBOX_CAMERA,
                _ => SKUGGBOX_CAMERA.to_string(),
            };
            return Some(PragmaDirective::Camera(pragma_content));
        }
        None
    }

    pub fn process_integrations(&mut self) {
        self.shader_src = self
            .shader_src
            .lines()
            .map(|line| match self.process_pragma(line) {
                Some(PragmaDirective::Camera(content)) => content,
                _ => line.to_string(),
            })
            .collect::<Vec<String>>()
            .join("\n");
    }

    pub fn process_includes(&mut self) {
        self.build_include_map(self.main_shader_path.clone(), self.main_shader_src.clone());

        let mut include_map: HashMap<String, String> = HashMap::new();
        // Go through all parts to be included and build up a new map
        for (include_name, part) in self.parts.iter() {
            let part_src = part.shader_src.clone();

            // find the includes in this particular part src
            let s = part_src
                .lines()
                .map(|line| {
                    if is_include_line(line) {
                        let pragma_statement = pragma_shader_name(line);
                        let include_string =
                            include_statement_from_string(pragma_statement.clone());
                        let k = self.parts.get(&pragma_statement).unwrap();
                        k.shader_src.replace(&include_string, &k.shader_src)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");

            include_map.insert(include_name.to_owned(), s);
        }

        let final_shader = self
            .main_shader_src
            .lines()
            .map(|line| {
                if is_include_line(line) {
                    let pragma_statement = pragma_shader_name(line);
                    debug!("including: {:?}", pragma_statement);
                    include_map.get(&pragma_statement).unwrap().to_owned()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        self.shader_src = final_shader;
    }

    pub fn build_include_map(&mut self, shader_path: PathBuf, shader_src: String) {
        self.included_files(shader_path.clone(), shader_src.clone())
            .iter()
            .for_each(|(inc_path, inc_name)| {
                let inc_src = read_shader_src(inc_path.to_owned()).unwrap();

                let part = Part {
                    shader_path: inc_path.to_owned(),
                    shader_src: inc_src,
                    shader_name: inc_name.to_owned(),
                    parent_path: shader_path.to_owned(),
                    parent_src: shader_src.to_owned(),
                };

                self.parts.insert(inc_name.to_owned(), part.clone());

                self.build_include_map(part.to_owned().shader_path, part.shader_src);
            });
    }

    /// Find #pragma include directives in a shader and return path and shader name to included files
    pub fn included_files(&self, parent_path: PathBuf, source: String) -> Vec<(PathBuf, String)> {
        source
            .lines()
            .filter(|line| is_include_line(line.trim_start()))
            .map(pragma_shader_name)
            .map(|shader_name| {
                (
                    parent_path
                        .parent()
                        .unwrap()
                        .join(Path::new(shader_name.as_str()))
                        .canonicalize()
                        .unwrap(),
                    shader_name,
                )
            })
            .collect()
    }

    pub fn recreate_file_list(&mut self) {
        self.files = self
            .parts
            .iter()
            .map(|(_, part)| part.shader_path.clone())
            .collect::<Vec<PathBuf>>();
    }
}
