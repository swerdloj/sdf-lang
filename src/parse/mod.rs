pub mod ast;
pub mod context;
pub mod glsl;

lalrpop_mod!(pub parser, "/parse/parser.rs");

pub struct Input {
    pub path: std::path::PathBuf,
    pub text: String,
}

impl Input {
    pub fn from_path<P: Into<std::path::PathBuf>>(path: P) -> Result<Self, std::io::Error> {
        let as_path = path.into();
        
        let text = std::fs::read_to_string(&as_path)?;
        
        Ok(Self {
            path: as_path,
            text,
        })
    }

    pub fn reload_text(&mut self) {
        self.text = std::fs::read_to_string(&self.path).unwrap();
    }

    /// span = (left, right)
    pub fn evaluate_span(&self, span: (usize, usize)) -> String {
        let mut line = 1;
        let mut column = 1;
    
        // Current is the current byte offset, meaning '\r' must increment this
        let mut current = 0usize;
    
        for c in self.text.chars() {
            if current == span.0 {
                break;
            }
    
            match c {
                // Advance to a new line, resetting the column
                '\n' => {
                    line += 1;
                    column = 1;
    
                    current += 1;
                }
    
                // Increase byte offset by one and ignore this
                '\r' => {
                    current += 1;
                }
    
                // Advance to the next column
                _ => {
                    current += 1;
                    column += 1;
                }
            }
        }
    
        // TODO: Use "right - left" to show how far error goes
        format!("{}:{}:{}", self.path.display(), line, column)
    }
}

/// Returns the parsed AST or formats the **default** lalrpop lexer error
pub fn parse(input: &Input) -> Result<ast::AST, String> {
    use lalrpop_util::ParseError;

    let ast = parser::ASTParser::new().parse(&input.text).map_err(|error| {
        // TODO: Print line and column (obtained via `token` and `location`)
        match error {
            ParseError::InvalidToken { location } => {
                let place = input.evaluate_span((location, location));
                format!("{}\nInvalid token, '{}', at location {}", place, &input.text[location..location+1], location)
            }

            ParseError::UnrecognizedEOF { location, expected } => {
                let place = input.evaluate_span((location, location));
                format!("{}\nFile ended while expecting one of {:?}", place, vec_to_string(expected))
            }
            
            ParseError::UnrecognizedToken { token, expected } => {
                let place = input.evaluate_span((token.0, token.2));
                format!("{}\nExpected one of {}, but found '{}'", place, vec_to_string(expected), (token.1).1)
            }
            
            ParseError::ExtraToken { token } => {
                let place = input.evaluate_span((token.0, token.2));
                format!("{}\nExtra token, {}\n", place, token.1)
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

// TODO: Implement a test function that compiles a series of test files

/// Simple parser test cases. Note that only full ASTs are generated
/// 
/// This is is to prevent lalrpop from generating more code than needed
#[allow(unused)]
#[cfg(test)]
mod parser_test {
    use crate::parse::parser;

    fn test_input() {

    }
}