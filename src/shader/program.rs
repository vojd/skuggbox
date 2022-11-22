use crate::VERTEX_SHADER;
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

#[derive(Clone)]
pub struct ShaderLocations {
    pub resolution: i32,
    pub time: i32,
    pub time_delta: i32,
    pub mouse: i32,
}

impl Default for ShaderLocations {
    fn default() -> Self {
        Self {
            resolution: -1,
            time: -1,
            time_delta: -1,
            mouse: -1,
        }
    }
}

#[derive(Clone)]
pub struct ShaderProgram {
    pub id: gl::types::GLuint,
}

impl ShaderProgram {
    pub fn is_valid(&self) -> bool {
        self.id < u32::MAX
    }

    pub fn free(&mut self) -> bool {
        if !self.is_valid() {
            return false;
        }

        unsafe {
            gl::DeleteProgram(self.id);
        }

        self.id = u32::MAX;
        true
    }

    /// Constructs a fragment shader from string.
    /// Adds on a simple vertex shader.
    pub fn from_frag_src(fragment_src: String) -> anyhow::Result<Self, ShaderError> {
        let vert_shader_id = shader_from_string(String::from(VERTEX_SHADER), gl::VERTEX_SHADER)?;
        let frag_shader_id = shader_from_string(fragment_src, gl::FRAGMENT_SHADER)?;
        log::info!(
            "Creating shader program: {} {}",
            vert_shader_id,
            frag_shader_id
        );
        let id = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(id, vert_shader_id);
            gl::AttachShader(id, frag_shader_id);
            gl::LinkProgram(id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = cstr_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }
            log::error!("linker error {}", error.to_string_lossy());
            panic!("linker error");
        }

        unsafe {
            gl::DetachShader(id, vert_shader_id);
            gl::DetachShader(id, frag_shader_id);
        }

        Ok(Self { id })
    }

    pub fn from_source(
        source: String,
        shader_type: gl::types::GLuint,
    ) -> anyhow::Result<Self, ShaderError> {
        let id = shader_from_string(source, shader_type)?;
        Ok(ShaderProgram { id })
    }

    #[allow(temporary_cstring_as_ptr)]
    pub fn uniform_location(&self, uniform_name: &str) -> i32 {
        unsafe { gl::GetUniformLocation(self.id, CString::new(uniform_name).unwrap().as_ptr()) }
    }

    /// Extract some common uniform locations
    pub fn uniform_locations(&self) -> ShaderLocations {
        ShaderLocations {
            resolution: self.uniform_location("iResolution"),
            time: self.uniform_location("iTime"),
            time_delta: self.uniform_location("iTimeDelta"),
            mouse: self.uniform_location("iMouse"),
        }
    }
}

impl Default for ShaderProgram {
    fn default() -> Self {
        Self { id: u32::MAX }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

fn shader_from_string(
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
