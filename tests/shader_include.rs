use skuggbox::shader::*;
use std::path::PathBuf;
use skuggbox::include_statement_from_string;
use std::collections::HashMap;

#[test]
fn shader_include() {

    use std::{println as info};

    let mut shader_path = PathBuf::from("./tests/files/main_test.glsl");
    let mut shader_src = read_shader_src(shader_path.clone()).unwrap();
    let mut result: HashMap<String, Part> = HashMap::new();
    build_include_map(shader_path.clone(), shader_src.clone(), &mut result);
    dbg!(&result);
    let final_shader = do_includes(shader_src, result.clone());

    let mut includer = Includer::new(shader_path);
    includer.process_includes();


    // let files_to_watch: Vec<PathBuf> = result.iter().map(|part| { part.to_owned().shader_path }).collect();
    // dbg!(&files_to_watch);
    // // now reverse the order of the list and include all files
    //
    // let final_shader: Vec<String> = result.iter().map(|src| {
    //     // println!("> {:?}", src.shader_name);
    //     // we know that if we don't have a shader name, then we have nothing to import
    //     let from = include_statement_from_string(src.shader_name.to_owned());
    //     // println!(">> {:?} {:?}", from, &src.shader_src);
    //     let new = src.to_owned().shader_src.replace(&from, " ");
    //     new.to_string()
    //
    // }).collect();

    // dbg!(&final_shader);



    // for part in result.iter() {
    //     println!("final parts {:?}", part);
    // }
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