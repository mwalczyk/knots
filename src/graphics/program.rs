use gl;
use gl::types::*;

use cgmath;
use cgmath::{Array, Matrix, Matrix4, Vector2, Vector3, Vector4};

use std::ffi::CString;
use std::ptr;
use std::str;

pub struct UniformEntry {
    name: String,
    location: GLint,
}

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn get_id(&self) -> GLuint {
        self.id
    }

    /// Compiles a shader of type `stage` from the source held in `src`.
    fn compile_shader(src: &String, stage: GLenum) -> Result<GLuint, String> {
        let shader;
        unsafe {
            shader = gl::CreateShader(stage);

            // Attempt to compile the shader
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            // Get the compile status
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);

                // Subtract 1 to skip the trailing null character
                buffer.set_len((len as usize) - 1);

                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );

                let error = String::from_utf8(buffer)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8");

                return Err(error);
            }
        }

        Ok(shader)
    }

    /// Links the shader program.
    fn link_program(vs: GLuint, fs: GLuint) -> Result<GLuint, String> {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);

            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // If there was an error, return the error string
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = Vec::with_capacity(len as usize);

                // Subtract 1 to skip the trailing null character
                buffer.set_len((len as usize) - 1);

                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                gl::DeleteShader(fs);
                gl::DeleteShader(vs);

                let error = String::from_utf8(buffer)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8");
                return Err(error);
            }

            Ok(program)
        }
    }

    /// Compiles a two-stage (vertex + fragment) shader from source.
    pub fn from_sources(vs_src: String, fs_src: String) -> Result<Program, String> {
        // Make sure that compiling each of the shaders was successful
        let vs_id = Program::compile_shader(&vs_src, gl::VERTEX_SHADER)?;
        let fs_id = Program::compile_shader(&fs_src, gl::FRAGMENT_SHADER)?;

        let id = Program::link_program(vs_id, fs_id)?;

        Ok(Program { id })
    }

    /// Binds this shader program.
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Unbinds this shader program.
    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn uniform_bool(&self, name: &str, value: bool) {
        self.uniform_1i(name, value as i32);
    }

    pub fn uniform_1i(&self, name: &str, value: i32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform1i(self.id, location, value as gl::types::GLint);
        }
    }

    pub fn uniform_1ui(&self, name: &str, value: u32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform1ui(self.id, location, value as gl::types::GLuint);
        }
    }

    pub fn uniform_1f(&self, name: &str, value: f32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform1f(self.id, location, value as gl::types::GLfloat);
        }
    }

    pub fn uniform_2i(&self, name: &str, value: &cgmath::Vector2<i32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform2iv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_2ui(&self, name: &str, value: &cgmath::Vector2<u32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform2uiv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_2f(&self, name: &str, value: &cgmath::Vector2<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform2fv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_3i(&self, name: &str, value: &cgmath::Vector3<i32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform3iv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_3ui(&self, name: &str, value: &cgmath::Vector3<u32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform3uiv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_3f(&self, name: &str, value: &cgmath::Vector3<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform3fv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_4i(&self, name: &str, value: &cgmath::Vector4<i32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform4iv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_4ui(&self, name: &str, value: &cgmath::Vector4<u32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform4uiv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_4f(&self, name: &str, value: &cgmath::Vector4<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniform4fv(self.id, location, 1, value.as_ptr());
        }
    }

    pub fn uniform_matrix_4f(&self, name: &str, value: &cgmath::Matrix4<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, CString::new(name).unwrap().as_ptr());
            gl::ProgramUniformMatrix4fv(self.id, location, 1, gl::FALSE, value.as_ptr());
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
