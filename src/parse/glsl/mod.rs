pub mod vec;
pub mod mat;
pub mod functions;

use crate::exit_with_message;

/// Whether a narrowing conversion via 'as' is valid
pub fn narrow_castable(from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }

    match from {
        "double" => {
            match to {
                "float" | "int" | "uint" => true,
                _ => false,
            }
        }
        
        "float" => {
            match to {
                "double" | "int" | "uint" => true,
                _ => false,
            }
        }

        "int" => {
            match to {
                "float" | "double" | "uint" => true,
                _ => false,
            }
        }

        "uint" => {
            match to {
                "float" | "int" | "double" => true,
                _ => false,
            }
        }

        _ => false,
    }
}

/// Whether types can be implicitly cast (non-narrowing cast)
pub fn castable(from: &str, to: &str) -> bool {
    if from == to {
        return true;
    }

    // TODO: Implement vec casts? Like uvec to ivec, etc.

    match to {
        "double" => {
            match from {
                "float" | "int" | "uint" => true,
                _ => false,
            }
        }

        "float" => {
            match from {
                "int" | "uint" => true,
                _ => false,
            }
        }

        "int" => {
            match from {
                "uint" => true,
                _ => false,
            }
        }

        "uint" => {
            match from {
                "int" => true,
                _ => false,
            }
        }

        "bool" => {
            match from {
                "double" | "fload" | "int" | "uint" => false,
                _ => false,
            }
        }

        x => {
            exit_with_message(format!("Type '{}' has no cast implementations. Cannot cast from '{}' to '{}'.", x, from, to));
            unreachable!();
        }
    }
}