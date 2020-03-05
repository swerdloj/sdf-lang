// Modeled after the Rust lexer:
// See https://github.com/rust-lang/rust/blob/master/src/librustc_lexer/src/lib.rs

/// All allowed literal values within the language
#[derive(Debug)]
pub enum Literal {
    Int,                // Numbers without decimals
    Float,              // Numbers with decimals
    // Str,             // NOTE: sdf-lang does not allow strings, chracters, etc. -> Maybe in the future for debugging
}

/// All allowed characters within the language. 
/// 
/// Note that this is not a language token, rather a string decomposition
#[derive(Debug)]
pub enum Lexeme {
    CommentSingle,      // "//"
    CommentMulti,       // "/* .. */"
    
    Whitespace,         // Anything like space, tab, linefeed, etc.

    Literal(Literal),   // Typed values
    Identifier(String), // Includes keywords
    
    Equals,             // "="
    Colon,              // ":"
    Comma,              // ","
    Semicolon,          // ";"
    Dot,                // "."

    ParenthesisOpen,    // "("
    ParenthesisClose,   // ")"
    BraceOpen,          // "{"
    BraceClose,         // "}"
    BracketOpen,        // "["
    BracketClose,       // "]"

    At,                 // "@"

    Star,               // "*"
    Plus,               // "+"
    Minus,              // "-"
    Slash,              // "/"

    Not,                // "!"
    And,                // "&"
    Or,                 // "|"
    Tilde,              // "~"
    LessThan,           // "<"
    GreaterThan,        // ">"
    Caret,              // "^"
    Percent,            // "%"

    Unknown,            // Invalid character
}

pub fn tokenize_string(string: String) -> impl Iterator<Item = Lexeme> {
    use regex::Regex;
    use super::Cursor;

    let mut cursor = Cursor::from_string(string);

    /*
        TODO:
        1. Generate symbolic tokens (string analysis)
        2. Generate language tokens (token stream analysis)
    */

    // underscore followed by numbers/characters/underscores OR character followed by numbers/characters/underscores
    let literal_regex = Regex::new(r"(_)*+([a-bA-z]+(_)+[0-9])*").unwrap();


    println!("Tokenizing string");

    std::iter::from_fn(move || {
        if cursor.is_finished() {
            None
        } else {
            Some(match_lexeme(&mut cursor))
        }
    })
}

fn match_lexeme(cursor: &mut super::Cursor) -> Lexeme {
    use Lexeme::*;

    match cursor.current_character() {
        "/" => {
            let next = cursor.peek_ahead(1);

            // FIXME: Temporary
            // cursor.advance();


            if next == "/" {
                // TODO: Skip to linefeed
                return CommentSingle;
            } else if next == "*" {
                // TODO: Skip to end of comment, then skip linefeed
                return CommentMulti;
            }

            // TODO: Move the cursor forward as calculated above


            Slash
        }

        "{" => {
            cursor.advance();
            BraceOpen
        }

        "}" => {
            cursor.advance();
            BraceClose
        }
        
        "=" => {
            cursor.advance();
            Equals
        }
        
        ":" => {
            cursor.advance();
            Colon
        }
        
        ";" => {
            cursor.advance();
            Semicolon
        }
        
        "," => {
            cursor.advance();
            Comma
        }
        
        "." => {
            cursor.advance();
            Dot
        }

        // TODO: The rest

        x => {
            println!("{}", x);
            Unknown
        }
    }
}