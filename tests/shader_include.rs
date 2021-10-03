use skuggbox::shader::*;
use std::path::PathBuf;


#[test]
fn shader_include() {

    let mut col :Vec<Part> = vec![];
    let source_file = PathBuf::from("./tests/files/main_test.glsl");
    let source = t(source_file, &mut col);
    dbg!(&source);
    dbg!("==========");
    dbg!(&col);
    // let shader_src = read_shader_src(source_file).unwrap();
    // let inc = included_files_in_file(shader_src);
    // dbg!(&inc);
    // dbg!("shader include test");
}