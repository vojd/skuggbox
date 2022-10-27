// use skuggbox::shader::*;
// use std::path::PathBuf;

//TODO(mathias): Fix these tests sometime. They broke during the big pre-processor war of -22
// #[test]
// fn test_shader_reload() {
//     let shader_path = PathBuf::from("./tests/files/main_test.glsl");
//     let config = PreProcessorConfig {
//         use_camera_integration: false,
//     };
//     let mut pre_processor = PreProcessor::new(config);
//     pre_processor.load_file(&shader_path);
//     pre_processor.pre_process();
//     dbg!(&pre_processor);
//     assert_eq!(pre_processor.shader_src.contains("#pragma include"), false);
// }
//
// #[test]
// fn test_camera_integration() {
//     let shader_path = PathBuf::from("./tests/files/camera_integration_test.glsl");
//     let config = PreProcessorConfig {
//         use_camera_integration: false,
//     };
//     let mut pre_processor = PreProcessor::new(config);
//     pre_processor.load_file(&shader_path);
//     pre_processor.pre_process();
//     dbg!(&pre_processor.);
//
//     let lines = pre_processor.shader_src.lines().count();
//     assert!(lines > 3, "No integration added");
// }
