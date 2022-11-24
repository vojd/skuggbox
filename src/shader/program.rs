use crate::{check_for_gl_error, VERTEX_SHADER};
use egui::TextBuffer;
use glow::{HasContext, Program, UniformLocation};
use std::ffi::CString;

pub fn cstr_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub fn cstr_to_str(cstr: &CString) -> String {
    cstr.to_string_lossy().to_string()
}

#[derive(Debug)]
pub enum ShaderError {
    CompilationError { error: String },
    FileError { error: String },
}

impl From<String> for ShaderError {
    fn from(err: String) -> Self {
        ShaderError::CompilationError { error: err }
    }
}
#[derive(Clone, Default)]
pub struct ShaderLocations {
    pub resolution: Option<UniformLocation>,
    pub time: Option<UniformLocation>,
    pub time_delta: Option<UniformLocation>,
    pub mouse: Option<UniformLocation>,
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
            check_for_gl_error!(&gl, "vertex_shader_compile");
            let frag_shader = compile_shader(gl, glow::FRAGMENT_SHADER, fragment_src.as_str())?;
            check_for_gl_error!(&gl, "fragment_shader_compile");

            let shader_sources = vec![vert_shader, frag_shader];
            let program = link_program(gl, &shader_sources).unwrap();
            log::debug!("Program created");
            gl.detach_shader(program, vert_shader);
            gl.detach_shader(program, frag_shader);

            gl.delete_shader(vert_shader);
            gl.delete_shader(frag_shader);

            return Ok(program);
        }
    }

    /// Extract some common uniform locations
    pub fn uniform_locations(gl: &glow::Context, program: Program) -> ShaderLocations {
        log::debug!("Reading uniform locations");
        unsafe {
            ShaderLocations {
                resolution: gl.get_uniform_location(program, "iResolution"),
                time: gl.get_uniform_location(program, "iTime"),
                time_delta: gl.get_uniform_location(program, "iTimeDelta"),
                mouse: gl.get_uniform_location(program, "iMouse"),
            }
        }
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
