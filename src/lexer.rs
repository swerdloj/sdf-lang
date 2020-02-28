enum Token {
    identifier(String), // TODO: How to handle the type? Part of AST node?
    keyword(String),    // TODO: is this correct?
    
    Assignment,         // "="
    TypeSpecifier,      // ":"
    StatementEnd,       // ";"

    ScopeBegin,         // "{"
    ScopeEnd,           // "}"
    ConstructorBegin,   // "{"
    ConstructorEnd,     // "}"
    FunctionBegin,      // "("
    FunctionEnd,        // ")"

    OperatorMethod,     // "."
    OperatorField,      // "."    
    OperatorPlus,       // "+"
    OperatorMinus,      // "-"
    OperatorDivide,     // "/"
    OperatorMultiply,   // "*"

    // TODO: How to handle comments?
    CommentSingle,      // "//"
    CommentMultiBegin,  // "/*"
    CommentMultiEnd,    // "*/"
}

pub fn tokenize_string(string: String) {
    println!("TODO: Tokenize string");
}