use skuggbox::shader::*;
use std::path::PathBuf;

#[test]
fn test_shader_reload() {
    let shader_path = PathBuf::from("./tests/files/main_test.glsl");
    let mut pre_processor = PreProcessor::new(shader_path);
    pre_processor.reload();
    dbg!(&pre_processor.shader_src);
    assert_eq!(pre_processor.shader_src.contains("#pragma include"), false);
}
