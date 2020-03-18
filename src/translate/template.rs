use crate::parse::ast::*;

pub fn uniforms(uniforms: &std::collections::HashSet<(String, String)>) -> String {
    let mut glsl = String::new();

    for (index, (name, ty)) in uniforms.iter().enumerate() {
        glsl.push_str(&format!("layout(location = {}) uniform {} {};\n", index, ty, name));
    }

    if uniforms.len() >= 1 {
        glsl.push('\n');
    }

    glsl
}

pub fn structure(name: &str, fields: &Vec<(String, String, Option<Literal>)>) -> String {
    let mut glsl = String::new();

    glsl.push_str(&format!("struct {} {{\n", name));

    for (field, ty, _defaults) in fields {
        glsl.push_str(&format!("\t{} {};\n", ty, field));
    }

    // Remove trailing "\n"
    glsl.pop();

    glsl.push_str("\n}\n\n");

    glsl
}

pub fn function(name: &str, parameters: &Vec<(String, String)>, return_type: &str) -> String {
    let mut glsl = String::new();

    let mut param_string = String::new();
    for (param_name, param_type) in parameters {
        param_string.push_str(&format!("{} {}, ", param_type, param_name));
    }

    // Remove trailing ", "
    if parameters.len() > 0 {
        param_string.pop();
        param_string.pop();
    }

    glsl.push_str(&format!("{} {}({}) {{\n", return_type, name, param_string));

    glsl.push_str("\n}\n\n");

    glsl
}

// TODO: This
pub fn scene(name: &str) -> String {
    let mut glsl = String::new();

    glsl.push_str(&format!("RayResult __scene__{}(vec3 point) {{\n", name));
    glsl.push_str("\tfloat distance; uint hit;");

    glsl.push_str("\n}\n\n");

    glsl
}