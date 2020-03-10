### See https://github.com/rust-lang/wg-grammar/tree/master for grammar reference

# sdf-lang Grammar:
## + : One or more
## ? : Zero or one
## * : Zero or more

Program -> (Scene | Function)+

Scene -> Scene Ident StatementBlock
Function -> Fn Ident ParameterList (Arrow Ident)? StatementBlock

TODO: Fix commas
ParameterList -> OpenParenthesis (Ident Colon Ident Comma?)* StatementBlock

StatementBlock -> OpenBrace (Statement | StatementBlock | Expression)* CloseBrace

TODO: Constructors?
Statement -> Expression
            | Let Ident (Colon Ident)? (Assign Expression)?
            | Semicolon

TODO: Dot operator?
Expression -> Literal
            | IfExpr
            | UnaryOperator Expression
            | Expression (BinaryOperator | BinaryAssign) Expression
            | OpenParenthesis Expression CloseParenthesis

IfExpr -> If (BoolExpr | Ident | Literal) (Else If?)?

BoolExpr -> (Ident | Literal | BoolExpr) BooleanOperator (Ident | Literal | BoolExpr)

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