use skuggbox::shader::*;
use std::path::PathBuf;


#[test]
fn shader_include() {

    use std::{println as info};

    let mut col :Vec<Part> = vec![];
    let mut source_file = PathBuf::from("./tests/files/main_test.glsl");
    let mut shader_src = read_shader_src(source_file.clone()).unwrap();
    let source = t(shader_src, source_file, &mut col);
    dbg!(&source);
    println!("hello");
    eprintln!("hi!");
    // dbg!("==========");
    // dbg!(&col);
    // let shader_src = read_shader_src(source_file).unwrap();
    // let inc = included_files_in_file(shader_src);
    // dbg!(&inc);
    // dbg!("shader include test");
}