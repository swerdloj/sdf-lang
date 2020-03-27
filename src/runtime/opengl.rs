use std::ffi::{CStr, CString};
use gl::types::*;

// Code from `safe_gl` implementation from my WIP engine
// 
// Originally inspired by:
// http://nercury.github.io/rust/opengl/tutorial/2018/02/08/opengl-in-rust-from-scratch-00-setup.html

/* ---------------- Shaders ---------------- */

/// OpenGL shader object
/// - On `Drop`, will delete the shader
pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn from_vertex_source(source: &CStr) -> Result<Shader, String> {
        let id = Self::from_source(source, gl::VERTEX_SHADER)?;
        Ok(Shader {id})
    }

    pub fn from_fragment_source(source: &CStr) -> Result<Shader, String> {
        let id = Self::from_source(source, gl::FRAGMENT_SHADER)?;
        Ok(Shader {id})
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    // Creates and compiles a shader
    fn from_source(source: &CStr, shader_type: GLenum) -> Result<GLuint, String> {
        let id = unsafe {
            gl::CreateShader(shader_type)
        };

        unsafe {
            // 1: Single source string, NULL: Source string is null-terminated
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut success: GLint = 1;
        unsafe {
            // returns GL_TRUE if successful, otherwise GL_FALSE
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut length: GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut length);
            } 

            let error = create_empty_cstring(length as usize);

            unsafe {
                gl::GetShaderInfoLog(id, length, std::ptr::null_mut(), error.as_ptr() as *mut _);
            }

            return Err(error.to_string_lossy().into_owned());
        }

        Ok(id)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

/* ---------------- Programs ---------------- */

/// OpenGL render program
/// - On `Drop`, will delete the program
pub struct Program {
    id: GLuint,
}

// TODO: Use a builder pattern for passing shader sources & creating this
impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe {
            gl::CreateProgram()
        };

        for shader in shaders {
            unsafe {
                gl::AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl::LinkProgram(program_id);
        }

        let mut success: GLint = 1;

        unsafe {
            // returns GL_TRUE if successful, otherwise GL_FALSE
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut length: GLint = 0;

            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut length);
            }

            let error = create_empty_cstring(length as usize);

            unsafe {
                gl::GetProgramInfoLog(program_id, length, std::ptr::null_mut(), error.as_ptr() as *mut _);
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                // Note: Detaching a shader will automatically delete it if not attached to anything else
                gl::DetachShader(program_id, shader.id());
            }
        }

        Ok(Program { id: program_id })
    }

    pub fn id(&self) -> GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

/* ---------------- Vertex Array Object ---------------- */

/// Dummy vertex array object
/// - Calling `bind()` once will guarentee the existence of a VAO for rendering
/// - On `Drop`, will delete the VAO
pub struct DummyVAO {
    pub dummy_vao: GLuint,
}

impl DummyVAO {
    pub fn new() -> Self {
        let mut vertex_array_object: GLuint = 0;
        unsafe{gl::CreateVertexArrays(1, &mut vertex_array_object);}

        DummyVAO {
            dummy_vao: vertex_array_object,
        }
    }

    pub fn bind(&self) {
        unsafe{gl::BindVertexArray(self.dummy_vao);}
    }
}

impl Drop for DummyVAO {
    fn drop(&mut self) {
        unsafe{gl::DeleteVertexArrays(1, &self.dummy_vao);}
    }
}

/* ---------------- Utility ---------------- */

/// Creates an empty `CString` with a given length
fn create_empty_cstring(length: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(length + 1);
    buffer.extend([b' '].iter().cycle().take(length));

    unsafe {
        CString::from_vec_unchecked(buffer)
    }
}

/// Reads a file and converts it to a CString
pub fn read_file_to_cstring<P: AsRef<std::path::Path>>(file_path: P) -> CString {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(file_path).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    
    CString::new(buffer).unwrap()
}

/* ---------------- OpenGL API ---------------- */

/// Set viewport dimensions with (0, 0) as the bottom left
pub fn set_viewport(width: GLsizei, height: GLsizei) {
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

pub fn set_clear_color(r: GLfloat, g: GLfloat, b: GLfloat) {
    unsafe {
        gl::ClearColor(r, g, b, 1.0);
    }
}

pub fn clear(mask: GLbitfield) {
    unsafe {
        gl::Clear(mask);
    }
}