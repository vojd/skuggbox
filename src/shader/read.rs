/// Utility functions to read shader content
/// and produce the necessary pieces to construct a
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use log::{info};

use crate::shader::ShaderError;
use crate::utils::pragma_shader_name;

/// Read a shader file from disk and include any shaders in #pragma include(shadername.ext)
/// TODO: Allow reading from multiple directories.
/// 1. Read from system dir
/// 2. Read from project dir
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

pub fn read_shader_src(source_file: PathBuf) -> anyhow::Result<String, ShaderError> {
    let mut file = File::open(source_file.clone()).map_err(|e| ShaderError::FileError {
        error: format!(
            "Err: {:?}, {:?} is invalid or does not exit",
            e, source_file
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

#[derive(Debug, Clone)]
pub struct Part {
    source_file: PathBuf,
    base_src: String,
}
pub fn t(source_file: PathBuf, col: &mut Vec<Part>) -> Part {

    let base_src = read_shader_src(source_file.clone()).unwrap();
    dbg!(&base_src);
    let part = Part {
        source_file: source_file.clone(),
        base_src: base_src.clone(),
    };

    col.push(part.clone());

    // now find any included file in the base src
    let includes = included_files(base_src.clone());

    let k:Vec<Part> = includes.iter().map(|include| {
        let new_filename = include.to_owned();
        source_file.parent().unwrap().join(Path::new(new_filename.as_str()))
    })
        .map(|a| t(a, col))
        .collect();

    part
}


/// NOTE TEST
pub fn included_files(source: String) -> Vec<String> {
    source
        .lines()
        .filter(|line| is_include_line(line.trim_start()))
        .map(pragma_shader_name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::included_files;

    #[test]
    fn returns_paths_to_included_files() {
        let source = "#pragma include(\"./shaders/base.frag\");".to_owned();
        let included_files = included_files(source);
        dbg!(&included_files);
        assert_ne!(included_files.len(), 0);
    }
}
