use crate::VERTEX_SHADER;
use egui::TextBuffer;
use glow::{HasContext, Program, UniformLocation};
use std::ffi::CString;
use std::fmt::Formatter;

pub fn cstr_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub fn cstr_to_str(cstr: &CString) -> String {
    cstr.to_string_lossy().to_string()
}

#[derive(Debug, Clone)]
pub enum ShaderError {
    CompilationError { error: String },
    FileError { error: String },
}

impl From<String> for ShaderError {
    fn from(err: String) -> Self {
        ShaderError::CompilationError { error: err }
    }
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Default, Debug)]
pub struct ShaderUniformLocations {
    pub resolution: Option<UniformLocation>,
    pub time: Option<UniformLocation>,
    pub time_delta: Option<UniformLocation>,
    pub mouse: Option<UniformLocation>,
    /// Direction of the mouse movement in vec2([-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0])
    pub mouse_dir: Option<UniformLocation>,
    /// Convenience uniform for quickly getting a-s-d-w movement into the shader.
    /// For more full control over the camera, use the `sb_camera_transform` instead
    pub cam_pos: Option<UniformLocation>,
    pub sb_camera_transform: Option<UniformLocation>,
    pub sb_color_a: Option<UniformLocation>,
}

#[derive(Clone)]
pub struct ShaderProgram {
    pub program: Program,
}

impl ShaderProgram {
    pub fn from_frag_src(
        gl: &glow::Context,
        fragment_src: String,
    ) -> anyhow::Result<Program, String> {
        unsafe {
            let vert_shader = compile_shader(gl, glow::VERTEX_SHADER, VERTEX_SHADER.as_str())?;
            macros::check_for_gl_error!(gl, "vertex_shader_compile");
            let frag_shader = compile_shader(gl, glow::FRAGMENT_SHADER, fragment_src.as_str())?;
            macros::check_for_gl_error!(gl, "fragment_shader_compile");

            let shader_sources = vec![vert_shader, frag_shader];
            let program = link_program(gl, &shader_sources).unwrap();
            log::debug!("Program created");
            gl.detach_shader(program, vert_shader);
            gl.detach_shader(program, frag_shader);

            gl.delete_shader(vert_shader);
            gl.delete_shader(frag_shader);

            Ok(program)
        }
    }

    /// # Safety
    ///
    /// Extract some common uniform locations using raw OpenGL calls, hence the unsafeness
    /// Values are set in the render function of app.rs
    pub unsafe fn uniform_locations(
        gl: &glow::Context,
        program: Program,
    ) -> ShaderUniformLocations {
        log::debug!("Reading uniform locations from program {:?}", program);
        let time = gl.get_uniform_location(program, "iTime");
        let resolution = gl.get_uniform_location(program, "iResolution");
        let time_delta = gl.get_uniform_location(program, "iTimeDelta");
        let mouse = gl.get_uniform_location(program, "iMouse");
        let mouse_dir = gl.get_uniform_location(program, "iMouseDir");
        let cam_pos = gl.get_uniform_location(program, "iCamPos");
        let sb_camera_transform = gl.get_uniform_location(program, "sbCameraTransform");
        let sb_color_a = gl.get_uniform_location(program, "sbColorA");

        let locations = ShaderUniformLocations {
            resolution,
            time,
            time_delta,
            mouse,
            mouse_dir,
            cam_pos,
            sb_camera_transform,
            sb_color_a,
        };

        log::debug!("shader locations {:?}", locations);

        locations
    }
}

pub(crate) unsafe fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    let shader = gl.create_shader(shader_type)?;

    gl.shader_source(shader, source);

    gl.compile_shader(shader);

    if gl.get_shader_compile_status(shader) {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(shader))
    }
}

pub(crate) unsafe fn link_program<'a, T: IntoIterator<Item = &'a glow::Shader>>(
    gl: &glow::Context,
    shaders: T,
) -> Result<glow::Program, String> {
    let program = gl.create_program()?;

    for shader in shaders {
        gl.attach_shader(program, *shader);
    }

    gl.link_program(program);

    if gl.get_program_link_status(program) {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(program))
    }
}
