// Modeled after the Rust lexer:
// See https://github.com/rust-lang/rust/blob/master/src/librustc_lexer/src/lib.rs

/// All allowed literal values within the language
#[derive(Debug, PartialEq)]
pub enum Literal {
    Int,                // Numbers without decimals
    Float,              // Numbers with decimals
    // Str,             // NOTE: sdf-lang does not allow strings, chracters, etc. -> Maybe in the future for debugging
}

/// All allowed characters within the language. 
/// 
/// Note that this is not a language token, rather a string decomposition
#[derive(Debug, PartialEq)]
pub enum Lexeme {
    CommentSingle,      // "//"
    CommentMulti,       // "/* .. */"
    
    Whitespace,         // Anything like space, tab, linefeed, etc.

    Literal(String),    // Typed values (needs to be parsed)
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
    // use regex::Regex;
    use super::Cursor;

    let mut cursor = Cursor::from_string(string);

    /*
        TODO:

        Track line number & column
        Report errors with useful information
    */

    println!("Tokenizing string...");

    std::iter::from_fn(move || {
        if cursor.is_finished() {
            None
        } else {
            Some(next_lexeme(&mut cursor))
        }
    })
}

pub fn strip(lexemes: &mut Vec<Lexeme>) {
    use Lexeme::*;

    lexemes.retain(|lexeme| {
        match lexeme {
            Whitespace 
            | CommentMulti
            | CommentSingle => false,

            _ => true,
        }
    })
}

fn next_lexeme(cursor: &mut super::Cursor) -> Lexeme {
    use Lexeme::*;

    match cursor.current_character() {

        // TODO: Allow hexadecimal, etc.
        // Literals
        n if n.is_ascii_digit() => {
            let from = cursor.current;
            while cursor.current_character().is_ascii_digit() {
                cursor.advance();
            }

            Literal(cursor.string[from..cursor.current].to_owned())
        }

        // Identifiers
        /* FIXME: Why doesn't this binding work? Gives Unknown
        c @ '_' |                   */
        c if (c.is_ascii_alphabetic() || c == '_') => {
            let from = cursor.current;

            let mut current = cursor.current_character();
            while current.is_ascii_alphanumeric() || current == '_' {
                cursor.advance();
                current = cursor.current_character();
            }

            Identifier(cursor.string[from..cursor.current].to_owned())
        }

        // Find whitespace and condence to a single Whitespace lexeme
        '\r'
        | '\n'
        | ' '
        | '\t' => {
            let is_whitespace = |c: char| -> bool {
                match c {
                    '\r' | '\n' | ' ' | '\t' => true,
                    _ => false,
                }
            };

            cursor.advance();

            while is_whitespace(cursor.current_character()) {
                cursor.advance();
            }

            Whitespace
        }

        // Check whether '/' means single line comment, a slash, or a multi-line comment
        '/' => {
            let next = cursor.peek(1);

            if next == '/' {
                // Skip to linefeed
                if let Some(offset) = cursor.seek_char('\n') {
                    cursor.advance_by(offset + 1);
                } else {
                    // TODO: When would this not work?
                    panic!("End of file?");
                }
                return CommentSingle;
            } else if next == '*' {
                // TODO: Skip to end of comment, then skip linefeed
                panic!("Multi-line comments not yet implemented");
                // return CommentMulti;
            }

            cursor.advance();
            Slash
        }

        '=' => {
            cursor.advance();
            Equals
        }
        
        ':' => {
            cursor.advance();
            Colon
        }
        
        ',' => {
            cursor.advance();
            Comma
        }

        ';' => {
            cursor.advance();
            Semicolon
        }
        
        '.' => {
            cursor.advance();
            Dot
        }

        '(' => {
            cursor.advance();
            ParenthesisOpen
        }

        ')' => {
            cursor.advance();
            ParenthesisClose
        }

        '{' => {
            cursor.advance();
            BraceOpen
        }

        '}' => {
            cursor.advance();
            BraceClose
        }

        '[' => {
            cursor.advance();
            BracketOpen
        }

        ']' => {
            cursor.advance();
            BracketClose
        }

        '@' => {
            cursor.advance();
            At
        }

        '*' => {
            cursor.advance();
            Star
        }

        '+' => {
            cursor.advance();
            Plus
        }

        '-' => {
            cursor.advance();
            Minus
        }

        '!' => {
            cursor.advance();
            Not   
        }

        '&' => {
            cursor.advance();
            And   
        }

        '|' => {
            cursor.advance();
            Or
        }

        '~' => {
            cursor.advance();
            Tilde
        }

        '<' => {
            cursor.advance();
            LessThan   
        }

        '>' => {
            cursor.advance();
            GreaterThan   
        }

        '^' => {
            cursor.advance();
            Caret   
        }

        '%' => {
            cursor.advance();
            Percent   
        }

        c => {
            println!("Unknown symbol, '{}'", c);
            cursor.advance();
            Unknown
        }
    }
}