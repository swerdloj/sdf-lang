### See https://github.com/rust-lang/wg-grammar/tree/master for grammar reference

# sdf-lang Grammar:
## + : One or more
## ? : Zero or one
## * : Zero or more

Program -> (Scene | Function)+

Scene -> Keyword::Scene Ident StatementBlock
Function -> Keyword::Fn Ident ParameterList (Arrow Ident)? StatementBlock

Parameters -> OpenParenthesis (Ident Colon Ident)* StatementBlock

StatementBlock -> OpenBrace (Statement | StatementBlock | Expression)* CloseBrace

TODO: Statements
Statement ->

TODO: Expressions
Expression -> 

TODO: Boolean Expressions
BooleanExpresion -> 

Keyword -> Let
         | If
         | Scene
         | Enum
         | Struct

Literal -> Float | Int

Delimiter -> Comma
           | Colon
           | Semicolon
           | ParenthesisOpen
           | ParenthesisClose
           | BraceOpen
           | BraceClose
           | BracketOpen
           | BracketClose

BinaryOperator -> Assign
                | AddAssign
                | SubtractAssign
                | DivideAssign
                | MultiplyAssign
                | Add
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