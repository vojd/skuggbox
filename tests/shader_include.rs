use skuggbox::include_statement_from_string;
use skuggbox::shader::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn shader_include() {
    let mut shader_path = PathBuf::from("./tests/files/main_test.glsl");
    let mut includer = Includer::new(shader_path);
    includer.process_includes();

}
// #[test]
// fn test_included_files() {
//     let mut shader_path = PathBuf::from("./tests/files/main_test.glsl");
//     let mut shader_src = read_shader_src(shader_path.clone()).unwrap();
//     let r = included_files(shader_path, shader_src);
//     dbg!(r);
// }

// #[test]
fn test_scan_for_includes() {
    let mut shader_path = PathBuf::from("./tests/files/main_test.glsl");
    let mut shader_src = read_shader_src(shader_path.clone()).unwrap();
    let result = scan_for_includes(shader_src.lines().collect::<Vec<&str>>());
    dbg!(&result);
}
