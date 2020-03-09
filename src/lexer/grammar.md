### See https://github.com/rust-lang/wg-grammar/tree/master for grammar reference

# sdf-lang Grammar:
## + : One or more
## ? : Zero or one
## * : Zero or more

Program -> (Scene | Function)+

Scene -> Keyword::Scene Ident StatementBlock
Function -> Keyword::Fn Ident ParameterList (Arrow Ident)? StatementBlock

// TODO: Commas
ParameterList -> OpenParenthesis (Ident Colon Ident)* StatementBlock

StatementBlock -> OpenBrace (Statement | StatementBlock | Expression)* CloseBrace

TODO: Statements
Statement -> Expression
            | Semicolon
            | Keyword::Let Ident (Colon Ident)? (Assign Expression)?

TODO: Expressions
Expression -> Literal
            | IfExpr
            | Expression BinaryOperator Expression
            | OpenParenthesis Expression CloseParenthesis

// TODO: LHS and RHS of BooleanOperator could also be boolean expressions
IfExpr -> Keyword::If ((Ident | Literal) BooleanOperator (Ident | Literal))+ (Keyword::Else Keyword::If?)?
        | Keyword::If (Ident | Literal) (Keyword::Else Keyword::If?)?

Keyword -> Let
         | If
         | Else
         | Scene
         | Enum
         | Struct

Literal -> Float | Int | Bool

BinaryAssignOperator -> AddAssign
                      | SubtractAssign
                      | DivideAssign
                      | MultiplyAssign

BinaryOperator -> Add
                | Subtract
                | Divide
                | Multiply
                | Modulo

BooleanOperator -> GreaterThan
                 | LessThan
                 | GreaterThanOrEqualTo
                 | LessThanOrEqualTo
                 | EqualTo
                 | NotEqualTo
                 | And
                 | Or

UnaryOperator -> Negate | Not