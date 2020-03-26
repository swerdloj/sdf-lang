use gl::types::*;

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