// AST enums, types, and structs

/// AST root
pub type AST = Vec<Item>;
pub type Span = (usize, usize);

#[derive(Debug)]
pub enum Item {
    Constant(ConstDeclaration),
    Function {
        name: String,
        // ( qualifier, identifier, type )
        parameters: Vec<(Option<FuncParamQualifier>, String, TypeSpecifier)>,
        // If not specified, return type will be "void"
        return_type: TypeSpecifier,
        statements: Vec<Statement>,
    },
    Scene {
        name: String,
        statements: Vec<Statement>,
    },
    Struct {
        name: String,
        // "field: type = optional_default,"
        fields: Vec<(String, TypeSpecifier, Option<Expression>)>,
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
    Features {
        features: Vec<String>,
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
    ArrayConstructor {
        expressions: Vec<Box<Expression>>,
        ty: String,
    },
    Identifier(String),
    Binary {
        lhs: Box<Expression>,
        operator: BinaryOperator,
        rhs: Box<Expression>,
        ty: String,
    },
    Unary {
        operator: UnaryOperator,
        expr: Box<Expression>,
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
    Index(Box<Expression>),
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
    pub ty: TypeSpecifier,
    // TODO: When constant expressions are implemented, this must be constant-checked
    pub value: SpannedExpression,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        tag: Option<Tag>,
        ident: String,
        ty: Option<TypeSpecifier>,
        expression: Option<SpannedExpression>,
    },
    LetConstructor {
        ident: String,
        constructor: Constructor,
    },
    Constant(ConstDeclaration),
    Assignment {
        lhs: SpannedExpression,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeSpecifier {
    Identifier(String),
    Array {
        ty: String,
        size: u32,
    }
}

impl TypeSpecifier {
    pub fn from_ident(id: &str) -> Self {
        TypeSpecifier::Identifier(id.to_owned())
    }

    pub fn type_name(&self) -> &str {
        match self {
            TypeSpecifier::Identifier(ident) => ident,
            TypeSpecifier::Array { ty, size: _ } => ty,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            TypeSpecifier::Identifier(ident) => ident.clone(),
            TypeSpecifier::Array { ty, size } => format!("{}[{}]", ty, size),
        }
    }
}

impl std::fmt::Display for TypeSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
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