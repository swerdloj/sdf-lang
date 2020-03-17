
use crate::exit_with_message;
use crate::parse::ast::*;
use std::collections::{HashMap, HashSet};

/* 
    TODO: Get rid of all the `.clone()` calls
*/

// See https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)

// TODO: This may not be needed. May instead simply keep everything as a String,
//        and declare these as valid identifiers *via* Strings
// enum Type {
//     Void,

//     Int(i32),
//     UInt(u32),
//     Bool(bool),
//     Float(f32),
//     Double(f64),

//     // `vec` types can be of size 2, 3, or 4
//     BVec2([bool; 2]),
//     BVec3([bool; 3]),
//     BVec4([bool; 4]),

//     IVec2([i32; 2]),
//     IVec3([i32; 3]),
//     IVec4([i32; 4]),
    
//     UVec2([u32; 2]),
//     UVec3([u32; 3]),
//     UVec4([u32; 4]),

//     Vec2([f32; 2]),
//     Vec3([f32; 3]),
//     Vec4([f32; 4]),

//     DVec2([f64; 2]),
//     DVec3([f64; 3]),
//     DVec4([f64; 4]),

//     // TODO: Matrices -- Indexed like arrays
//     // matNxM where N and M can be 2, 3, or 4
//     // matN where matN == matNxN

//     // TODO: The rest

//     // TODO: This
//     Struct(String),
// }

// impl Type {
//     fn from_string(identifier: String) -> Self {
//         use Type::*;

//         match identifier.as_str() {
//             "int" => Int(0),
//             "uint" => UInt(0),
//             "bool" => Bool(false),
//             "float" => Float(0.0f32),
//             "double" => Double(0.0f64),

//             "vec2" => Vec2([0.0, 0.0]),
//             "vec3" => Vec3([0.0, 0.0, 0.0]),
//             "vec4" => Vec4([0.0, 0.0, 0.0, 0.0]),

//             _ => {
//                 unimplemented!()
//             }
//         }
//     }
// }

struct StructSignature {
    // Struct identifier
    name: String,
    // Fields and types with optional defaults
    // TODO: How to represent the default value?
    fields: Vec<(String, String, Option<String>)>,
}

struct FunctionSignature {
    // Function identifier
    name: String,
    parameters: Vec<(String, String)>,
    return_type: String,
}

struct Context {
    functions: HashMap<String, FunctionSignature>,
    structs: HashMap<String, StructSignature>,
    primitive_types: HashSet<&'static str>,
}

impl Context {
    fn new() -> Self {
        let mut primitive_types = HashSet::new();
        primitive_types.insert("int");
        primitive_types.insert("uint");
        primitive_types.insert("void");
        // functions.insert("length", ...);
        // TODO: Macro-ize this and add the rest

        Context {
            functions: HashMap::new(),
            structs: HashMap::new(),
            primitive_types,
        }
    }

    fn declare_function(&mut self, name: String, parameters: Vec<(String, String)>, ty: Option<String>) {
        let signature = FunctionSignature {
            name: name.clone(),
            parameters: parameters.iter().map(|(field, ty)| 
                            ( field.clone(), self.validate_type(ty.clone()) ) 
                        ).collect(),
            return_type: if let Some(return_type) = ty {
                            return_type
                         } else { "void".to_owned() },
        };
        
        if let Some(old) = self.functions.insert(name, signature) {
            exit_with_message(format!("Error: Function {} was declared multiple times", old.name));
        }
    }

    fn validate_function(&self, name: String) -> String {
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
    fn validate_type(&self, type_name: String) -> String {
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
    TODO: 

    This function should transform the AST into valid GLSL code.
    In order to maintain scopes, types, etc., this function utilizes a context.

    That context must track valid identifiers 
        (functions, primitives, structs, builtins, etc.)
*/
pub fn traverse_ast(ast: &AST) {
    let mut context = Context::new();

    // Unfortunately, GLSL requires functions to be declared in order of use
    // sdf-lang must replicate this
    for item in ast {
        // `Item`s always have global scopes
        match item {
            Item::Struct { name, fields } => {

            }

            Item::Function { name, parameters, return_type, statements } => {
                context.declare_function(name.clone(), parameters.clone(), return_type.clone());
            }

            Item::Scene { name, statements } => {

            }
        }
    }
}