use super::castable;

// see http://www.shaderific.com/glsl-functions

// TODO: If a function requires a float, and the passed value is castable to float,
//       that still won't work because of GLSL's overloading (give this error message)

// FIXME: A lot of this code could be simplified (lots of repetition)

pub fn is_builtin(function: &str) -> bool {
    match function {
        "radians" | "degrees" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" |
        "exp" | "log" | "exp2" | "log2" |"sqrt" | "inversesqrt" |"abs" | "sign" | 
        "floor" | "ceil" | "fract" |"normalize" | "pow" | "mod" | "min" | "max" | 
        "clamp" |"mix" | "step" | "smoothstep" | "distance" | "dot" | "cross" |
        "faceforward" | "reflect" | "refract" | "matrixCompMult" | "lessThan" |
        "lessThanEqual" |"greaterThan" | "greaterThanEqual" | "equal" | "notEqual" |
        "any" | "all" | "not" |"texture2D" | "textureCube" | "length"
           => true,

         _ => false,
    }
}

/// Workaround for overloaded methods within GLSL (sdf-lang does not support overloading)
pub fn validate_function(function: &str, types: &Vec<String>) -> Result<String, String> {   
    match types.len() {
        0 => Err(format!("Error: '{}' does not accept zero parameters", function)),
        

        1 => validate_single_param(function, &types[0]),
        2 => validate_two_params(function, types),
        3 => validate_three_params(function, types),

        n => Err(format!("Error: Function '{}' does not accept {} parameters", function, n)),
    }
}

// TODO: Matrices:  matrixCompMult,
//       Bool Vecs: lessThan, lessThanEqual, greaterThan, greaterThanEqual, equal, notEqual, any, all, not

fn validate_three_params(function: &str, types: &Vec<String>) -> Result<String, String> {
    match function {
        // return_type = function(return_type, return_type or float, return_type or float)
        "clamp" => {
            // FIXME: All four cases aren't need here
            if (  ( types[0] == types[1]) && (types[1] == types[2]) )
               || ( types[1] == "float"   && types[2] == types[0]   )
               || ( types[1] == "float"   && types[2] == "float"    )  
               || ( types[1] == "float"   && types[2] == types[0]   )
            {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[0].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[0], types)),
                }
            } else {
                Err(format!("Error: '{}' requires all three parameters to be same type unless the second and third are floats", function))
            }
        }

        // return_type = function(return_type, return_type, return_type or float)
        "mix" => {
            if (types[0] == types[1]) && ((types[1] == types[2]) || types[2] == "float") {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[0].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[2], types)),
                }
            } else {
                Err(format!("Error: '{}' requires all three parameters to be same type unless the third is a float (got {:?})", function, types))
            }
        }

        // return_type = function(return_type or float, return_type or float, return_type)
        "smoothstep" => {
            if ((types[0] == types[1]) && (types[1] == types[2]))
               || ( types[0] == "float" && types[1] == "float" )  
            {
                match types[2].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[2].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[2], types)),
                }
            } else {
                Err(format!("Error: '{}' requires all three parameters to be same type unless the first and second are floats", function))
            }
        }

        // return_type = function(return_type, return_type, return_type)
        "faceforward" => {
            if (types[0] == types[1]) && (types[1] == types[2]) {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[0].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[0], types)),
                }
            } else {
                Err(format!("Error: '{}' requires all three parameters to be same type", function))
            }
        }

        // return_type = function(return_type, return_type, float)
        "refract" => {
            if (types[0] == types[1]) && (types[2] == "float") {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[0].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[0], types)),
                }
            } else {
                Err(format!("Error: '{}' requires two of the same types and a float for the third parameter", function))
            }
        }

        "texture2D" => {
            if (types[0] == "sampler2D") && (types[1] == "vec2") && (types[2] == "float") {
                Ok("vec4".to_owned())
            } else {
                Err(format!("Error: '{}' with three parameters requires 'sampler2D', 'vec2', and 'float'. Found '{:?}'", function, types))
            }
        }

        "textureCube" => {
            if (types[0] == "samplerCube") && (types[1] == "vec2") && (types[2] == "float") {
                Ok("vec4".to_owned())
            } else {
                Err(format!("Error: '{}' with three parameters requires 'samplerCube', 'vec2', and 'float'. Found '{:?}'", function, types))
            }
        }

        _ => Err(format!("Error: '{}' does not accept three parameters", function)),
    }
}

fn validate_two_params(function: &str, types: &Vec<String>) -> Result<String, String> {
    match function {
        // vec4 = function(sampler2D, vec2)
        "texture2D" => {
            if types[0] == "sampler2D" && types[1] == "vec2" {
                Ok("vec4".to_owned())
            } else {
                Err(format!("Error: '{}' with two parameters requires 'sampler2D' and 'vec2'. Found '{:?}'", function, types))
            }
        }

        // vec4 = function(samplerCube, vec3)
        "textureCube" => {
            if types[0] == "samplerCube" && types[1] == "vec3" {
                Ok("vec4".to_owned())
            } else {
                Err(format!("Error: '{}' with two parameters requires 'samplerCube' and 'vec3'. Found '{:?}'", function, types))
            }
        }

        // vec3 = function(vec3, vec3)
        "cross" => {
            if types[0] == "vec3" && types[1] == "vec3" {
                Ok("vec3".to_owned())
            } else {
                Err(format!("Error: '{}' accepts two of 'vec3'. Got {:?}", function, types))
            }
        }

        // return_type = function(return_type, return_type)
        "atan" | "pow" | "reflect" => {
            if types[0] == types[1] {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[0].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[0], types)),
                }
            } else {
                Err(format!("Error: '{}' requires two of the same types", function))
            }
        }

        // float = function(type, type)
        "distance" | "dot" => {
            if types[0] == types[1] {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok("float".to_owned()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[0], types)),
                }
            } else {
                Err(format!("Error: '{}' requires two of the same types", function))
            }
        }

        // return_type = function(return_type, return_type or float)
        "mod" | "min" | "max" => {
            if (types[0] == types[1]) || types[1] == "float" {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[0].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[0], types)),
                }
            } else {
                Err(format!("Error: '{}' requires two of the same type unless the second parameter is a float", function))
            }
        }

        // return_type = function(return_type or float, return_type)
        "step" => {
            if (types[0] == types[1]) || types[0] == "float" {
                match types[0].as_ref() {
                    "float" | "vec2" | "vec3" | "vec4" => Ok(types[0].clone()),
                    _ => Err(format!("Error: '{}' does not accept type '{}' (got {:?})", function, &types[0], types)),
                }
            } else {
                Err(format!("Error: '{}' requires two of the same types unless the first parameter is a float", function))
            }
        }

        _ => Err(format!("Error: '{}' does not accept two parameters", function)),
    }
}

fn validate_single_param(function: &str, ty: &str) -> Result<String, String> {
    match function {
        "radians" | "degrees"     | 
        "sin"     | "cos"         | "tan"   |
        "asin"    | "acos"        | "atan"  |
        "exp"     | "log"         | "exp2"  | "log2" |
        "sqrt"    | "inversesqrt" |
        "abs"     | "sign"        | 
        "floor"   | "ceil"        | "fract" |
        "normalize"
        => {
            // This is ok here (see top todo)
            if castable(ty, "float")? {
                return Ok("float".to_owned());
            }

            match ty {
                "float" | "vec2" | "vec3" | "vec4" => Ok(ty.to_owned()),
                _ => Err(format!("Error: '{}' does not work with type '{}'", function, ty)),
            }
        }

        "length" => {
            if castable(ty, "float")? {
                return Ok("float".to_owned());
            }

            match ty {
                "float" | "vec2" | "vec3" | "vec4" => Ok("float".to_owned()),
                _ => Err(format!("Error: '{}' does not work with type '{}'", function, ty)),
            }
        }

        _ => Err(format!("Error: '{}' does not accept one parameter", function)),
    }
}