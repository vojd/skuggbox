use std::ffi::CString;

pub fn cstr_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub fn cstr_to_str(cstr: &CString) -> String {
    cstr.to_string_lossy().to_string()
}

pub fn string_between<'v>(value: &'v str, start: &str, end: &str) -> &'v str {
    if let Some(start_idx) = value.rfind(start) {
        if let Some(end_idx) = value.rfind(end) {
            let from_byte = start_idx + start.len();
            let to_byte = end_idx;
            if from_byte < value.len() {
                return &value[from_byte..to_byte];
            }
        }
    }
    ""
}

/// Returns the shader name to be imported on a line of shader code
pub fn pragma_shader_name(line: &str) -> String {
    let shader_name = string_between(line, "include(", ")");
    shader_name.replace('\'', "").replace('\"', "")
}

pub fn include_statement_from_string(shader_name: String) -> String {
    format!("#pragma include({});", shader_name)
}

#[cfg(test)]
mod tests {
    use super::{pragma_shader_name, string_between};

    #[test]
    fn is_string_between() {
        let line = "#pragma include('some-shader.glsl')";
        assert_eq!(string_between(line, "include(", ")"), "'some-shader.glsl'");
    }

    #[test]
    fn is_shader_name() {
        let line_single_quotes = "#pragma include('some-shader.glsl')";
        assert_eq!(
            pragma_shader_name(line_single_quotes),
            "some-shader.glsl".to_string()
        );

        let line_double_quotes = "#pragma include(\"some-shader.glsl\")";
        assert_eq!(
            pragma_shader_name(line_double_quotes),
            "some-shader.glsl".to_string()
        );

        let line_no_quotes = "#pragma include(some-shader.glsl)";
        assert_eq!(
            pragma_shader_name(line_no_quotes),
            "some-shader.glsl".to_string()
        );
    }
}
