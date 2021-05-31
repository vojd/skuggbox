use log::info;
use log::trace;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum GLSLType {
    // primitives
    Int,
    Float,
    Bool,

    // collections
    Vec2,
    Vec3,
    Vec4,
}

impl FromStr for GLSLType {
    type Err = UniformError;

    fn from_str(input: &str) -> Result<GLSLType, Self::Err> {
        match input {
            "bool" => Ok(GLSLType::Bool),
            "int" => Ok(GLSLType::Int),
            "float" => Ok(GLSLType::Float),

            "vec2" => Ok(GLSLType::Vec2),
            "vec3" => Ok(GLSLType::Vec3),
            "vec4" => Ok(GLSLType::Vec4),
            _ => Err(UniformError::TypeError),
        }
    }
}

/// hmm... might be overkill
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum GLSLValue {
    Int(i32),
    Float(f32),
    Bool(bool),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Uniform {
    name: String,
    glsl_type: GLSLType,
    value: Option<GLSLValue>,
}

#[derive(Debug)]
pub enum UniformError {
    ParseError,
    TypeError,
}

/// Read the given shader and extract all existing uniform values
/// Append these to the uniform handler
pub fn read_uniforms(shader_path: PathBuf) -> Vec<Uniform> {
    let mut uniforms: Vec<Uniform> = vec![];

    trace!("Reading uniforms");
    let file = File::open(shader_path.as_os_str());
    if let Ok(f) = file {
        let f = BufReader::new(f);

        for line in f.lines().flatten() {
            if is_uniform(line.clone()) {
                info!("is uniform");
                if let Ok(uniform) = extract_uniform(line) {
                    uniforms.push(uniform);
                };
            }
        }
    }

    uniforms
}

fn is_uniform(line: String) -> bool {
    line.trim().starts_with("uniform")
}

/// Parse a uniform from the shader and return an internal Rust representation.
/// Uniforms holds no initial values.
fn extract_uniform(line: String) -> Result<Uniform, UniformError> {
    if !is_uniform(line.clone()) {
        return Err(UniformError::ParseError);
    }

    let parts: Vec<String> = line
        .trim()
        .split_whitespace()
        .take(3)
        .map(|s| s.to_string())
        .collect();
    // TODO: Should not be necessary. Do the check in line above
    if !parts.len() == 3 {
        return Err(UniformError::ParseError);
    }

    let mut uniform_name = parts.get(2).ok_or(UniformError::ParseError)?.to_string();
    if uniform_name.ends_with(";") {
        uniform_name.truncate(uniform_name.len() - 1)
    }
    let u_type = parts.get(1).ok_or(UniformError::ParseError)?;
    let uniform_type = GLSLType::from_str(u_type)?;

    Ok(Uniform {
        name: uniform_name,
        glsl_type: uniform_type,
        value: None,
    })
}

#[cfg(test)]
mod tests {
    use crate::uniforms::{extract_uniform, is_uniform, GLSLType};
    use std::str::FromStr;

    #[test]
    fn is_uniform_line() {
        let line = "uniform vec2 value;".to_string();
        assert!(is_uniform(line));

        let line_with_spaces = "       uniform vec2 my_uniform;".to_string();
        assert!(is_uniform(line_with_spaces));

        let line_with_comments = "// uniform;".to_string();
        assert!(!is_uniform(line_with_comments));

        let line_with_suffix_comment = "uniform vec2 value; // a comment".to_string();
        assert!(is_uniform(line_with_suffix_comment));
    }

    #[test]
    fn extract_uniform_valid() {
        let line = "uniform vec2 value        ; //;;".to_string();
        let uniform = extract_uniform(line).unwrap_or_else(|_| panic!("Uniform not ok"));
        assert_eq!(uniform.name, "value");
        assert_eq!(uniform.glsl_type, GLSLType::from_str("vec2").unwrap());
    }

    #[test]
    fn extract_uniform_valid_no_trailing_semicolon() {
        let line = "uniform vec2 value;".to_string();
        let uniform = extract_uniform(line).unwrap_or_else(|_| panic!("Uniform not ok"));
        assert_eq!(uniform.name, "value");
        assert_eq!(uniform.glsl_type, GLSLType::from_str("vec2").unwrap());
    }

    #[test]
    fn extract_uniform_errors() {
        let line = "uniform ; //;;".to_string();
        let uniform = extract_uniform(line);
        assert!(uniform.is_err());
        // assert_eq!(uniform, Err(UniformError::ParseError));
    }

    #[test]
    fn glsl_type_from_string() {
        assert_eq!(GLSLType::from_str("vec2").unwrap(), GLSLType::Vec2);
        assert_eq!(GLSLType::from_str("vec3").unwrap(), GLSLType::Vec3);
        assert_eq!(GLSLType::from_str("vec4").unwrap(), GLSLType::Vec4);
    }
}
