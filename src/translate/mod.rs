pub mod template;

use crate::exit_with_message;
use crate::parse::ast::*;

use std::collections::{HashMap, HashSet};

/* 
    TODO: Get rid of `.clone()` calls
*/

// See https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)

struct StructSignature {
    name: String,
    // Fields and types with optional defaults
    // TODO: Support more than just literals as defaults (custom types, vecs, etc.)
    fields: Vec<(String, String, Option<Literal>)>,
}

struct FunctionSignature {
    name: String,
    // (field_name, field_type)
    parameters: Vec<(String, String)>,
    return_type: String,
}

pub struct Context {
    functions: HashMap<String, FunctionSignature>,
    structs: HashMap<String, StructSignature>,
    primitive_types: HashSet<&'static str>,
}

impl Context {
   pub fn new() -> Self {
        macro_rules! declare_primitive_types {
            ( $( $x:expr ),+ ) => {{
                    let mut types = HashSet::new();
                    $( types.insert($x); )*
                    types
                }};
        }

        // TODO: vectors, arrays, and matrices will be treated uniquely
        // see http://www.shaderific.com/glsl-types
        let primitive_types = declare_primitive_types!(
            "float", "double", "bool", "int", "uint", "sampler2D", "samplerCube"
        );

        // see http://www.shaderific.com/glsl-functions
        let mut functions = HashMap::new();
        
        // functions.insert("length", ...);

        Context {
            functions,
            structs: HashMap::new(),
            primitive_types,
        }
    }

    // TODO: Support more than just Literals as default values
    pub fn declare_struct(&mut self, name: String, fields: Vec<(String, String, Option<Literal>)>) {
        let signature = StructSignature {
            name: name.clone(),
            fields: fields.iter().map(|(field, ty, default)|
                        ( field.clone(), self.validate_type(ty.clone()), default.clone() )
                    ).collect(),
        };

        if let Some(old) = self.structs.insert(name, signature) {
            exit_with_message(format!("Error: Struct '{}' was declared multiple times", old.name));
        }
    }

    pub fn declare_function(&mut self, name: String, parameters: Vec<(String, String)>, ty: String) {
        let signature = FunctionSignature {
            name: name.clone(),
            parameters: parameters.iter().map(|(field, ty)| 
                            ( field.clone(), self.validate_type(ty.clone()) ) 
                        ).collect(),
            return_type: ty,
        };
        
        if let Some(old) = self.functions.insert(name, signature) {
            exit_with_message(format!("Error: Function '{}' was declared multiple times", old.name));
        }
    }

    pub fn validate_function(&self, name: String) -> String {
        if self.functions.contains_key(&name) {
            name
        } else {
            exit_with_message(format!("Error: Unknown or undeclared function '{}'", name));
            // Process exits before this, so no return is needed
            unreachable!();
        }
    }

    /// Returns type_name if it is a valid, previously declared type.
    /// Otherwise, prints error and exits
    pub fn validate_type(&self, type_name: String) -> String {
        if self.primitive_types.contains(type_name.as_str()) || self.structs.contains_key(&type_name) {
            type_name
        } else {   
            exit_with_message(format!("Error: Unknown or undeclared type '{}'", type_name));
            // Process exits before this, so no return is needed
            unreachable!();
        }
    }
}

/*

    This function should transform the AST into valid GLSL code.
    In order to maintain scopes, types, etc., this function utilizes a context.

    That context must track valid identifiers 
        (functions, primitives, structs, builtins, etc.)
    This is done within the parser, meaning a parsed file has an associated context.

    TODO: A lot of this could be moved to the parser

*/
pub fn transpile(ast: &AST, context: &Context) -> String {
    // Unfortunately, GLSL requires functions to be declared in order of use
    // sdf-lang can compensate for this by forward declaring all functions
    // OR sdf-lang can also require forced ordering

    let mut glsl = String::new();

    for item in ast {
        // `Item`s always have global scopes
        match item {
            Item::Struct { name, fields } => {
                glsl.push_str(&template::structure(name, fields));
            }

            Item::Function { name, parameters, return_type, statements } => {
                // TODO: Body statements
                glsl.push_str(&template::function(name, parameters, &return_type));
            }

            Item::Scene { name, statements } => {
                glsl.push_str(&template::scene(name));
            }
        }
    }

    glsl
}