/// Utility functions to read shader content
/// and produce the necessary pieces to construct a
use std::collections::{HashMap, BTreeMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(not(test))]
use log::{info, warn};

#[cfg(test)]
use std::{println as info, println as warn};


use crate::shader::ShaderError;
use crate::utils::{pragma_shader_name, string_between, include_statement_from_string};
use std::borrow::{BorrowMut, Borrow};
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
pub fn build_include_map(shader_path: PathBuf, mut shader_src: String, result: &mut HashMap<String, Part>) {

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
            build_include_map(part.to_owned().shader_path, part.to_owned().shader_src, result);
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

/// performs inclusions in one particular file
pub fn do_includes(shader_src: String, includes: HashMap<String, Part>) -> String {
    let mut v = shader_src.lines().collect::<Vec<&str>>();
    let hs = scan_for_includes(v.clone());
    for (k, entry) in hs {
        let include_name = pragma_shader_name(entry);
        let part = includes.get(include_name.as_str()).unwrap();
        println!(":: {:?}, {:?}", k, part.shader_src);

        v[k] = part.shader_src.as_str();
    }
    //
    // let k = v
    //     .iter()
    //     .map(|line| {
    //         if is_include_line(line) {
    //             let include_name = pragma_shader_name(line);
    //             let ss = includes.get(include_name.as_str()).map(|s|{
    //                 // println!(">> {:?}", s.shader_src);
    //                 s.shader_src.as_str()
    //             }).unwrap();
    //             ss
    //         } else {
    //             line
    //         }
    //     })
    //     .collect::<Vec<&str>>();
    //
    dbg!(&v);
    v.join("\n")
}

pub fn scan_for_includes(lines: Vec<&str>) -> HashMap<usize, &str> {
    let mut includes:HashMap<usize, &str> = HashMap::new();

    for (i, &line) in lines.iter().enumerate() {
        if is_include_line(line) {
            println!(">> {:?}, {:?}", i, line);
            includes.insert(i, line);
        }
    };
    includes
}

pub struct Includer {
    main_shader_path: PathBuf,
    main_shader_src: String,
    parts: BTreeMap<PathBuf, Vec<Part>>,
}

impl Includer {
    pub fn new(shader_path: PathBuf) -> Self {
        let mut shader_src = read_shader_src(shader_path.clone()).unwrap();
        Self {
            main_shader_src: shader_src,
            main_shader_path: shader_path,
            parts: Default::default(),
        }
    }

    pub fn process_includes(&mut self) {
        self.build_include_map(self.main_shader_path.clone(), self.main_shader_src.clone());
        let mut result: Vec<String> = Vec::new();
        // dbg!(&self.parts);
        // now go through the parts and inject all existing code into the right places
        for (include_name, parts) in self.parts.iter_mut() {

            println!("1> {:?}", include_name.file_name());

            for part in parts.iter_mut().rev() {

                println!("2> part {:?}", part.shader_src);
            }

            // // println!(">>> {:?}, {:?}\n", include_name.file_name(), parts);
            // // TODO: Store all sources in another hashmap to avoid re-opening the files
            // // it's stupid to re-open the files for reading but hey, here we go
            // let src = read_shader_src(include_name.to_owned()).unwrap();
            //
            // for part in parts.iter() {
            //     let k = include_statement_from_string(part.shader_name.to_owned());
            //     println!("2. >> {:?}\n", k);
            //     let st = part.shader_src.to_owned().replace(&k, src.as_str());
            //     println!("3. >>> {:?}\n", st);
            //     result.push(st);
            // }
        }

        // println!("4. ?> {:?}", result);

        // nice, that's all the includes, now include anything that's left in the main shader

    }

    pub fn build_include_map(&mut self, shader_path: PathBuf, mut shader_src: String) {

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
                if let Some(parts) = self.parts.get_mut(shader_path.as_path()) {
                    parts.push(part.clone());
                } else {
                    self.parts.insert(shader_path.to_owned(), vec![part.clone()]);
                }

                // self.parts.insert(inc_name.to_owned(), part.clone());
                self.build_include_map(part.to_owned().shader_path, part.to_owned().shader_src);
            });

        // dbg!(&self.parts);
    }
}