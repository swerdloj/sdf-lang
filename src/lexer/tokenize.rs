// TODO: Account for GLSL keywords so user cannot use them for variable names, etc.

enum Keyword {
    Let,
    If,
    Scene,
    Enum,
    Struct,
}

enum Primitive {
    Vec2,
    Vec3,
    Vec4,
    Int,
    Float,
    Matrix,
}

enum Comparator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    EqualTo,
    NotEqualTo,
    Not,
}

enum Operator {
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulo,
}