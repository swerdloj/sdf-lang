use crate::exit;
use crate::parse::ast;
use super::glsl::castable;
use super::glsl;

use std::collections::{HashMap, HashSet};

// See https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)

struct StructSignature {
    name: String,
    // Fields and types with optional defaults (field, type, default)
    fields: Vec<(String, String, Option<ast::Expression>)>,
    has_implementation: bool,
}

struct FunctionSignature {
    name: String,
    // (field_name, field_type)
    parameters: Vec<(String, String)>,
    return_type: String,
}

pub struct Scope {
    // scope -> (name -> type)
    scopes: HashMap<usize, HashMap<String, String>>,

    // "global", "loop", "if", "function", "scene", etc.
    scope_variants: Vec<String>,

    // 0 is global scope
    current: usize,
}

// TODO: Wrap these in checks and call them from context.name rather than context.scopes.name
impl Scope {
    fn new() -> Self {
        let mut scopes = HashMap::new();
        scopes.insert(0, HashMap::new());

        Scope {
            scopes,
            scope_variants: vec!["global".to_owned()],
            current: 0,
        }
    }

    pub fn is_within_loop(&self) -> bool {
        self.scope_variants.contains(&String::from("loop"))
    }

    pub fn current_kind(&self) -> &String {
        self.scope_variants.last().unwrap()
    }

    pub fn push_scope(&mut self, kind: &str) {
        self.current += 1;
        self.scope_variants.push(kind.to_owned());
        self.scopes.insert(self.current, HashMap::new());
    }

    // No check is needed because scopes cannot be popped more than they are pushed
    pub fn pop_scope(&mut self) {
        self.scopes.remove(&self.current);
        self.scope_variants.pop();
        self.current -= 1;
    }

    fn add_var_to_scope(&mut self, name: String, ty: String) {
        // println!("Adding '{}' to nested scope {}", &name, self.current);
        if let Some(_old) = self.scopes.get_mut(&self.current).unwrap().insert(name.clone(), ty) {
            exit!(format!("Error: Variable '{}' already exists in the current scope", name));
        }
    }

    pub fn is_var_in_scope(&self, name: &str) -> bool {
        for scope in 0..=self.current {
            if self.scopes.get(&scope).unwrap().get(name).is_some() {
                return true;
            }
        }

        false
    }

    pub fn var_type(&self, name: &str) -> String {
        for scope in 0..=self.current {
            if let Some(ty) = self.scopes.get(&scope).unwrap().get(name) {
                return ty.to_owned();
            }
        }

        exit!(format!("Unknown identifier '{}'", name));
    }
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
    // TODO: Implement the scope for tagged variables (and allow shadowing?)
    uniforms: HashSet<(String, String, /* DEFAULT VALUE HERE */)>,
    outs: HashSet<(String, String, /* DEFAULT VALUE HERE */)>,

    pub scopes: Scope,
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
        
        // TODO: Insert default functions (need special mechanism for overloading?)
        // functions.insert("length", FunctionSignature{name: "length", ..}, "float");

        // TODO: HashSet does not save insertion order. This is probably not an issue, but look into it.
        let mut uniforms = HashSet::new();
        uniforms.insert(("time".to_owned(), "int".to_owned()));
        uniforms.insert(("window_size".to_owned(), "vec2".to_owned()));
        uniforms.insert(("mouse_position".to_owned(), "vec2".to_owned()));

        // FIXME: This MUST be in out location 0 (must save this order)
        let mut outs = HashSet::new();
        outs.insert(("out_color".to_owned(), "float".to_owned()));

        Context {
            functions,
            structs: HashMap::new(),
            primitive_types,
            uniforms,
            outs,
            scopes: Scope::new(),
        }
    }

    pub fn add_var_to_scope(&mut self, name: String, ty: String) {
        if self.is_primitive(&name) {
            exit!(format!("Error: Cannot name variable as primitive type '{}'", name));
        }

        self.scopes.add_var_to_scope(name, ty);
    }

    pub fn is_primitive(&self, type_name: &str) -> bool {
        self.primitive_types.contains(type_name)
    }

    pub fn declare_uniform(&mut self, name: String, ty: String /*, initial_value: ?? */) {
        if !self.uniforms.insert((name.clone(), ty)) {
            exit!(format!("Error: Uniform '{}' was already declared", &name));
        }
    }

    pub fn uniforms(&self) -> &HashSet<(String, String)> {
        &self.uniforms
    }
    
    pub fn declare_out(&mut self, name: String, ty: String /*, initial_value: ?? */) {
        if !self.outs.insert((name.clone(), ty)) {
            exit!(format!("Error: Out '{}' was already declared", &name));
        }
    }
    
    pub fn outs(&self) -> &HashSet<(String, String)> {
        &self.outs
    }

    pub fn declare_struct(&mut self, name: String, fields: Vec<(String, String, Option<ast::Expression>)>) {       
        if self.is_primitive(&name) {
            exit!(format!("Error: Cannot name struct '{}' the same as a primitive type", &name));
        }
        
        let signature = StructSignature {
            name: name.clone(),
            fields: fields.iter().map(|(field, ty, default)|
                        ( field.clone(), self.validate_type(ty), default.clone() )
                    ).collect(),
            has_implementation: false,
        };

        if let Some(old) = self.structs.insert(name, signature) {
            exit!(format!("Error: Struct '{}' was declared multiple times", old.name));
        }
    }

    pub fn declare_implementation(&mut self, struct_name: &str) {
        if let Some(signature) = self.structs.get_mut(struct_name) {
            if !signature.has_implementation {
                signature.has_implementation = true;
            } else {
                exit!(format!("Error: Struct '{}' already has an implementation.", struct_name));
            }
        } else {
            exit!(format!("Error: No such struct exists, '{}'", struct_name));
        }
    }

    pub fn struct_field_type(&self, struct_name: &str, field_name: &str) -> String {
        if let Some(signature) = self.structs.get(struct_name) {
            for (name, ty, _default) in &signature.fields {
                if name == field_name {
                    return ty.to_owned();
                }
            }
            exit!(format!("Error: Struct '{}' does not have field '{}'", struct_name, field_name));
        } else {
            exit!(format!("Error: Type '{}' is not a struct or does not exist (tried accessing field '{}')", struct_name, field_name));
        }
    }

    /// Order constructor arguments and place defaults where needed
    pub fn generate_constructor(&self, ty: &str, fields: Vec<(String, ast::Expression)>) -> Vec<(String, ast::Expression)> {
        // Existance is already guarenteed, so can just unwrap()
        let signature = self.structs.get(ty).unwrap();
        
        let mut constructor: Vec<(String, ast::Expression)> = Vec::new();
        
        let mut supplied = HashMap::new();
        for (field_name, expr) in fields {
            supplied.insert(field_name, expr);
        }

        let mut all_fields = HashSet::new();
        // Ensure no extra fields were given by the user
        for (field_name, _field_type, _default) in &signature.fields {
            all_fields.insert(field_name.clone());
        }

        for field_name in supplied.keys() {
            if !all_fields.contains(field_name) {
                exit!(format!("Error: The struct '{}' has no field '{}'.", ty, field_name));
            }
        }

        for (field_name, field_type, default) in &signature.fields {
            if let Some(user_supplied) = supplied.get(field_name) {
                // Ensure types are compatible
                if !castable(&self.expression_type(user_supplied), field_type) {
                    exit!(format!("Error: The field '{}' on struct '{}' has type '{}', but got incompatible type '{}'", field_name, ty, field_type, self.expression_type(user_supplied)));
                }

                constructor.push((field_name.clone(), user_supplied.clone()));
            } else {
                // Use default
                if let Some(def) = default {
                    constructor.push((field_name.clone(), def.clone()));
                } else {
                    exit!(format!("Error: The constructor for '{}' has no default for field '{}', but no value was supplied.", ty, field_name.clone()));
                }
            }
        }

        constructor
    }

    // pub fn function_type(&self, name: &str) -> String {
    //     if let Some(function) = self.functions.get(name) {
    //         function.return_type.clone()
    //     } else {
    //         exit!(format!("Error: No such function exists: '{}'", name));
    //         unreachable!();
    //     }
    // }

    pub fn declare_function(&mut self, name: String, parameters: Vec<(String, String)>, ty: String) {
        if glsl::functions::is_builtin(&name) {
            exit!(format!("Error: A builtin function, '{}' exists with the same name", &name));
        }
        
        if self.is_primitive(&name) {
            exit!(format!("Error: Cannot name function as primitive type '{}'", name));
        }
        
        let signature = FunctionSignature {
            name: name.clone(),
            parameters: parameters.iter().map(|(field, ty)| 
                            ( field.clone(), self.validate_type(ty) ) 
                        ).collect(),
            return_type: ty,
        };
        
        if let Some(old) = self.functions.insert(name, signature) {
            exit!(format!("Error: Function '{}' was declared multiple times", old.name));
        }
    }

    /// Validates a function call, returning the function's type.
    /// Constructs vector types similarly.
    pub fn check_function_call(&self, name: &str, passed_param_types: Vec<String>) -> String {
        if glsl::vec::is_vec_constructor_or_type(name) {
            return glsl::vec::validate_constructor(name, &passed_param_types);
        }
        else if glsl::functions::is_builtin(name) {
            return glsl::functions::validate_function(name, &passed_param_types);
        }
        
        if let Some(function) = self.functions.get(name) {
            if function.parameters.len() != passed_param_types.len() {
                exit!(format!("The function '{}' takes {} parameters, but {} were supplied", name, function.parameters.len(), passed_param_types.len()));
            }

            for ((param_name, param_type), passed_type) in function.parameters.iter().zip(passed_param_types.iter()) {
                if !castable(passed_type, param_type) {
                    exit!(format!("The parameter '{}' in function '{}' takes a '{}', but a '{}' was given (cannot cast)",
                                                        param_name, name, param_type, passed_type));
                }
            }

            function.return_type.clone()
        } else {
            exit!(format!("The function '{}' was not found", name));
        }
    }

    pub fn add_type(&self, left_type: &str, right_type: &str) -> String {
        let resulting = match left_type {
            "double" => {
                match right_type {
                    "double" | "float" | "int" | "uint" => "double",
                    _ => exit!(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }
            
            "float" => {
                match right_type {
                    "double" => "double",
                    "float" | "int" | "uint" => "float",
                    _ => exit!(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }
            
            "int" => {
                match right_type {
                    "double" => "double",
                    "float" => "float",
                    "int" => "int",
                    "uint" => "int",
                    _ => exit!(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }

            "uint" => {
                match right_type {
                    "double" => "double",
                    "float" => "float",
                    "int" => "int",
                    "uint" => "uint",
                    _ => exit!(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }

            v @ "dvec2" | v @ "dvec3" | v @ "dvec4" => {
                match right_type {
                    "double" | "float" | "int" | "uint" => v,
                    r => {
                        if v == r {
                            v
                        } else {
                            exit!(format!("Error: Cannot add type '{}' to type '{}'", r, v));
                        }
                    }
                    // _ => panic!(format!("Vector type '{}' must be left of added type '{}'", left_type, right_type))
                }
            }

            v @ "vec2" | v @ "vec3" | v @ "vec4" => {
                match right_type {
                    "float" | "int" | "uint" => v,
                    r => {
                        if v == r {
                            v
                        } else {
                            exit!(format!("Error: Cannot add type '{}' to type '{}'", r, v));
                        }
                    }
                    // _ => panic!(format!("Vector type '{}' must be left of added type '{}'", left_type, right_type))
                }
            }

            v @ "ivec2" | v @ "ivec3" | v @ "ivec4" => {
                match right_type {
                    "int" | "uint" => v,
                    r => {
                        if v == r {
                            v
                        } else {
                            exit!(format!("Error: Cannot add type '{}' to type '{}'", r, v));
                        }
                    }
                    // _ => panic!(format!("Vector type '{}' must be left of added type '{}'", left_type, right_type))
                }
            }

            v @ "uvec2" | v @ "uvec3" | v @ "uvec4" => {
                match right_type {
                    "uint" => v,
                    r => {
                        if v == r {
                            v
                        } else {
                            exit!(format!("Error: Cannot add type '{}' to type '{}'", r, v));
                        }
                    }
                    // _ => panic!(format!("Vector type '{}' must be left of added type '{}'", left_type, right_type))
                }
            }

            _ => exit!(format!("Cannot add/subtract type '{}' with type '{}'", left_type, right_type)),
        };

        resulting.to_owned()
    }

    pub fn multiply_type(&self, left_type: &str, right_type: &str) -> String {
        // TODO: Is it exactly the same?
        self.add_type(left_type, right_type)
    }

    pub fn negate_type(&self, type_name: &str) -> String {
        match type_name {
            "uint" => "int".to_owned(),

            "uvec2" => "ivec2".to_owned(),
            "uvec3" => "ivec3".to_owned(),
            "uvec4" => "ivec4".to_owned(),

            "bool" | "bvec2" | "bvec3" | "bvec4" => exit!("Cannot negate boolean types".to_owned()),

            // double, float, int, vec2, vec3, vec4, etc.
            x => {
                if self.structs.contains_key(x) {
                    exit!("Only numeric types can be negated".to_owned());
                }

                // TODO: Need to check more types before this (bvecs, enums, etc.)
                x.to_owned()
            }
        }
    }

    /// Returns type_name if it is a valid, previously declared type.
    /// Otherwise, prints error and exits
    pub fn validate_type(&self, type_name: &str) -> String {
        if self.primitive_types.contains(type_name) || self.structs.contains_key(type_name) {
            type_name.to_owned()
        } else {   
            exit!(format!("Error: Unknown or undeclared type '{}'", type_name));
        }
    }

    pub fn expression_type(&self, expression: &ast::Expression) -> String {
        match expression {
            ast::Expression::Literal(literal) => {
                match literal {
                    ast::Literal::Bool(_) => {
                        "bool"
                    }

                    ast::Literal::Float(_) => {
                        "float"
                    }

                    ast::Literal::Int(_) => {
                        "int"
                    }
                    
                    ast::Literal::UInt(_) => {
                        "uint"
                    }
                    
                    x => exit!(format!("Type of {:?} is not implemented yet", x)),
                
                }.to_owned()
            }

            ast::Expression::Identifier(name) => {
                self.scopes.var_type(name)
            }

            ast::Expression::Binary {ty, ..} => {
                ty.clone()
            }

            ast::Expression::Unary {ty, ..} => {
                ty.clone()
            }

            ast::Expression::FunctionCall(call) => {
                call.ty.clone()
            }

            ast::Expression::If {ty, .. } => {
                ty.clone()
            }

            ast::Expression::Member(member) => {
                member.ty.clone()
            }
        }
    }
}