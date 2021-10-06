/// Utility functions to read shader content
/// and produce the necessary pieces to construct a
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(not(test))]
use log::{info, warn};

#[cfg(test)]
use std::{println as info, println as warn};


use crate::shader::ShaderError;
use crate::utils::{pragma_shader_name, string_between};

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
    pub shader_name: Option<String>,
}

/// Traverse the #pragma includes through recursion
/// Write result into the `result` Vec
/// TODO: Error handling
pub fn create_include_list(shader_path: PathBuf, mut shader_src: String, result: &mut Vec<Part>) {

    let part = Part {
        shader_src: shader_src.clone(),
        shader_path: shader_path.clone(),
        shader_name: None,
    };
    result.push(part);

    let includes: Vec<Part> = included_files(shader_path.clone(), shader_src.clone()).iter().map(|(inc_path, inc_name)| {
        let inc_src = read_shader_src(inc_path.to_owned()).unwrap();
        Part {
            shader_path: inc_path.to_owned(),
            shader_src: inc_src.to_owned(),
            shader_name: Some(inc_name.to_owned()),
        }
    }).collect();

    // TODO: Handle recursive imports - where `a inc b` and `b inc a`
    includes.iter().for_each(|part| {
        create_include_list(part.to_owned().shader_path, part.to_owned().shader_src, result);
    });
}

/// Find #pragma include directives in a shader and return path and shader name to included files
pub fn included_files(parent_path: PathBuf, source: String) -> Vec<(PathBuf, String)> {
    let included = source
        .lines()
        .filter(|line| is_include_line(line.trim_start()))
        .map(pragma_shader_name)
        .map(|shader_name|
            (
                parent_path.parent().unwrap().join(Path::new(shader_name.as_str())).canonicalize().unwrap(),
                shader_name,
            )
        )
        .collect();

    included
}
