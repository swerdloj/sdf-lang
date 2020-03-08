use super::analyze::Lexeme;

#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    Literal(Literal),
    Delimiter(Delimiter),
    BinaryOperator(BinaryOperator),
    UnaryOperator(UnaryOperator),

    Tag(String),    // "@uniform", etc.
}

#[derive(Debug)]
pub enum Keyword {
    Let,    // "let"
    If,     // "if"
    Scene,  // "scene"
    Enum,   // "enum"
    Struct, // "struct"
    
    // TODO: Account for GLSL keywords so user cannot use them for variable names, etc.
}

#[derive(Debug)]
pub enum Literal {
    Float(String),
    Int(String),
}

// #[derive(Debug)]
// pub enum Primitive {
//     Vec2,   // "vec2"
//     Vec3,   // "vec3"
//     Vec4,   // "vec4"
//     Int,    // "int"
//     Float,  // "float"
//     Matrix, // "mat#x#"
// }

#[derive(Debug)]
pub enum Delimiter {
    Comma,              // ","
    Colon,              // ":"
    Semicolon,          // ";"

    ParenthesisOpen,    // "("
    ParenthesisClose,   // ")"
    BraceOpen,          // "{"
    BraceClose,         // "}"
    BracketOpen,        // "["
    BracketClose,       // "]"
}

#[derive(Debug)]
pub enum BinaryOperator {
    Assign,                 // "a = b"
    AddAssign,              // "a += b"
    SubtractAssign,         // "a -= b"
    DivideAssign,           // "a /= b"
    MultiplyAssign,         // "a *= b"

    Add,                    // "a + b"
    Subtract,               // "a - b"
    Divide,                 // "a / b"
    Multiply,               // "a * b"
    Modulo,                 // "a % b"

    GreaterThan,            // "a > b"
    LessThan,               // "a < b"
    GreaterThanOrEqualTo,   // "a >= b"
    LessThanOrEqualTo,      // "a <= b"
    EqualTo,                // "a == b"
    NotEqualTo,             // "a != b"

    And,                    // "a && b"
    Or,                     // "a || b"
}

#[derive(Debug)]
pub enum UnaryOperator {
    Negate,     // "-a"
    Not,        // "!a"

    // TODO: Reference could be translated as `inout` in GLSL, etc.
}

struct LexemeCursor {
    lexemes: Vec<Lexeme>,
    current: usize,
}

impl LexemeCursor {
    pub fn from_lexemes(lexemes: Vec<Lexeme>) -> Self {
        LexemeCursor {
            lexemes,
            current: 0,
        }
    }

    pub fn advance(&mut self) {
        self.current += 1;
    }

    pub fn current_lexeme(&self) -> &Lexeme {
        &self.lexemes[self.current]
    }
}

pub fn tokenize_string(input: String) -> Vec<Token> {
    let lexemes = super::analyze::analyze_string(input);
    tokenize_lexemes(lexemes).collect()
}

// TODO: Could probably remove the iterator and just return a `Vec<Token>`
pub fn tokenize_lexemes(lexemes: Vec<Lexeme>) -> impl Iterator<Item = Token> {
    let mut cursor = LexemeCursor::from_lexemes(lexemes);

    let mut open_parenthesis_stack: Vec<Lexeme> = Vec::new();

    std::iter::from_fn(move || {
        next_token(&mut cursor, &mut open_parenthesis_stack)
    })
}

fn next_token(cursor: &mut LexemeCursor, parenthesis_stack: &mut Vec<Lexeme>) -> Option<Token> {
    // FIXME: Parenthesis errors print the open type, but should print close type
    match cursor.current_lexeme() {

        // Identifiers
        Lexeme::Identifier(name) => {
            match name.as_str() {
                "let" => {
                    cursor.advance();
                    Some(Token::Keyword(Keyword::Let))
                }

                "if" => {
                    cursor.advance();
                    Some(Token::Keyword(Keyword::If))
                }

                "enum" => {
                    cursor.advance();
                    Some(Token::Keyword(Keyword::Enum))
                }

                "struct" => {
                    cursor.advance();
                    Some(Token::Keyword(Keyword::Struct))
                }

                "scene" => {
                    cursor.advance();
                    Some(Token::Keyword(Keyword::Scene))
                }

                // User-defined identifier
                _ => {
                    let id_name = name.clone();
                    cursor.advance();

                    Some(Token::Identifier(id_name))
                }
            }
        }

        Lexeme::ParenthesisOpen => {
            cursor.advance();
            parenthesis_stack.push(Lexeme::ParenthesisOpen);

            Some(Token::Delimiter(Delimiter::ParenthesisOpen))
        }
        
        Lexeme::ParenthesisClose => {
            cursor.advance();
            let kind = parenthesis_stack.pop().expect("Closing parenthesis has no mathing opener");

            if kind != Lexeme::ParenthesisOpen {
                panic!("Expected '{:?}', found ')'", kind);
            }

            Some(Token::Delimiter(Delimiter::ParenthesisClose))
        }

        Lexeme::BraceOpen => {
            cursor.advance();
            parenthesis_stack.push(Lexeme::BraceOpen);

            Some(Token::Delimiter(Delimiter::BraceOpen))
        }

        Lexeme::BraceClose => {
            cursor.advance();
            let kind = parenthesis_stack.pop().expect("Closing brace has no mathing opener");

            if kind != Lexeme::BraceOpen {
                panic!("Expected '{:?}', found '}}'", kind);
            }

            Some(Token::Delimiter(Delimiter::BraceClose))
        }

        Lexeme::BracketOpen => {
            cursor.advance();
            parenthesis_stack.push(Lexeme::BracketOpen);

            Some(Token::Delimiter(Delimiter::BracketOpen))
        }

        Lexeme::BracketClose => {
            cursor.advance();
            let kind = parenthesis_stack.pop().expect("Closing bracket has no mathing opener");

            if kind != Lexeme::BracketOpen {
                panic!("Expected '{:?}', found ']'", kind);
            }

            Some(Token::Delimiter(Delimiter::BracketClose))
        }

        Lexeme::LiteralValue(literal) => {
            // FIXME: `r` is an ownership workaround
            let r = match literal {
                super::analyze::Literal::Float(number) => {
                    Token::Literal(Literal::Float(number.clone()))
                }
                
                super::analyze::Literal::Int(number) => {
                    Token::Literal(Literal::Int(number.clone()))
                }
            };

            cursor.advance();
            Some(r)
        }

        Lexeme::GreaterThan => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::GreaterThanOrEqualTo));
            }

            Some(Token::BinaryOperator(BinaryOperator::GreaterThan))
        }

        Lexeme::LessThan => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::LessThanOrEqualTo));
            }

            Some(Token::BinaryOperator(BinaryOperator::LessThan))
        }

        Lexeme::At => {
            cursor.advance();

            if let Lexeme::Identifier(name) = cursor.current_lexeme() {
                let tag_identifier = name.clone();
                cursor.advance();

                return Some(Token::Tag(tag_identifier));
            } else {
                panic!("Expected '@<identifier>', found only '@'");
            }
        }

        Lexeme::And => {
            cursor.advance();

            // And comparison
            if *cursor.current_lexeme() == Lexeme::And {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::And));
            } else {
                panic!("Expected '&&', found '&'");
            }
        }

        Lexeme::Or => {
            cursor.advance();

            // Or comparison
            if *cursor.current_lexeme() == Lexeme::Or {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::Or));
            } else {
                panic!("Expected '||', found '|'");
            }
        }

        Lexeme::Plus => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::AddAssign));
            }

            Some(Token::BinaryOperator(BinaryOperator::Add))
        }

        Lexeme::Minus => {
            cursor.advance();

            match *cursor.current_lexeme() {
                // "-="
                Lexeme::Equals => {
                    cursor.advance();
                    return Some(Token::BinaryOperator(BinaryOperator::SubtractAssign));
                }

                // TODO: How to know if subtract or negate?

                _ => {}
            }

            Some(Token::BinaryOperator(BinaryOperator::Subtract))
        }

        Lexeme::Not => {
            cursor.advance();

            // NotEqualTo comparison
            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::NotEqualTo));
            }


            // Logical not
            Some(Token::UnaryOperator(UnaryOperator::Not))
        }

        Lexeme::Equals => {
            cursor.advance();

            // EqualTo comparison 
            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::EqualTo));
            }

            // Assignment
            Some(Token::BinaryOperator(BinaryOperator::Assign))
        }

        Lexeme::Slash => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::DivideAssign));
            }

            Some(Token::BinaryOperator(BinaryOperator::Divide))
        }

        Lexeme::Star => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Some(Token::BinaryOperator(BinaryOperator::MultiplyAssign));
            }

            Some(Token::BinaryOperator(BinaryOperator::Multiply))
        }

        Lexeme::Percent => {
            cursor.advance();
            Some(Token::BinaryOperator(BinaryOperator::Modulo))
        }

        Lexeme::Comma => {
            cursor.advance();
            Some(Token::Delimiter(Delimiter::Comma))
        }

        Lexeme::Colon => {
            cursor.advance();
            Some(Token::Delimiter(Delimiter::Colon))
        }

        Lexeme::Semicolon => {
            cursor.advance();
            Some(Token::Delimiter(Delimiter::Semicolon))
        }

        Lexeme::EndOfStream => {
            cursor.advance();

            if !parenthesis_stack.is_empty() {
                panic!("Unclosed parenthesis: {:?}", parenthesis_stack);
            }

            // End the iterator
            None
        }

        // Unhandled or unimplemented
        x => {
            panic!("Unhandled lexeme: '{:?}'", x);
        }
    }
}