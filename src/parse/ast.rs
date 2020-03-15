// AST enums, types, and structs

/// AST root
pub type AST = Vec<Item>;

#[derive(Debug)]
pub enum Item {
    Function {
        parameters: Vec<(String, String)>,
        return_type: Option<String>,
        statements: Vec<Statement>,
    },
    Scene {
        name: String,
        statements: Vec<Statement>,
    },
    Struct {
        name: String,
        // TODO: What would be valid defaults? Expression seems to broad.
        // field: type = optional_default,
        fields: Vec<(String, String, Option<Expression>)>,
    }
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
    Unary {
        operator: UnaryOperator,
        rhs: Box<Expression>,
    },
    __Temporary,
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

    Multiply,
    Divide,

    EqualTo,
    NotEqualTo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
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
    Expression(Expression),
}

#[derive(Debug, Clone, Copy)]
/// GLSL Types
pub enum Literal {
    Float(f32),
    Double(f64),
    // FIXME: Type should default to i64, then decide on i/u 32 later (for parsing)
    Int(i32),
    UInt(u32),
    Bool(bool),
}