// AST enums, types, and structs

/// AST root
pub type AST = Vec<Item>;

#[derive(Debug)]
pub enum Item {
    Function {
        name: String,
        parameters: Vec<(String, String)>,
        // If not specified, return type will be "void"
        return_type: String,
        statements: Vec<Statement>,
    },
    Scene {
        name: String,
        statements: Vec<Statement>,
    },
    Struct {
        name: String,
        // "field: type = optional_default,"
        fields: Vec<(String, String, Option<Expression>)>,
    },
    Implementation {
        struct_name: String,
        // Contains only functions with references to `self`
        functions: Vec<Item>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary {
        lhs: Box<Expression>,
        operator: BinaryOperator,
        rhs: Box<Expression>,
        ty: String,
    },
    Unary {
        operator: UnaryOperator,
        rhs: Box<Expression>,
        ty: String,
    },
    FunctionCall {
        name: String,
        parameters: Vec<Expression>,
        ty: String,
    },
    If {
        expression: Box<Expression>,
        if_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        else_if_block: Option<Box<Expression>>,
        ty: String,
    },
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub ty: String,
    pub fields: Vec<(String, Expression)>,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,

    Multiply,
    Divide,

    Cast,
    Member,

    EqualTo,
    NotEqualTo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        ident: String,
        tag: Option<Tag>,
        ty: Option<String>,
        expression: Option<Expression>,
    },
    LetConstructor {
        ident: String,
        constructor: Constructor,
    },
    Assignment {
        ident: String,
        op: AssignmentOperator,
        expression: Expression,
    },
    Return {
        expression: Option<Expression>,
    },
    For {
        loop_var: String,
        from: Expression,
        to: Expression,
        block: Vec<Statement>,
    },
    While {
        condition: Expression,
        block: Vec<Statement>,
    },
    Continue,
    Break,
    Expression(Expression),
}

#[derive(Debug, Clone)]
/// A tag identifies variables which require CPU initialization or modification
pub enum Tag {
    Uniform,
    Texture2D,
    Out,
    // TODO: What else would be needed?
}

#[derive(Debug, Clone)]
/// GLSL Types
pub enum Literal {
    Float(f32),
    Double(f64),
    // FIXME: Type should default to i64, then decide on i/u 32 later (for parsing)
    Int(i32),
    UInt(u32),
    Bool(bool),
}