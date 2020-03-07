use super::analyze::Lexeme;

#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Identifier {
        name: String,
        id_type: Primitive,
        value: Option<String>,
    },
    Primitive(Primitive),
    Delimiter(Delimiter),
    BinaryOperator(BinaryOperator),
    UnaryOperator(UnaryOperator),
    // FIXME: Remove this after finishing implementations
    Uknown,
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
pub enum Primitive {
    Vec2,   // "vec2"
    Vec3,   // "vec3"
    Vec4,   // "vec4"
    Int,    // "int"
    Float,  // "float"
    Matrix, // "mat#x#"
}

#[derive(Debug)]
pub enum Delimiter {
    Comma,      // ","
    Colon,      // ":"
    Semicolon,  // ";"
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

pub fn tokenize_lexemes(lexemes: Vec<Lexeme>) -> impl Iterator<Item = Token> {
    let mut cursor = LexemeCursor::from_lexemes(lexemes);

    std::iter::from_fn(move || {
        if cursor.current == cursor.lexemes.len() {
            None
        } else {
            Some(next_token(&mut cursor))
        }
    })
}

fn next_token(cursor: &mut LexemeCursor) -> Token {
    match cursor.current_lexeme() {
        // Identifiers
        Lexeme::Identifier(name) => {
            match name.as_str() {
                "let" => {
                    cursor.advance();
                    Token::Keyword(Keyword::Let)
                }

                "if" => {
                    cursor.advance();
                    Token::Keyword(Keyword::If)
                }

                "enum" => {
                    cursor.advance();
                    Token::Keyword(Keyword::Enum)
                }

                "struct" => {
                    cursor.advance();
                    Token::Keyword(Keyword::Struct)
                }

                "scene" => {
                    cursor.advance();
                    Token::Keyword(Keyword::Scene)
                }

                // User-defined identifier
                _ => {
                    // unimplemented!();

                    let id_name = name.clone();
                    cursor.advance();


                    Token::Identifier {
                        name: id_name,
                        id_type: Primitive::Int,
                        value: Some("Nothing".to_owned()),
                    }
                }
            }
        }

        // Lexeme::LiteralValue(literal) => {
        //     match literal {
        //         super::analyze::Literal::Float(number) => {
        //             unimplemented!()
        //         }
                
        //         super::analyze::Literal::Int(number) => {
        //             unimplemented!()
        //         }
        //     }
        // }

        Lexeme::And => {
            cursor.advance();

            // And comparison
            if *cursor.current_lexeme() == Lexeme::And {
                cursor.advance();
                return Token::BinaryOperator(BinaryOperator::And);
            } else {
                panic!("Expected '&&', found '&'");
            }
        }

        Lexeme::Plus => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Token::BinaryOperator(BinaryOperator::AddAssign);
            }

            Token::BinaryOperator(BinaryOperator::Add)
        }

        Lexeme::Minus => {
            cursor.advance();

            match *cursor.current_lexeme() {
                // "-="
                Lexeme::Equals => {
                    cursor.advance();
                    return Token::BinaryOperator(BinaryOperator::SubtractAssign);
                }

                // TODO: How to know if subtract or negate?

                _ => {}
            }

            Token::UnaryOperator(UnaryOperator::Negate)
        }

        Lexeme::Not => {
            cursor.advance();

            // NotEqualTo comparison
            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Token::BinaryOperator(BinaryOperator::NotEqualTo);
            }


            // Logical not
            Token::UnaryOperator(UnaryOperator::Not)
        }

        Lexeme::Equals => {
            cursor.advance();

            // EqualTo comparison 
            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Token::BinaryOperator(BinaryOperator::EqualTo);
            }

            // Assignment
            Token::BinaryOperator(BinaryOperator::Assign)
        }

        Lexeme::Slash => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Token::BinaryOperator(BinaryOperator::DivideAssign);
            }

            Token::BinaryOperator(BinaryOperator::Divide)
        }

        Lexeme::Star => {
            cursor.advance();

            if *cursor.current_lexeme() == Lexeme::Equals {
                cursor.advance();
                return Token::BinaryOperator(BinaryOperator::MultiplyAssign);
            }

            Token::BinaryOperator(BinaryOperator::Multiply)
        }

        Lexeme::Percent => {
            cursor.advance();
            Token::BinaryOperator(BinaryOperator::Modulo)
        }

        Lexeme::Comma => {
            cursor.advance();
            Token::Delimiter(Delimiter::Comma)
        }

        Lexeme::Colon => {
            cursor.advance();
            Token::Delimiter(Delimiter::Colon)
        }

        Lexeme::Semicolon => {
            cursor.advance();
            Token::Delimiter(Delimiter::Semicolon)
        }

        // Unhandled or unimplemented
        x => {
            // panic!("Unhandled lexeme. '{:?}'", x);

            // FIXME: Remove this later
            cursor.advance();
            Token::Uknown
        }
    }
}