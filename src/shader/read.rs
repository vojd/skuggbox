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
    shader_src: String,
    files: Vec<PathBuf>,
}

pub fn t(mut shader_src: String, source_file: PathBuf, col: &mut Vec<Part>) -> Part {
    println!("start: source file {:?}", &source_file.to_str());

    let mut part = Part {
        source_file: source_file.clone(),
        shader_src: shader_src.clone(),
        files: vec![],
    };



    // find all included files in this particular file

    // for each included file extract their sources and file paths

    // for each included file add the extracted source into the parent

    // return modified source and path

    let includes : Vec<(PathBuf, String)> = included_files(source_file.clone(), shader_src.clone());
    println!("includes => {:?}", includes);
    for (inc_shader_path, inc_shader_filename) in includes.iter() {

        let included_src = read_from_file(inc_shader_path.to_owned()).unwrap();
        let string_to_replace = format!("#pragma include({})", inc_shader_filename);
        shader_src = shader_src.replace(string_to_replace.as_str(), included_src.as_str());

        part.files.push(inc_shader_path.to_owned());
        part.shader_src = shader_src.clone();

        col.push(part.clone());
        println!("recurse src {:?} path {:?}", included_src.as_str(), inc_shader_path.as_os_str());
        t(included_src, inc_shader_path.to_owned(), col);
    }

    part
}


/// NOTE TEST
pub fn included_files(parent_path: PathBuf, source: String) -> Vec<(PathBuf, String)> {
    source
        .lines()
        .filter(|line| is_include_line(line.trim_start()))
        .map(pragma_shader_name)
        .map(|shader_name|
             (
                 parent_path.parent().unwrap().join(Path::new(shader_name.as_str())),
                 shader_name,
             )
        )
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
