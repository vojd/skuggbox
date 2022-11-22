use crate::VERTEX_SHADER;
use egui::TextBuffer;
use glow::{HasContext, Program, Shader, UniformLocation};
use std::any::Any;
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
    pub fn is_valid(&self) -> bool {
        // TODO(mathias): fix!
        // self.id < u32::MAX
        // just temp added
        false
    }

    pub fn free(&mut self) -> bool {
        if !self.is_valid() {
            return false;
        }

        unsafe {
            // TODO(mathias): fix!
            // gl::DeleteProgram(self.id);
        }

        //TODO(mathias): fix!
        // self.id = u32::MAX;
        true
    }

    pub fn from_frag_src(
        gl: &glow::Context,
        fragment_src: String,
    ) -> anyhow::Result<Program, ShaderError> {
        // let vert_shader_id =
        //     glow_shader_from_string(&gl, String::from(VERTEX_SHADER), gl::VERTEX_SHADER)?;
        // let frag_shader_id = glow_shader_from_string(&gl, fragment_src, gl::FRAGMENT_SHADER)?;

        unsafe {
            let vert_shader =
                compile_shader(gl, glow::VERTEX_SHADER, VERTEX_SHADER.as_str()).unwrap();
            let frag_shader =
                compile_shader(gl, glow::FRAGMENT_SHADER, fragment_src.as_str()).unwrap();

            let stuff = vec![vert_shader, frag_shader];
            let program = link_program(gl, &stuff).unwrap();
            gl.detach_shader(program, vert_shader);
            gl.detach_shader(program, frag_shader);

            gl.delete_shader(vert_shader);
            gl.delete_shader(frag_shader);

            return Ok(program);
        }

        // TODO(mathias): No need to return Self, just return the program
        // Err(ShaderError::CompilationError {
        //     error: "failed".to_string(),
        // })
    }

    /// Constructs a fragment shader from string.
    /// Adds on a simple vertex shader.
    // pub fn from_frag_src_2(
    //     gl: &glow::Context,
    //     fragment_src: String,
    // ) -> anyhow::Result<Program, ShaderError> {
    //     let vert_shader_id = shader_from_string(
    //         gl: &glow::Context,
    //         String::from(VERTEX_SHADER),
    //         glow::VERTEX_SHADER,
    //     )?;
    //     let frag_shader_id =
    //         shader_from_string(gl: &glow::Context, fragment_src, glow::FRAGMENT_SHADER)?;
    //     log::info!(
    //         "Creating shader program: {} {}",
    //         vert_shader_id,
    //         frag_shader_id
    //     );
    //     let id = unsafe { gl.create_program() };
    //     unsafe {
    //         // gl::AttachShader(id, vert_shader_id);
    //         // gl::AttachShader(id, frag_shader_id);
    //         // gl::LinkProgram(id);
    //
    //         gl.attach_shader(id, vert_shader_id);
    //     }
    //
    //     let mut success: gl::types::GLint = 1;
    //     unsafe {
    //         gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
    //     }
    //
    //     if success == 0 {
    //         let mut len: gl::types::GLint = 0;
    //         unsafe {
    //             gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
    //         }
    //
    //         let error = cstr_with_len(len as usize);
    //
    //         unsafe {
    //             gl::GetProgramInfoLog(
    //                 id,
    //                 len,
    //                 std::ptr::null_mut(),
    //                 error.as_ptr() as *mut gl::types::GLchar,
    //             );
    //         }
    //         log::error!("linker error {}", error.to_string_lossy());
    //         panic!("linker error");
    //     }
    //
    //     unsafe {
    //         gl::DetachShader(id, vert_shader_id);
    //         gl::DetachShader(id, frag_shader_id);
    //     }
    //
    //     Ok(Self { id })
    // }

    // pub fn from_source(
    //     source: String,
    //     shader_type: gl::types::GLuint,
    // ) -> anyhow::Result<Self, ShaderError> {
    //     let id = shader_from_string(source, shader_type)?;
    //     Ok(ShaderProgram { id })
    // }

    // #[allow(temporary_cstring_as_ptr)]
    // pub fn uniform_location(
    //     &self,
    //     gl: &glow::Context,
    //     program: &Program,
    //     uniform_name: &str,
    // ) -> UniformLocation {
    //     unsafe { gl.get_uniform_location(program, uniform_name).unwrap() }
    // }

    /// Extract some common uniform locations
    pub fn uniform_locations(gl: &glow::Context, program: Program) -> ShaderLocations {
        unsafe {
            ShaderLocations {
                // resolution: self.uniform_location(&gl, &program, "iResolution"),
                // time: self.uniform_location(&gl, &program, "iTime"),
                // time_delta: self.uniform_location(&gl, &program, "iTimeDelta"),
                // mouse: self.uniform_location(&gl, &program, "iMouse"),
                resolution: gl.get_uniform_location(program.clone(), "iResolution"),
                time: gl.get_uniform_location(program.clone(), "iTime"),
                time_delta: gl.get_uniform_location(program.clone(), "iTimeDelta"),
                mouse: gl.get_uniform_location(program.clone(), "iMouse"),
            }
        }
    }
}

// impl Default for ShaderProgram {
//     fn default() -> Self {
//         Self { id: u32::MAX }
//     }
// }

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        // TODO(mathias): FIX!
        // unsafe {
        //     // gl::DeleteProgram(self.id);
        // }
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

// fn glow_shader_from_string(
//     gl: &glow::Context,
//     source: String,
//     shader_type: gl::types::GLuint,
// ) -> anyhow::Result<Shader, ShaderError> {
//     let c_src = CString::new(source).expect("Could not convert source to CString");
//
//     // check for includes
//
//     let shader = unsafe { gl.create_shader(shader_type).unwrap() };
//
//     unsafe {
//         gl::ShaderSource(id, 1, &c_src.as_ptr(), std::ptr::null());
//         gl::CompileShader(id);
//     }
//
//     let mut success: gl::types::GLint = 1;
//     unsafe {
//         gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
//     }
//
//     // in case something failed we want to extract the error and return it
//     if success == 0 {
//         // fetch the required buffer length
//         let mut len: gl::types::GLint = 0;
//         unsafe {
//             gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
//         }
//
//         let error = cstr_with_len(len as usize);
//
//         unsafe {
//             gl::GetShaderInfoLog(
//                 id,
//                 len,
//                 std::ptr::null_mut(),
//                 error.as_ptr() as *mut gl::types::GLchar,
//             )
//         }
//
//         // Prints the compilation error to console
//         log::error!("{}", error.to_string_lossy());
//         return Err(ShaderError::CompilationError {
//             error: cstr_to_str(&error),
//         });
//     }
//
//     Ok(id)
// }

fn shader_from_string(
    gl: &glow::Context,
    source: String,
    shader_type: gl::types::GLuint,
) -> anyhow::Result<gl::types::GLuint, ShaderError> {
    let c_src = CString::new(source).expect("Could not convert source to CString");

    // check for includes

    let id = unsafe { gl::CreateShader(shader_type) };

    unsafe {
        gl::ShaderSource(id, 1, &c_src.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    // in case something failed we want to extract the error and return it
    if success == 0 {
        // fetch the required buffer length
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = cstr_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            )
        }

        // Prints the compilation error to console
        log::error!("{}", error.to_string_lossy());
        return Err(ShaderError::CompilationError {
            error: cstr_to_str(&error),
        });
    }

    Ok(id)
}
