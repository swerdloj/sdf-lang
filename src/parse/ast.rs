// AST stuff

/// AST root node
#[derive(Debug)]
pub enum AST {
    Function,
    Scene,
}

#[derive(Debug)]
pub enum Node {
    // StatementBlock(Vec<Node>),
    Statement,
    Expression,
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Binary {
        LHS: Box<Expression>,
        Operator: BinaryOperator,
        RHS: Box<Expression>,
    }
}

#[derive(Debug)]
pub enum BinaryOperator {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum Statement {
    Let {
        Ident: String,
        Type: Option<String>,
        Expression: Option<Expression>,
    }
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