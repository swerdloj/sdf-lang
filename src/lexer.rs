// Modeled after the Rust lexer:
// See https://github.com/rust-lang/rust/blob/master/src/librustc_lexer/src/lib.rs

/// All allowed literal values within the language
enum Literal {
    Int,                // Numbers without decimals
    Float,              // Numbers with decimals
    // Str,             // NOTE: sdf-lang does not allow strings, chracters, etc. -> Maybe in the future for debugging
}

/// All allowed characters within the language. 
/// 
/// Note that this is not a language token, rather a string decomposition
enum TokenKind {
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

pub fn tokenize_string(string: String) {
    /*
        TODO:
        1. Generate TokenKinds
        2. ?
    */


    println!("TODO: Tokenize string");
}