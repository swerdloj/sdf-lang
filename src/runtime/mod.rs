extern crate sdl2;
extern crate gl;


pub mod application;
pub mod opengl;
pub mod timing;


use gl::types::*;

use crate::parse;

use std::path::PathBuf;
use std::collections::{HashSet, HashMap};

pub struct Runtime {
    sdf_path: PathBuf,
    
    /// name -> (location, type)
    uniforms: HashMap<String, (usize, parse::ast::TypeSpecifier)>,

    features: HashSet<String>,

    /// One VAO must be bound to draw anything
    _dummy_vao: opengl::DummyVAO,
}

impl Runtime {
    pub fn new<P: Into<PathBuf>>(sdf_path: P) -> Self {
        let dummy_vao = opengl::DummyVAO::new();
        dummy_vao.bind();

        Runtime {
            sdf_path: sdf_path.into(),
            uniforms: HashMap::new(),
            features: HashSet::new(),
            _dummy_vao: dummy_vao,
        }
    }

    // TODO: Consider having this return a result
    // FIXME: Calling this function multiple times doubles memory usage once for no reason
    //        but only if the shader changed (even though it is recompiled regardless).
    //        I can't track it to a single object in this function, and this happens randomly.
    pub fn reload_shader(&mut self) {      
        let input = parse::Input::from_path(&self.sdf_path).unwrap();
        if let crate::parse::context::ShaderType::Fragment = input.shader_type {} else {
            println!("\nError: Shader was not declared as a fragment shader");
            return;
        }
        
        let mut ast = parse::parse(&input);
        if ast.is_err() {
            println!("\nA shader error prevented reloading: ");
            println!("{}\n", ast.err().unwrap());
            return;
        }

        let context = crate::translate::validate(ast.as_mut().unwrap(), &input);
        if context.is_err() {
            println!("\nA shader error prevented reloading: ");
            println!("{}\n", context.err().unwrap());
            return;
        }

        let c = context.unwrap();
        let glsl = crate::translate::translate(&ast.unwrap(), &c);

        let default_vertex_shader = opengl::Shader::from_vertex_source(
            &opengl::read_file_to_cstring("./src/runtime/shaders/full_screen.vert")
        ).unwrap();

        let fragment_shader = opengl::Shader::from_fragment_source(
            &std::ffi::CString::new(glsl).unwrap()
        ).unwrap();

        let gl_program = opengl::Program::from_shaders(&[
            default_vertex_shader, fragment_shader
        ]).unwrap();

        c.get_uniform_map(&mut self.uniforms);
        c.get_features(&mut self.features);

        gl_program.set_used();
    }

    fn get_uniform_location(&self, name: &str) -> Result<i32, String> {
        if let Some(location) = self.uniforms.get(name) {
            Ok(location.0 as i32)
        } else {
            Err(format!("No such uniform exists, '{}'", name))
        }
    }

    /// Update the shader's window dimension value
    pub fn set_window_dimensions(&mut self, width: i32, height: i32) {
        if self.features.contains("window_dimensions") {
            unsafe {
                gl::Uniform2fv(self.get_uniform_location("window_dimensions").unwrap(), 1, &[width as GLfloat, height as GLfloat] as *const _);
            }
        }
    }

    /// Update the shader's time value
    pub fn set_time(&mut self, seconds: f32) {
        if self.features.contains("time") {
            unsafe {
                gl::Uniform1f(self.get_uniform_location("time").unwrap(), seconds);
            }
        }
    }

    /// Draw the fragment shader's output
    pub fn render(&self) {
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}