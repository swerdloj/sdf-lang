extern crate sdl2;
extern crate gl;

use gl::types::*;

pub mod application;
pub mod opengl;

use std::path::PathBuf;
use std::collections::HashMap;

pub struct Runtime {
    sdf_file: PathBuf,
    /// name -> (location, type)
    uniforms: HashMap<String, (usize, String)>,
    /// One VAO must be bound to draw anything
    _dummy_vao: opengl::DummyVAO,
}

impl Runtime {
    pub fn new<P: Into<PathBuf>>(sdf_file: P) -> Self {
        let dummy_vao = opengl::DummyVAO::new();
        dummy_vao.bind();

        Runtime {
            sdf_file: sdf_file.into(),
            uniforms: HashMap::new(),
            _dummy_vao: dummy_vao,
        }
    }

    pub fn reload_shader(&mut self) {
        let input = std::fs::read_to_string(&self.sdf_file).unwrap();
        
        let mut ast = crate::parse::parse(&input).map_err(|e| {
            println!("{}", e);
            return;
        }).unwrap();

        let context = crate::translate::validate(&mut ast);
        let glsl = crate::translate::translate(&ast, &context);

        let default_vertex_shader = opengl::Shader::from_vertex_source(
            &opengl::read_file_to_cstring("./src/runtime/shaders/full_screen.vert")
        ).unwrap();

        let fragment_shader = opengl::Shader::from_fragment_source(
            &std::ffi::CString::new(glsl).unwrap()
        ).unwrap();

        let gl_program = opengl::Program::from_shaders(&[
            default_vertex_shader, fragment_shader
        ]).unwrap();

        context.get_uniform_map(&mut self.uniforms);

        gl_program.set_used();
    }

    fn get_uniform_location(&self, name: &str) -> i32 {
        self.uniforms.get(name).unwrap().0 as i32
    }

    /// Update the shader's window dimension value
    pub fn set_window_dimensions(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Uniform2fv(self.get_uniform_location("window_dimensions"), 1, &[width as GLfloat, height as GLfloat] as *const _);
        }
    }

    /// Update the shader's time value
    pub fn set_time(&mut self, seconds: f32) {
        unsafe {
            gl::Uniform1f(self.get_uniform_location("time"), seconds);
        }
    }

    /// Draw the fragment shader's output
    pub fn render(&self) {
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}