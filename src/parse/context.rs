use crate::parse::ast;
use crate::parse::ast::TypeSpecifier;

use super::glsl::castable;
use super::glsl;

use std::collections::{HashMap, HashSet};

// See https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)

struct StructSignature {
    name: String,
    // Fields and types with optional defaults (field, type, default)
    fields: Vec<(String, TypeSpecifier, Option<ast::Expression>)>,
    has_implementation: bool,
}

struct FunctionSignature {
    name: String,
    // (field_name, field_type)
    parameters: Vec<(String, TypeSpecifier)>,
    return_type: TypeSpecifier,
}

// TODO: Refactor type system to be like this
#[derive(PartialEq, Debug)]
pub enum ScopeType {
    Global,
    Function {
        return_type: TypeSpecifier,
    },
    If,
    Loop,
    Impl,
}

pub struct Scope {
    // scope -> (name -> (type, is_const))
    scopes: HashMap<usize, HashMap<String, (TypeSpecifier, bool)>>,

    // "global", "loop", "if", "function", "scene", etc.
    scope_variants: Vec<ScopeType>,

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
            // Initialize with the global scope active
            scope_variants: vec![ScopeType::Global],
            current: 0,
        }
    }

    pub fn is_within_loop(&self) -> bool {
        self.scope_variants.contains(&ScopeType::Loop)
    }

    pub fn expected_return_type(&self) -> Result<TypeSpecifier, String> {
        for scope in &self.scope_variants {
            if let ScopeType::Function {return_type: t} = scope {
                return Ok(t.clone());
            }
        }

        Err(format!("Unexpected return statement outside of function"))
    }

    pub fn current_kind(&self) -> &ScopeType {
        self.scope_variants.last().unwrap()
    }

    pub fn push_scope(&mut self, kind: ScopeType) {
        self.current += 1;
        self.scope_variants.push(kind);
        self.scopes.insert(self.current, HashMap::new());
    }

    // No check is needed because scopes cannot be popped more than they are pushed
    pub fn pop_scope(&mut self) {
        self.scopes.remove(&self.current);
        self.scope_variants.pop();
        self.current -= 1;
    }

    fn add_var_to_scope(&mut self, name: String, ty: TypeSpecifier, is_constant: bool) -> Result<(), String> {
        if let Some(_old) = self.scopes.get_mut(&self.current).unwrap().insert(name.clone(), (ty.clone(), is_constant)) {
            Err(format!("Variable '{}' already exists in the current scope", name))
        } else {   
            Ok(())
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

    pub fn var_type(&self, name: &str) -> Result<&TypeSpecifier, String> {
        for scope in 0..=self.current {
            if let Some((ty, _is_const)) = self.scopes.get(&scope).unwrap().get(name) {
                return Ok(ty);
            }
        }

        Err(format!("Unknown identifier '{}'", name))
    }

    pub fn is_var_constant(&self, name: &str) -> Result<bool, String> {
        for scope in 0..=self.current {
            if let Some((_ty, is_const)) = self.scopes.get(&scope).unwrap().get(name) {
                return Ok(*is_const);
            }
        }

        Err(format!("Unknown identifier '{}'", name))
    }
}

#[derive(PartialEq, Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
    Library,
}

pub struct Context {
    /// Which shader type the current Context is for (unaffected by imports)
    pub shader_type: ShaderType,

    /// Function name -> Function signature
    functions: HashMap<String, FunctionSignature>,

    /// Struct name -> Struct fields/defaults
    structs: HashMap<String, StructSignature>,

    /// Primitives such as int, uint, bool, etc.
    primitive_types: HashSet<&'static str>,

    /// Collection of user-declared uniforms, their types, and defaults
    // TODO: Default value is not implemented yet
    // TODO: Implement the scope for tagged variables (and allow shadowing?)
    uniforms: HashSet<(String, TypeSpecifier, /* DEFAULT VALUE HERE */)>,
    outs: HashSet<(String, TypeSpecifier, /* DEFAULT VALUE HERE */)>,

    pub scopes: Scope,
}

impl Context {
    pub fn new(shader_type: &ShaderType) -> Self {
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
            "vec2", "vec3", "vec4", "ivec2", "ivec3", "ivec4", "bvec2", "bvec3", "bvec4",
            "dvec2", "dvec3", "dvec4", "uvec2", "uvec3", "uvec4"
            //,"mat2", "mat3", "mat4", "mat2x2", "mat2x3", "mat2x4", "mat3x2", "mat3x3", "mat3x4",
            //"may4x2", "mat4x3", "mat4x4"
        );



        // TODO: Allow different defaults based on shader type
        // TODO: Allow shader-type declaration (like @FRAGMENT or @VERTEX)



        // Default functions are handled externally (glsl::functions)
        // see http://www.shaderific.com/glsl-functions
        let functions = HashMap::new();

        // TODO: HashSet does not save insertion order. This is probably not an issue, but look into it.
        let mut uniforms = HashSet::new();
        let mut outs = HashSet::new();
        // Note that this defaults to the global scope
        let mut scopes = Scope::new();

        // TODO: Finish this once arrays are implemented
        // https://www.khronos.org/opengl/wiki/Built-in_Variable_(GLSL)
        match shader_type {
            ShaderType::Vertex => {
                // Inputs
                scopes.add_var_to_scope( "gl_VertexID".to_owned(), TypeSpecifier::from_ident("int"), true).unwrap();
                scopes.add_var_to_scope( "gl_InstanceID".to_owned(), TypeSpecifier::from_ident("int"), true).unwrap();
                // Note: These are 4.60+
                scopes.add_var_to_scope( "gl_DrawID".to_owned(), TypeSpecifier::from_ident("int"), true).unwrap();
                scopes.add_var_to_scope( "gl_BaseVertex".to_owned(), TypeSpecifier::from_ident("int"), true).unwrap();
                scopes.add_var_to_scope( "gl_BaseInstance".to_owned(), TypeSpecifier::from_ident("int"), true).unwrap();
                
                // Outputs
                scopes.add_var_to_scope( "gl_Position".to_owned(), TypeSpecifier::from_ident("vec4"), false).unwrap();
                scopes.add_var_to_scope( "gl_PointSize".to_owned(), TypeSpecifier::from_ident("float"), false).unwrap();
                // TODO: gl_ClipDistance
            }

            ShaderType::Fragment => {
                // FIXME: This MUST be in out location 0 (must save this particular location)
                outs.insert(("out_color".to_owned(), TypeSpecifier::from_ident("vec4")));
                scopes.add_var_to_scope( "out_color".to_owned(),    TypeSpecifier::from_ident("vec4"), false).unwrap();
                scopes.add_var_to_scope( "gl_FragCoord".to_owned(), TypeSpecifier::from_ident("vec4"), true).unwrap();
                
                // TODO: Add the rest
            }

            ShaderType::Compute => {
                // TODO: This
            }

            ShaderType::Library => {
                // Nothing to do here
            }
        }

        
        // Libraries should not require feature usage
        if *shader_type != ShaderType::Library {   
            // TODO: Make these features optional via opt-in (like "use time ...")

            uniforms.insert(("time".to_owned(), TypeSpecifier::from_ident("float")));
            uniforms.insert(("window_dimensions".to_owned(), TypeSpecifier::from_ident("vec2")));
            uniforms.insert(("mouse_position".to_owned(), TypeSpecifier::from_ident("vec2")));
            
            scopes.add_var_to_scope( "time".to_owned(), TypeSpecifier::from_ident("float"), false              ).unwrap();
            scopes.add_var_to_scope( "window_dimensions".to_owned(), TypeSpecifier::from_ident("vec2"), false  ).unwrap();
        }

        Context {
            shader_type: shader_type.clone(),
            functions,
            structs: HashMap::new(),
            primitive_types,
            uniforms,
            outs,
            scopes,
        }
    }

    pub fn add_var_to_scope(&mut self, name: String, ty: TypeSpecifier, is_constant: bool) -> Result<(), String> {
        if self.is_primitive(&name) {
            return Err(format!("Cannot name variable as primitive type '{}'", name));
        }

        self.scopes.add_var_to_scope(name, ty.clone(), is_constant)?;

        Ok(())
    }

    pub fn is_primitive(&self, type_name: &str) -> bool {
        self.primitive_types.contains(type_name)
    }

    pub fn declare_uniform(&mut self, name: String, ty: TypeSpecifier /*, initial_value: ?? */) -> Result<(), String> {
        if !self.uniforms.insert((name.clone(), ty.clone())) {
            Err(format!("Uniform '{}' was already declared", &name))
        } else {
            Ok(())
        }
    }

    pub fn uniforms(&self) -> &HashSet<(String, TypeSpecifier)> {
        &self.uniforms
    }

    /// Fill a given HashMap with uniform information.
    /// name -> (location, type)
    pub fn get_uniform_map(&self, map: &mut HashMap<String, (usize, TypeSpecifier)>) {
        let mut location = 0;
        for (name, ty) in &self.uniforms {
            map.insert(name.clone(), (location, ty.clone()));
            location += 1;
        }
    }
    
    pub fn declare_out(&mut self, name: String, ty: TypeSpecifier /*, initial_value: ?? */) -> Result<(), String> {
        if !self.outs.insert((name.clone(), ty.clone())) {
            Err(format!("Out '{}' was already declared", &name))
        } else {
            Ok(())
        }
    }
    
    pub fn outs(&self) -> &HashSet<(String, TypeSpecifier)> {
        &self.outs
    }

    pub fn declare_struct(&mut self, name: String, passed_fields: Vec<(String, TypeSpecifier, Option<ast::Expression>)>) -> Result<(), String> {       
        if self.is_primitive(&name) {
            return Err(format!("Cannot name struct '{}' the same as a primitive type", &name));
        }

        let mut fields = Vec::new();
        for (field, ty, default) in passed_fields {
            fields.push( 
                ( field.clone(), self.validate_type(&ty)?, default.clone() ) 
            );
        }
        
        let signature = StructSignature {
            name: name.clone(),
            fields,
            has_implementation: false,
        };

        if let Some(old) = self.structs.insert(name, signature) {
            Err(format!("Struct '{}' was declared multiple times", old.name))
        } else {
            Ok(())
        }
    }

    pub fn declare_implementation(&mut self, struct_name: &str) -> Result<(), String> {
        if let Some(signature) = self.structs.get_mut(struct_name) {
            if !signature.has_implementation {
                signature.has_implementation = true;
            } else {
                return Err(format!("Struct '{}' already has an implementation.", struct_name));
            }
        } else {
            return Err(format!("No such struct exists, '{}'", struct_name));
        }

        Ok(())
    }

    pub fn struct_field_type(&self, struct_name: &str, field_name: &str) -> Result<&TypeSpecifier, String> {
        if let Some(signature) = self.structs.get(struct_name) {
            for (name, ty, _default) in &signature.fields {
                if name == field_name {
                    return Ok(ty);
                }
            }
            Err(format!("Struct '{}' does not have field '{}'", struct_name, field_name))
        } else {
            Err(format!("Type '{}' is not a struct or does not exist (tried accessing field '{}')", struct_name, field_name))
        }
    }

    /// Order constructor arguments and place defaults where needed
    pub fn generate_constructor(&self, ty: &str, fields: Vec<(String, ast::SpannedExpression)>) -> Result<Vec<(String, ast::SpannedExpression)>, String> {
        // Existance is already guarenteed, so can just unwrap()
        let signature = self.structs.get(ty).unwrap();
        
        let mut constructor: Vec<(String, ast::SpannedExpression)> = Vec::new();
        
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
                return Err(format!("The struct '{}' has no field '{}'.", ty, field_name));
            }
        }

        for (field_name, field_type, default) in &signature.fields {
            if let Some(user_supplied) = supplied.get(field_name) {
                // Ensure types are compatible
                if !castable(&self.expression_type(&user_supplied.expression)?, &field_type.as_string())? {
                    return Err(format!("The field '{}' on struct '{}' has type '{}', but got incompatible type '{}'", field_name, ty, field_type, self.expression_type(&user_supplied.expression)?));
                }

                constructor.push((field_name.clone(), user_supplied.clone()));
            } else {
                // Use default
                if let Some(def) = default {
                    // Default span will not be checked again
                    constructor.push((field_name.clone(), ast::SpannedExpression {expression: def.clone(), span: (0, 0)}));
                } else {
                    return Err(format!("The constructor for '{}' has no default for field '{}', but no value was supplied.", ty, field_name.clone()));
                }
            }
        }

        Ok(constructor)
    }

    pub fn declare_function(&mut self, name: String, declared_parameters: Vec<(Option<ast::FuncParamQualifier>, String, TypeSpecifier)>, return_type: TypeSpecifier) -> Result<(), String> {       
        if glsl::functions::is_builtin(&name) {
            return Err(format!("A builtin function, '{}' exists with the same name", &name));
        }
        
        if self.is_primitive(&name) {
            return Err(format!("Cannot name function as primitive type '{}'", name));
        }

        let mut parameters = Vec::new();
        for (_qual, parameter, ty) in declared_parameters {
            parameters.push( (parameter.clone(), self.validate_type(&ty)?) );
        }
        
        let signature = FunctionSignature {
            name: name.clone(),
            parameters,
            return_type,
        };
        
        if let Some(old) = self.functions.insert(name, signature) {
            Err(format!("Function '{}' was declared multiple times", old.name))
        } else {
            Ok(())
        }
    }

    // TODO: Force 2-parameter functions only (for sanity/feasability)
    // TODO: Do not allow vec constructors to pass through here
    pub fn check_function_apply(&self, name: &str, passed_param_types: Vec<TypeSpecifier>) -> Result<(usize, TypeSpecifier), String> {
        if glsl::functions::is_builtin(name) {
            let ty = glsl::functions::can_arrow( name, &passed_param_types )?;
            return Ok((2, passed_param_types[0].clone()));
        }
        
        if let Some(signature) = self.functions.get(name) {   
            if signature.parameters.len() == 0 {
                return Err(format!("The function '{}' does not accept any parameters", name));
            } else if signature.return_type.as_string() == "void" {
                return Err(format!("The function '{}' does not return anything (required for '<-' syntax)", name));
            }

            if passed_param_types.len() < signature.parameters.len() {
                return Err(format!("The function '{}' accepts {} parameters, but only {} were given (minimum of {} needed for '<-' syntax)",
                                    name, signature.parameters.len(), passed_param_types.len(), signature.parameters.len()));
            }

            let expected_type = &signature.parameters[0].1;
            if signature.return_type != *expected_type {
                return Err(format!("The function '{}' returns a different type from what it accepts (must be same types for '<-' syntax)", name));
            }

            for (_name, ty) in &signature.parameters {
                if ty != expected_type {
                    return Err(format!("The function '{}' accepts paremters of different types (must be same types for '<-' syntax)", name));
                }
            }
            for ty in &passed_param_types {
                if ty != expected_type {
                    return Err(format!("The application of function '{}' requires parameters of type '{}', but got type '{}'", name, expected_type, ty));
                }
            }

            Ok( (signature.parameters.len(), signature.return_type.clone()) )
        } else {
            Err(format!("The function '{}' does not exist", name))
        }
    }

    /// Validates a function call, returning the function's type.
    /// Constructs vector types similarly.
    pub fn check_function_call(&self, name: &str, passed_param_types: Vec<TypeSpecifier>) -> Result<TypeSpecifier, String> {
        // None of these special cases can ever be arrays
        if glsl::vec::is_vec_constructor_or_type(name) {
            return glsl::vec::validate_constructor(name, &passed_param_types);
        }
        else if glsl::functions::is_builtin(name) {
            return Ok(TypeSpecifier::Identifier(glsl::functions::validate_function(name, &passed_param_types)?));
        }
        
        if let Some(function) = self.functions.get(name) {
            if function.parameters.len() != passed_param_types.len() {
                return Err(format!("The function '{}' takes {} parameters, but {} were supplied", name, function.parameters.len(), passed_param_types.len()));
            }

            for ((param_name, param_type), passed_type) in function.parameters.iter().zip(passed_param_types.iter()) {
                if !castable(&passed_type.as_string(), &param_type.as_string())? {
                    return Err(format!("The parameter '{}' in function '{}' takes a '{}', but a '{}' was given (cannot cast)",
                                                        param_name, name, param_type, passed_type));
                }
            }

            Ok(function.return_type.clone())
        } else {
            Err(format!("The function '{}' was not found", name))
        }
    }

    pub fn add_type(&self, left_type: &str, right_type: &str) -> Result<String, String> {
        // TODO: when the right_type is a vec type
        let resulting = match left_type {
            "double" => {
                match right_type {
                    "double" | "float" | "int" | "uint" => "double",
                    _ => return Err(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }
            
            "float" => {
                match right_type {
                    "double" => "double",
                    "float" | "int" | "uint" => "float",
                    x @ "vec2" | x @ "vec3" | x @ "vec4" => x,
                    _ => return Err(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }
            
            "int" => {
                match right_type {
                    "double" => "double",
                    "float" => "float",
                    "int" => "int",
                    "uint" => "int",
                    _ => return Err(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }

            "uint" => {
                match right_type {
                    "double" => "double",
                    "float" => "float",
                    "int" => "int",
                    "uint" => "uint",
                    _ => return Err(format!("Types '{}' and '{}' are incompatible or not implemented", left_type, right_type)),
                }
            }

            v @ "dvec2" | v @ "dvec3" | v @ "dvec4" => {
                match right_type {
                    "double" | "float" | "int" | "uint" => v,
                    r => {
                        if v == r {
                            v
                        } else {
                            return Err(format!("Cannot add type '{}' to type '{}'", r, v));
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
                            return Err(format!("Cannot add type '{}' to type '{}'", r, v));
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
                            return Err(format!("Cannot add type '{}' to type '{}'", r, v));
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
                            return Err(format!("Cannot add type '{}' to type '{}'", r, v));
                        }
                    }
                    // _ => panic!(format!("Vector type '{}' must be left of added type '{}'", left_type, right_type))
                }
            }

            _ => return Err(format!("Cannot add/subtract type '{}' with type '{}'", left_type, right_type)),
        };

        Ok(resulting.to_owned())
    }

    pub fn multiply_type(&self, left_type: &str, right_type: &str) -> Result<String, String> {
        // TODO: Is it exactly the same?
        self.add_type(left_type, right_type)
    }

    pub fn negate_type(&self, type_name: &str) -> Result<String, String> {
        match type_name {
            "uint" => Ok("int".to_owned()),

            "uvec2" => Ok("ivec2".to_owned()),
            "uvec3" => Ok("ivec3".to_owned()),
            "uvec4" => Ok("ivec4".to_owned()),

            "bool" | "bvec2" | "bvec3" | "bvec4" => return Err("Cannot negate boolean types".to_owned()),

            // double, float, int, vec2, vec3, vec4, etc.
            x => {
                if self.structs.contains_key(x) {
                    return Err("Only numeric types can be negated".to_owned());
                }

                // TODO: Need to check more types before this (bvecs, enums, etc.)
                Ok(x.to_owned())
            }
        }
    }

    /// Returns the TypeSpecifier if it is of a valid type/array.
    pub fn validate_type(&self, ty: &TypeSpecifier) -> Result<TypeSpecifier, String> {
        if self.primitive_types.contains(ty.type_name()) || self.structs.contains_key(ty.type_name()) {
            Ok(ty.clone())
        } else {
            Err(format!("Unknown or undeclared type '{}'", ty.type_name()))
        }
    }

    pub fn validate_type_name(&self, name: &str) -> Result<(), String> {
        if self.primitive_types.contains(name) || self.structs.contains_key(name) {
            Ok(())
        } else {
            Err(format!("Unknown or undeclared type '{}'", name))
        }
    }

    pub fn expression_type(&self, expression: &ast::Expression) -> Result<String, String>{
        Ok(match expression {
            ast::Expression::ArrayConstructor { expressions, ty } => {
                format!("{}[{}]", ty, expressions.len())
            }

            ast::Expression::Parenthesized(expr) => {
                self.expression_type(expr.as_ref())?
            }

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
                    
                    x => return Err(format!("Type of {:?} is not implemented yet", x)),
                
                }.to_owned()
            }

            ast::Expression::Identifier(name) => {
                self.scopes.var_type(name)?.as_string()
            }

            ast::Expression::Binary {ty, ..} => {
                ty.clone()
            }

            ast::Expression::Unary {operator, expr: _, ty} => {
                match operator {
                    ast::UnaryOperator::Index(_) => {
                        // TODO: Is this correct?
                        // TODO: Note that multi-dimensional arrays are version 4.30+ only
                        
                        // Strip the array type off, leaving only the base type
                        // ty.split("[").take(1).collect::<Vec<&str>>()[0].to_owned()
                        ty.split("[").next().unwrap().to_owned()
                    }
                    
                    _ => {
                        ty.clone()
                    }
                }
            }

            ast::Expression::FunctionApply(apply) => {
                apply.ty.clone()
            }

            ast::Expression::FunctionCall(call) => {
                call.ty.clone()
            }

            ast::Expression::If {ty, ..} => {
                ty.clone()
            }

            ast::Expression::Member(member) => {
                member.ty.clone()
            }
        })
    }
}