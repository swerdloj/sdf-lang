use crate::exit_with_message;

use std::collections::{HashMap, HashSet};

use crate::parse::ast;

// See https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)

struct StructSignature {
    name: String,
    // Fields and types with optional defaults (field, type, default)
    fields: Vec<(String, String, Option<ast::Expression>)>,
}

struct FunctionSignature {
    name: String,
    // (field_name, field_type)
    parameters: Vec<(String, String)>,
    return_type: String,
}

pub struct Context {
    /// Function name -> Function signature
    functions: HashMap<String, FunctionSignature>,
    /// Struct name -> Struct fields/defaults
    structs: HashMap<String, StructSignature>,
    /// Primitives such as int, uint, bool, etc.
    primitive_types: HashSet<&'static str>,
    /// Collection of user-declared uniforms, their types, and defaults
    // TODO: Default value is not implemented yet
    uniforms: HashSet<(String, String /*, DEFAULT VALUE HERE */)>,

    // FIXME: Everything is in global scope for now
    /// Map of identifier -> type
    scopes: HashMap<String, String>,
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
            "float", "double", "bool", "int", "uint", "sampler2D", "samplerCube",
            "vec2", "vec3", "vec4"
        );

        // see http://www.shaderific.com/glsl-functions
        let functions = HashMap::new();
        
        // functions.insert("length", FunctionSignature{name: "length", ..}, "float");

        let mut uniforms = HashSet::new();
        uniforms.insert(("time".to_owned(), "int".to_owned()));
        // uniforms.push(("window_size".to_owned(), "vec2".to_owned(), ??));
        // uniforms.push(("mouse_position".to_owned(), "vec2".to_owned(), ??));

        Context {
            functions,
            structs: HashMap::new(),
            primitive_types,
            uniforms,
            
            scopes: HashMap::new(),
        }
    }

    pub fn add_var_to_scope(&mut self, name: String, ty: String) {
        if let Some(_old) = self.scopes.insert(name.clone(), self.validate_type(ty)) {
            exit_with_message(format!("Error: Variables '{}' already exists in the current scope", name));
        }
    }

    pub fn is_var_in_scope(&self, name: &str) -> bool {
        if self.scopes.get(name).is_some() {
            true
        } else {
            false
        }
    }

    pub fn var_type(&self, name: String) -> String {
        if let Some(ty) = self.scopes.get(&name) {
            ty.to_owned()
        } else {
            exit_with_message(format!("Unknown identifier '{}'", name));
            unreachable!();
        }
    }

    pub fn declare_uniform(&mut self, name: String, ty: String /*, initial_value: ?? */) {
        if !self.uniforms.insert((name.clone(), ty)) {
            exit_with_message(format!("Error: Uniform '{}' was already declared", &name));
        }
    }

    pub fn uniforms(&self) -> &HashSet<(String, String)> {
        &self.uniforms
    }

    pub fn declare_struct(&mut self, name: String, fields: Vec<(String, String, Option<ast::Expression>)>) {       
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

    pub fn generate_constructor(&self, ty: &str, fields: Vec<(String, ast::Expression)>) -> Vec<(String, ast::Expression)> {
        // Order the arguments and place defaults where missing

        // Existance is already guarenteed, so can just unwrap()
        let signature = self.structs.get(ty).unwrap();
        
        let mut constructor: Vec<(String, ast::Expression)> = Vec::new();
        
        let mut supplied = HashMap::new();
        for (field_name, expr) in fields {
            supplied.insert(field_name, expr);
        }

        let mut all_fields = HashSet::new();
        // Ensure no extra fields were given by the user
        for (field_name, _, _) in &signature.fields {
            all_fields.insert(field_name.clone());
        }

        for field_name in supplied.keys() {
            if !all_fields.contains(field_name) {
                exit_with_message(format!("Error: The struct '{}' has no field '{}'.", ty, field_name));
            }
        }

        // TODO: Ensure types are compatible
        for (field_name, field_type, default) in &signature.fields {
            if let Some(user_supplied) = supplied.get(field_name) {
                constructor.push((field_name.clone(), user_supplied.clone()));
            } else {
                // Use default
                if let Some(def) = default {
                    constructor.push((field_name.clone(), def.clone()));
                } else {
                    exit_with_message(format!("Error: The constructor for '{}' has no default for field '{}', but no value was supplied.", ty, field_name.clone()));
                }
            }
        }

        assert_eq!(constructor.len(), signature.fields.len());

        constructor
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

    pub fn check_function_call(&self, name: &str, passed_param_types: Vec<String>) -> String {
        if let Some(function) = self.functions.get(name) {
            if function.parameters.len() != passed_param_types.len() {
                exit_with_message(format!("The function '{}' takes {} parameters, but {} were supplied", name, function.parameters.len(), passed_param_types.len()));
            }

            for ((param_name, param_type), passed_type) in function.parameters.iter().zip(passed_param_types.iter()) {
                if !Self::castable(passed_type, param_type) {
                    exit_with_message(format!("The parameter '{}' in function '{}' takes a '{}', but a '{}' was given",
                                                        param_name, name, param_type, passed_type));
                }
            }

            function.return_type.clone()
        } else {
            exit_with_message(format!("The function '{}' was not found", name));
            unreachable!();
        }
    }

    // TODO: Require all narrowing conversions to have explicit casts
    // TODO: Implement explcit casting
    pub fn castable(from: &str, to: &str) -> bool {
        if from == to {
            return true;
        }

        println!("Casting {} to {}", from, to);

        match to {
            "double" => {
                match from {
                    "float" | "int" | "uint" => true,
                    "bool" => false,
                    x => {
                        exit_with_message(format!("Cannot cast type '{}' to 'double'", x));
                        unreachable!();
                    },
                }
            }

            "float" => {
                match from {
                    "int" | "uint" => true,
                    "bool" => false,
                    x => {
                        exit_with_message(format!("Cannot cast type '{}' to 'float'", x));
                        unreachable!();
                    },
                }
            }

            "int" => {
                match from {
                    "uint" => true,
                    "bool" | "float" => false,
                    x => {
                        exit_with_message(format!("Cannot cast type '{}' to 'int'", x));
                        unreachable!();
                    },
                }
            }

            "uint" => {
                match from {
                    "int" => true,
                    "bool" => false,
                    x => {
                        exit_with_message(format!("Cannot cast type '{}' to 'uint'", x));
                        unreachable!();
                    },
                }
            }

            "bool" => {
                match from {
                    "double" | "fload" | "int" | "uint" => false,
                    x => {
                        exit_with_message(format!("Cannot cast type '{}' to 'bool'", x));
                        unreachable!();
                    }
                }
            }

            x => {
                exit_with_message(format!("Type '{}' has no cast implementations. Cannot cast from '{}' to '{}'.", x, from, to));
                unreachable!();
            }
        }
    }

    pub fn add_type(&self, left_type: &str, right_type: &str) -> String {
        let resulting = match left_type {
            "double" => {
                match right_type {
                    "double" | "float" | "int" | "uint" => "double",
                    _ => {
                        panic!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)
                    }
                }
            }
            
            "float" => {
                match right_type {
                    "double" => "double",
                    "float" | "int" | "uint" => "float",
                    _ => {
                        panic!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)
                    }
                }
            }
            
            "int" => {
                match right_type {
                    "double" => "double",
                    "float" => "float",
                    "int" => "int",
                    "uint" => "int",
                    _ => {
                        panic!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)
                    }
                }
            }

            "uint" => {
                match right_type {
                    "double" => "double",
                    "float" => "float",
                    "int" => "int",
                    "uint" => "uint",
                    _ => {
                        panic!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)
                    }
                }
            }

            // TODO: Remaining types (vecs, etc.)

            _ => {
                exit_with_message(format!("Cannot add/subtract type '{}' with type '{}'", left_type, right_type));
                unreachable!();
            }
        };

        resulting.to_owned()
    }

    pub fn multiply_type(&self, left_type: &str, right_type: &str) -> String {
        // FIXME: Is it exactly the same?
        self.add_type(left_type, right_type)
    }

    pub fn negate_type(&self, type_name: &str) -> String {
        match type_name {
            "uint" => "int".to_owned(),
            "bool" => {
                exit_with_message("Cannot negate a boolean".to_owned());
                unreachable!();
            }

            x => {
                if self.structs.contains_key(x) {
                    exit_with_message("Only numeric types can be negated".to_owned());
                }

                // TODO: Need to check more types before this (bvecs, enums, etc.)
                x.to_owned()
            }
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

    pub fn expression_type(&self, expression: &ast::Expression) -> String {
        match expression {
            ast::Expression::Literal(literal) => {
                match literal {
                    ast::Literal::Bool(_) => {
                        "bool".to_owned()
                    }

                    ast::Literal::Float(_) => {
                        "float".to_owned()
                    }

                    ast::Literal::Int(_) => {
                        "int".to_owned()
                    }
                    
                    ast::Literal::UInt(_) => {
                        "uint".to_owned()
                    }

                    x => {
                        panic!("Type of {:?} is not implemented yet", x)
                    },

                    // TODO: The rest
                }
            }

            ast::Expression::Identifier(name) => {
                self.var_type(name.to_owned())
            }

            ast::Expression::Binary {ty, ..} => {
                ty.clone()
            }

            ast::Expression::Unary {ty, ..} => {
                ty.clone()
            }

            ast::Expression::FunctionCall {ty, ..} => {
                ty.clone()
            }
        }
    }
}