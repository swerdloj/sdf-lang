pub mod ast;
pub mod context;

lalrpop_mod!(pub parser, "/parse/parser.rs");


/// Returns the parsed AST or formats the **default** lalrpop lexer error
pub fn parse(input: &str, context: &mut context::Context) -> Result<ast::AST, String> {
    use lalrpop_util::ParseError;

    let ast = parser::ASTParser::new().parse(context, input).map_err(|error| {
        // TODO: Print line and column (obtained via `token` and `location`)
        match error {
            ParseError::InvalidToken { location } => {
                format!("Invalid token, '{}', at location {}", &input[location..location+1], location)
            }

            ParseError::UnrecognizedEOF { location: _, expected } => {
                format!("File ended while expecting one of {:?}", vec_to_string(expected))
            }
            
            ParseError::UnrecognizedToken { token, expected } => {
                format!("Expected one of {}, but found '{}'", vec_to_string(expected), (token.1).1)
            }
            
            ParseError::ExtraToken { token: _ } => {
                format!("TODO: extra token error")
            }
            
            ParseError::User { error } => {
                error.to_owned()
            }
        }
    });

    ast
}

/// Makes lalrpop errors readable
fn vec_to_string(vec: Vec<String>) -> String {
    let mut string = String::new();
    for item in vec {
        string.push_str(&item);
        string.push_str(", ");
    }

    // Remove trailing ", "
    string.pop();
    string.pop();

    string
}

/// Simple parser test cases. Note that only full ASTs are generated
/// 
/// This is is to prevent lalrpop from generating more code than needed
#[allow(unused)]
#[cfg(test)]
mod parser_test {
    use crate::parse::parser;

    fn test_input(input: &str) {
        let ast = super::parse(input, &mut super::context::Context::new());

        println!("{:#?}", ast.unwrap());
    }
    
    #[test]
    fn expression_integration() {
       test_input("
                scene main {
                    let x = 2 + 3;

                    let y = 7;

                    let z: Type {
                        field1: (x * y) - 4,
                    };
                }
        ");
    }

    #[test]
    fn raw_expressions() {
       test_input("
            scene main {
                1 / -x + 2 != 7 * 1 - (3 / 4) ;
            }
        ");
    }

    #[test]
    fn comments() {
       test_input("
            /*
            struct commented_out {
                field1: value1 = default,
                field2: no_default,
            }
            stuff in comment here
            */

            /* Same line */

            // Comment here

            scene main {
                let box: Box {
                    field1: value1,
                    field2: value2,
                };
                let x = 7;
                let y = x; // Another comment
            }
        ");
    }

    #[test]
    fn struct_function_scene_and_constructor() {
       test_input("
            struct something {
                field1: int = 12,
                field2: float,
            }

            scene main {
                let box: Box {
                    field1: 1,
                    field2: 1,
                };
                let x = 7;
                let y = x;
            }

            fn function() {
                statements;
            }
        ");
    }

    #[test]
    fn expression_statement() {
       test_input("
            scene main {
                expression_as_statement;
            }
        ");
    }

    #[test]
    fn let_statement() {
       test_input("
            scene main {
                let x = 1;
            }
        ");
    }

    #[test]
    fn let_constructor() {
       test_input("
            scene main {
                let x: y { 
                    f1: 1, 
                    f2: 2,
                };
            }
        ");
    }
}