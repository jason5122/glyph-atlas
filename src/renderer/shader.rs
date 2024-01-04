use std::ffi::CStr;

use crate::gl;
use crate::gl::types::*;

/// A wrapper for a shader program id, with automatic lifetime management.
#[derive(Debug)]
pub struct ShaderProgram(GLuint);

impl ShaderProgram {
    pub fn new(
        shader_header: Option<&str>,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
    ) -> Self {
        let vertex_shader = Shader::new(shader_header, gl::VERTEX_SHADER, vertex_shader);
        let fragment_shader = Shader::new(shader_header, gl::FRAGMENT_SHADER, fragment_shader);

        let program = unsafe { Self(gl::CreateProgram()) };

        let mut success: GLint = 0;
        unsafe {
            gl::AttachShader(program.id(), vertex_shader.id());
            gl::AttachShader(program.id(), fragment_shader.id());
            gl::LinkProgram(program.id());
            gl::GetProgramiv(program.id(), gl::LINK_STATUS, &mut success);
        }

        program
    }

    /// Get uniform location by name. Panic if failed.
    pub fn get_uniform_location(&self, name: &'static CStr) -> GLint {
        // This call doesn't require `UseProgram`.
        let ret = unsafe { gl::GetUniformLocation(self.id(), name.as_ptr()) };
        ret
    }

    /// Get the shader program id.
    pub fn id(&self) -> GLuint {
        self.0
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.0) }
    }
}

/// A wrapper for a shader id, with automatic lifetime management.
#[derive(Debug)]
struct Shader(GLuint);

impl Shader {
    fn new(shader_header: Option<&str>, kind: GLenum, source: &'static str) -> Self {
        let version_header = "#version 330 core\n";
        let mut sources = Vec::<*const GLchar>::with_capacity(3);
        let mut lengthes = Vec::<GLint>::with_capacity(3);
        sources.push(version_header.as_ptr().cast());
        lengthes.push(version_header.len() as GLint);

        if let Some(shader_header) = shader_header {
            sources.push(shader_header.as_ptr().cast());
            lengthes.push(shader_header.len() as GLint);
        }

        sources.push(source.as_ptr().cast());
        lengthes.push(source.len() as GLint);

        let shader = unsafe { Self(gl::CreateShader(kind)) };

        let mut success: GLint = 0;
        unsafe {
            gl::ShaderSource(
                shader.id(),
                lengthes.len() as GLint,
                sources.as_ptr().cast(),
                lengthes.as_ptr(),
            );
            gl::CompileShader(shader.id());
            gl::GetShaderiv(shader.id(), gl::COMPILE_STATUS, &mut success);
        }

        shader
    }

    fn id(&self) -> GLuint {
        self.0
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) }
    }
}
