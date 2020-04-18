pub mod vec;
pub mod mat;
pub mod functions;

// TODO: Implement vec casts like uvec to ivec, etc.


/// Whether a narrowing conversion via 'as' is valid.
pub fn narrow_castable(from: &str, to: &str) -> Result<bool, String> {
    if from == to {
        return Ok(true);
    }

    // TODO: Can arrays of the same size cast between compatible base types??
    if from.contains("[") || to.contains("[") {
        return Ok(false);
        // return Err(format!("Arrays cannot be cast (tried casting '{}' to '{}')", from, to));
    }

    match from {
        "double" => {
            match to {
                "float" | "int" | "uint" => Ok(true),
                _ => Ok(false),
            }
        }
        
        "float" => {
            match to {
                "double" | "int" | "uint" => Ok(true),
                _ => Ok(false),
            }
        }

        "int" => {
            match to {
                "float" | "double" | "uint" => Ok(true),
                _ => Ok(false),
            }
        }

        "uint" => {
            match to {
                "float" | "int" | "double" => Ok(true),
                _ => Ok(false),
            }
        }

        x => Err(format!("Type '{}' has no cast implementations. Cannot cast from '{}' to '{}'.", x, from, to)),
    }
}

/// Whether types can be implicitly cast (non-narrowing cast)
pub fn castable(from: &str, to: &str) -> Result<bool, String> {
    if from == to {
        return Ok(true);
    }

    // Cannot cast between array types (even for compatible base types)
    if from.contains("[") || to.contains("[") {
        return Ok(false);
        // return Err(format!("Arrays cannot be cast (tried casting '{}' to '{}')", from, to));
    }

    match to {
        "double" => {
            match from {
                "float" | "int" | "uint" => Ok(true),
                _ => Ok(false),
            }
        }

        "float" => {
            match from {
                "int" | "uint" => Ok(true),
                _ => Ok(false),
            }
        }

        "int" => {
            match from {
                "uint" => Ok(true),
                _ => Ok(false),
            }
        }

        "uint" => {
            match from {
                _ => Ok(false),
            }
        }

        "bool" => {
            match from {
                "double" | "fload" | "int" | "uint" => Ok(false),
                _ => Ok(false),
            }
        }

        x => Err(format!("Type '{}' has no cast implementations. Cannot cast from '{}' to '{}'.", x, from, to)),
    }
}