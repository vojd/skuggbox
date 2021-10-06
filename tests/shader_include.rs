use skuggbox::shader::*;
use std::path::PathBuf;
use skuggbox::include_statement_from_string;

#[test]
fn shader_include() {

    use std::{println as info};

    let mut shader_path = PathBuf::from("./tests/files/main_test.glsl");
    let mut shader_src = read_shader_src(shader_path.clone()).unwrap();
    let mut result: Vec<Part> = vec![];
    create_include_list(shader_path, shader_src, result.as_mut());
    result.sort();
    result.dedup_by(|a, b| a.shader_path == b.shader_path);

    let files_to_watch: Vec<PathBuf> = result.iter().map(|part| { part.to_owned().shader_path }).collect();
    dbg!(&files_to_watch);
    // now reverse the order of the list and include all files

    let final_shader: Vec<String> = result.iter().map(|src| {
        // we know that if we don't have a shader name, then we have nothing to import
        if let Some(name) = src.to_owned().shader_name {
            let new = src.to_owned().shader_src.replace(&include_statement_from_string(name), "");
            println!("new {:?}", new);
        }
        String::from("helo")

    }).collect();

    dbg!(&final_shader);



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