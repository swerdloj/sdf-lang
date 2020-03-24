use crate::parse::context::Context;
use crate::exit_with_message;

// TODO: Add the remaining vec types

/// Returns whether a function is actually a vec constructor
pub fn is_vec_constructor_or_type(fn_name: &str) -> bool {
    match fn_name {
        "bvec2" | "bvec3" | "bvec4" |
        "ivec2" | "ivec3" | "ivec4" |
        "uvec2" | "uvec3" | "uvec4" |
        "vec2"  | "vec3"  | "vec4"  |
        "dvec2" | "dvec3" | "dvec4" 
          => true,

        _ => false,
    }
}

/// Checks whether a given swizzle is valid for the vec type. Then checks whether the swizzle is assignable.
/// If so, returns the swizzle type
pub fn validate_swizzle_for_assignment(vec_type: &str, swizzle: &str) -> String {
    let swizzle_type = validate_swizzle(vec_type, swizzle);
    
    if swizzle.len() == 1 {
        return swizzle_type;
    }

    let mut seen = std::collections::HashSet::new();
    for field in swizzle.chars() {
        if !seen.insert(field) {
            exit_with_message(format!("Error: Assignment swizzles cannot repeat fields ('{}')", swizzle));
        }
    }

    swizzle_type
}

/// Checks whether a given swizzle is valid for the vec type. Returns the swizzle type if so
pub fn validate_swizzle(vec_type: &str, swizzle: &str) -> String {
    if swizzle.len() > 4 {
        exit_with_message(format!("Error: Swizzle can only be up to four items in size"));
    }

    let primitive = vec_primitive_type(vec_type);

    // These unwraps should be guarenteed as safe at this point
    let vec_size = vec_type.to_owned().pop().unwrap().to_digit(10).unwrap();

    // TODO: Allow more than just "xyzw" ?

    // Note that all vec types have at least 2 fields
    for field in swizzle.chars() {
        match field {
            'x' | 'y' => {

            }

            'z' => {
                if vec_size < 3 {
                    exit_with_message(format!("Error: '{}' has no third component, y", vec_type));
                }
            }

            'w' => {
                if vec_size < 4 {
                    exit_with_message(format!("Error: '{}' has no fourth component, w", vec_type));
                }
            }

            _ => unreachable!(),
        }
    }

    // Single element of the vec
    if swizzle.len() == 1 {
        primitive.to_owned()
    } else {
        let mut result = vec_type.to_owned();

        result.pop();
        result.push_str(&swizzle.len().to_string());

        result
    }
}

/// Returns the base type of a vector
fn vec_primitive_type(vec_type: &str) -> &'static str {
    match vec_type {
        "bvec2" | "bvec3" | "bvec4" => "bool",
        "ivec2" | "ivec3" | "ivec4" => "int",
        "uvec2" | "uvec3" | "uvec4" => "uint",
        "vec2"  | "vec3"  | "vec4"  => "float",
        "dvec2" | "dvec3" | "dvec4" => "double",

        _ => unreachable!(),
    }
}

// See https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)#Vector_constructors
/// Returns vec type if the constructor is valid
pub fn validate_constructor(vec_type: &str, passed: &Vec<String>) -> String {
    let num_args = passed.len();
    let primitive = vec_primitive_type(vec_type);

    if num_args == 0 {
        exit_with_message(format!("Error: Type '{}' must be initialized with values", vec_type));
    }
    
    // Special case for 'vec3(1.)' or similar
    if num_args == 1 && Context::castable(&passed[0], vec_primitive_type(vec_type)) {
        return vec_type.to_owned();
    }

    match vec_type {
        "bvec2" | "ivec2" | "uvec2" | "vec2" | "dvec2" => {
            if num_args > 2 {
                exit_with_message(format!("Error: Too many arguments for '{}'", vec_type));
            }
            if !Context::castable(&passed[0], primitive) || !Context::castable(&passed[1], primitive) {
                exit_with_message(format!("Error: Both '{}' arguments must be castable to '{}'", vec_type, primitive));
            }
        }

        v3 @ "bvec3" | v3 @ "ivec3" | v3 @ "uvec3" | v3 @ "vec3" | v3 @ "dvec3" => {
            let mut v2 = v3.to_owned();
            v2.pop(); v2.push('2');

            if num_args > 3 {
                exit_with_message(format!("Error: Too many arguments for '{}'", vec_type));
            }

            // vec3 can be made of one vec2 and one primitive
            if num_args == 2 && 
                ! ( passed[0] == v2 && Context::castable(&passed[1], primitive)
                ||  passed[1] == v2 && Context::castable(&passed[0], primitive) ) 
            {
                exit_with_message(format!("Error: '{}' can be built from only one '{}' and one '{}' or three '{}'s", vec_type, v2, primitive, primitive));
            }

            if num_args == 3 && !(Context::castable(&passed[0], primitive) && Context::castable(&passed[1], primitive) && Context::castable(&passed[2], primitive)) {
                exit_with_message(format!("Error: All three '{}' arguments must be castable to '{}'", vec_type, primitive));
            }
        }

        v4 @ "bvec4" | v4 @ "ivec4" | v4 @ "uvec4" | v4 @ "vec4" | v4 @ "dvec4" => {
            let mut v2 = v4.to_owned();
            v2.pop(); v2.push('2');

            let mut v3 = v4.to_owned();
            v3.pop(); v3.push('3');

            if num_args > 4 {
                exit_with_message(format!("Error: Too many arguments for '{}'", vec_type));
            }

            // vec4 can be made of one vec3 and one primitive
            // or two vec2s
            if num_args == 2 && 
                ! ( passed[0] == v3 && Context::castable(&passed[1], primitive)
                ||  passed[1] == v3 && Context::castable(&passed[0], primitive) 
                ||  passed[0] == v2 && passed[1] == v2 ) 
            {
                exit_with_message(format!("Error: '{}' can be built from only two '{}'s, one '{}' and two '{}'s, one '{}' and one '{}', or four '{}'s", vec_type, v2, v2, primitive, v3, primitive, primitive));
            }
            
            // vec4 cn be made of one vec2 and two primitives
            if num_args == 3 && !(  (passed[0] == v2 && Context::castable(&passed[1], primitive) && Context::castable(&passed[2], primitive)) 
            || (passed[1] == v2 && Context::castable(&passed[0], primitive) && Context::castable(&passed[2], primitive))  
            || (passed[2] == v2 && Context::castable(&passed[0], primitive) && Context::castable(&passed[1], primitive)) )
            {
                exit_with_message(format!("Error: '{}' can be built from only two '{}'s, one '{}' and two '{}'s, one '{}' and one '{}', or four '{}'s", vec_type, v2, v2, primitive, v3, primitive, primitive));
            }

            // vec4 can be made of four primitives
            if num_args == 4 && 
              !(Context::castable(&passed[0], primitive) && Context::castable(&passed[1], primitive) 
                && Context::castable(&passed[2], primitive) && Context::castable(&passed[3], primitive)) 
            {
                exit_with_message(format!("Error: All four '{}' arguments must be castable to '{}'", vec_type, primitive))
            }
        }

        _ => {
            exit_with_message(format!("Error: 'vec' type '{}' is not implemented", vec_type));
        }
    }

    vec_type.to_owned()
}