// AST enums, types, and structs

/// AST root
pub type AST = Vec<Item>;
pub type Span = (usize, usize);

#[derive(Debug)]
pub enum Item {
    Constant(ConstDeclaration),
    Function {
        name: String,
        parameters: Vec<(Option<FuncParamQualifier>, String, String)>,
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
    Import {
        file_name: String,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum FuncParamQualifier {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone)]
pub struct SpannedExpression {
    pub expression: Expression,
    pub span: (usize, usize),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Parenthesized(Box<Expression>),
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
    FunctionApply(FunctionApply),
    FunctionCall(FunctionCall),
    Member(Member),
    If {
        expression: Box<Expression>,
        if_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
        else_if_block: Option<Box<Expression>>,
        ty: String,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionApply {
    pub name: String,
    pub func_parameters: usize,
    pub parameters: Vec<Expression>,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub parameters: Vec<Expression>,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub fields: Vec<(String, SpannedExpression)>,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,

    Multiply,
    Divide,

    Cast,

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
pub struct ConstDeclaration {
    pub ident: String,
    pub ty: String,
    // TODO: When constant expressions are implemented, this must be constant-checked
    pub value: SpannedExpression,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        tag: Option<Tag>,
        ident: String,
        ty: Option<String>,
        expression: Option<SpannedExpression>,
    },
    LetConstructor {
        ident: String,
        constructor: Constructor,
    },
    Constant(ConstDeclaration),
    Assignment {
        lhs: IdentOrMember,
        op: AssignmentOperator,
        expression: SpannedExpression,
    },
    Return {
        expression: Option<SpannedExpression>,
    },
    For {
        loop_var: String,
        from: SpannedExpression,
        to: SpannedExpression,
        block: Vec<Statement>,
    },
    While {
        condition: SpannedExpression,
        block: Vec<Statement>,
        do_while: bool,
    },
    Continue(Span),
    Break(Span),
    // FIXME: Parser cannot put a SpannedExpression here for some reason
    Expression {
        expression: Expression,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum IdentOrMember {
    Ident(String),
    Member(Member),
}

#[derive(Debug, Clone)]
pub struct Member {
    // ident.function().ident.function() etc.
    pub path: Vec<IdentOrFunction>,
    // The final item's type
    pub ty: String,
}
#[derive(Debug, Clone)]
pub enum IdentOrFunction {
    Ident(String),
    Function(FunctionCall),
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