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

#[test]
fn test_camera_integration() {
    let shader_path = PathBuf::from("./tests/files/camera_integration_test.glsl");
    let mut pre_processor = PreProcessor::new(shader_path);
    pre_processor.reload();
    dbg!(&pre_processor.shader_src);

    let lines = pre_processor.shader_src.lines().count();
    assert!(lines > 3, "No integration added");
}
