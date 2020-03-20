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
                struct Type {
                    field1: int,
                }

                scene main {
                    let x = 2 + 3;

                    let y = 7;

                    let z: Type {
                        field1: (x * y) - 4,
                    };
                }
        ");
    }
}