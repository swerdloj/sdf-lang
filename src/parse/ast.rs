// AST stuff

/// AST root node
#[derive(Debug)]
pub enum AST {
    Function {
        parameters: Vec<(String, String)>,
        return_type: Option<String>,
        statements: Vec<Statement>,
    },
    Scene {
        name: String,
        statements: Vec<Statement>,
    },
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary {
        lhs: Box<Expression>,
        operator: BinaryOperator,
        rhs: Box<Expression>,
    },
}

#[derive(Debug)]
pub struct Constructor {
    pub ty: String,
    pub fields: Vec<(String, Expression)>,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Plus,
    Minus,
}

pub enum AssignmentOperator {
    Assign,
}

#[derive(Debug)]
pub enum Statement {
    Let {
        ident: String,
        ty: Option<String>,
        expression: Option<Expression>,
    },
    LetConstructor {
        ident: String,
        constructor: Constructor,
    },
}

#[derive(Debug)]
/// GLSL Types
pub enum Literal {
    Float(f32),
    Double(f64),
    // FIXME: Type should default to i64, then decide on i/u 32 later (for parsing)
    Int(i32),
    UInt(u32),
    Bool(bool),
}