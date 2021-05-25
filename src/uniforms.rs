use log::info;
use log::trace;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Read the given shader and extract all existing uniform values
/// Append these to the uniform handler
pub fn read_uniforms(shader_path: PathBuf) {
    trace!("Read uniforms");
    let file = File::open(shader_path.as_os_str());
    if let Ok(f) = file {
        let f = BufReader::new(f);

        for line in f.lines().flatten() {
            if is_uniform(line.clone()) {
                info!("is uniform");
                extract_uniform(line);
            }
        }
    }
}

fn is_uniform(line: String) -> bool {
    line.trim().starts_with("uniform")
}

fn extract_uniform(uniform_line: String) {
    let mut p = uniform_line.split_whitespace();

    if let (Some(_), Some(data_type), Some(uniform_name)) = (p.next(), p.next(), p.next()) {
        info!("{} {}", data_type, uniform_name);
        println!("{} {}", data_type, uniform_name);
    }
}

#[cfg(test)]
mod tests {
    use crate::uniforms::{extract_uniform, is_uniform};

    #[test]
    fn is_uniform_line() {
        let line = "uniform vec2 value;".to_string();
        assert!(is_uniform(line));

        let line_with_spaces = "       uniform vec2 value;".to_string();
        assert!(is_uniform(line_with_spaces));

        let line_with_comments = "// uniform;".to_string();
        assert!(!is_uniform(line_with_comments));

        let line_with_suffix_comment = "uniform vec2 value; // a comment".to_string();
        assert!(is_uniform(line_with_suffix_comment));
    }
    #[test]
    fn uniform_extraction() {
        let line = "uniform vec2 value;".to_string();
        let uniform = extract_uniform(line);
    }
}
