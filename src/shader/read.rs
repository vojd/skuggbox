/// Utility functions to read shader content
/// and produce the necessary pieces to construct a
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(not(test))]
use log::{info, warn};

#[cfg(test)]
use std::{println as info, println as warn};

use crate::shader::ShaderError;
use crate::utils::{include_statement_from_string, pragma_shader_name, string_between};
use std::borrow::{Borrow, BorrowMut};
use std::convert::TryInto;

/// Read a shader file from disk and include any shaders in #pragma include(shadername.ext)
/// TODO: Allow reading from multiple directories.
/// 1. Read from system dir
/// 2. Read from project dir
/// TODO: REMOVE
pub fn read_from_file(source_file: PathBuf) -> anyhow::Result<String, ShaderError> {
    let s = read_shader_src(source_file.clone())?;

    // Find potential include files
    let mut includes = HashMap::<&str, String>::new();
    for line in s.lines() {
        if is_include_line(line.trim_start()) {
            let shader_name = pragma_shader_name(line);
            let path = source_file
                .parent()
                .expect("Could not read path from shader source file");
            let import_source_file = path.join(Path::new(shader_name.as_str()));
            let k = read_shader_src(import_source_file)?;
            includes.insert(line, k);
        }
    }

    if includes.is_empty() {
        return Ok(s);
    }

    // Do the actual inclusion
    let k: String = s
        .lines()
        .map(|line| {
            if includes.contains_key(line) {
                info!("including {:?}", line);
                format!("{}\n", includes.get(line).unwrap())
            } else {
                format!("{}\n", line)
            }
        })
        .collect();

    Ok(k)
}

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
pub struct Part {
    pub shader_path: PathBuf,
    pub shader_src: String,
    pub shader_name: String,
    pub parent_path: PathBuf,
    pub parent_src: String,
}

/// Traverse the #pragma includes through recursion
/// Write a hashmap to be used for actual inclusion of files
/// TODO: Error handling
pub fn build_include_map(
    shader_path: PathBuf,
    mut shader_src: String,
    result: &mut HashMap<String, Part>,
) {
    included_files(shader_path.clone(), shader_src.clone())
        .iter()
        .for_each(|(inc_path, inc_name)| {
            let inc_src = read_shader_src(inc_path.to_owned()).unwrap();
            let part = Part {
                shader_path: inc_path.to_owned(),
                shader_src: inc_src.to_owned(),
                shader_name: inc_name.to_owned(),
                parent_path: shader_path.to_owned(),
                parent_src: shader_src.to_owned(),
            };
            result.insert(inc_name.to_owned(), part.clone());
            build_include_map(
                part.to_owned().shader_path,
                part.to_owned().shader_src,
                result,
            );
        });
}

/// Find #pragma include directives in a shader and return path and shader name to included files
pub fn included_files(parent_path: PathBuf, source: String) -> Vec<(PathBuf, String)> {
    let included = source
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
        .collect();

    included
}

// TODO: Use this to reduce read cost
pub fn scan_for_includes(lines: Vec<&str>) -> HashMap<usize, &str> {
    let mut includes: HashMap<usize, &str> = HashMap::new();

    for (i, &line) in lines.iter().enumerate() {
        if is_include_line(line) {
            println!(">> {:?}, {:?}", i, line);
            includes.insert(i, line);
        }
    }
    includes
}

pub struct Includer {
    main_shader_path: PathBuf,
    main_shader_src: String,
    parts: BTreeMap<String, Part>,

    // contains the final shader
    pub(crate) shader_src: String,
}

impl Includer {
    pub fn new(shader_path: PathBuf) -> Self {
        let mut shader_src = read_shader_src(shader_path.clone()).unwrap();
        Self {
            main_shader_src: shader_src,
            main_shader_path: shader_path,
            parts: Default::default(),
            shader_src: Default::default(),
        }
    }

    pub fn process_includes(&mut self) {
        self.build_include_map(self.main_shader_path.clone(), self.main_shader_src.clone());

        let mut newmap: HashMap<String, String> = HashMap::new();
        // now go through the parts and inject all existing code into the right places
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

            newmap.insert(include_name.to_owned(), s);
        }

        let final_shader = self
            .main_shader_src
            .lines()
            .map(|line| {
                if is_include_line(line) {
                    let pragma_statement = pragma_shader_name(line);
                    dbg!(&pragma_statement);
                    newmap.get(&pragma_statement).unwrap().to_owned()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        self.shader_src = final_shader;
    }

    pub fn build_include_map(&mut self, shader_path: PathBuf, shader_src: String) {
        included_files(shader_path.clone(), shader_src.clone())
            .iter()
            .for_each(|(inc_path, inc_name)| {
                let inc_src = read_shader_src(inc_path.to_owned()).unwrap();

                let part = Part {
                    shader_path: inc_path.to_owned(),
                    shader_src: inc_src.to_owned(),
                    shader_name: inc_name.to_owned(),
                    parent_path: shader_path.to_owned(),
                    parent_src: shader_src.to_owned(),
                };

                self.parts.insert(inc_name.to_owned(), part.clone());

                self.build_include_map(part.to_owned().shader_path, part.to_owned().shader_src);
            });
    }
}
