pub mod template;

use crate::parse::ast::*;

/*

    This function should transform the AST into valid GLSL code.
    In order to maintain scopes, types, etc., this function utilizes a context.

    That context must track valid identifiers 
        (functions, primitives, structs, builtins, etc.)
    This is done within the parser, meaning a parsed file has an associated context.

    TODO: A lot of this could be moved to the parser

*/
pub fn translate(ast: &AST, context: &crate::parse::context::Context) -> String {
    // Unfortunately, GLSL requires functions to be declared in order of use
    // sdf-lang can compensate for this by forward declaring all functions
    // OR sdf-lang can also require forced ordering

    let mut glsl = String::new();

    // TODO: Allow user to specify version
    glsl.push_str("#version 450 core\n\n");
    // glsl.push_str("out vec4 __out__color;\n\n");

    glsl.push_str(&template::uniforms(context.uniforms()));

    // TODO: Allow let statements at global scope for global variables
    for item in ast {
        // `Item`s always have global scopes
        match item {
            Item::Struct { name, fields } => {
                glsl.push_str(&template::structure(name, fields));
            }

            Item::Function { name, parameters, return_type, statements } => {
                // TODO: Body statements
                glsl.push_str(&template::function(name, parameters, &return_type, statements));
            }

            Item::Scene { name, statements } => {
                glsl.push_str(&template::scene(name));
            }
        }
    }

    glsl
}