// 0 - Input -> Environment/Text
// 1 - Analyze(Text) -> Lexemes
// 2 - Lexer(Lexemes) -> Tokens
// 3 - Parser(Tokens) -> ?? (AST?)
// 4 - ?? -> GLSL

#[macro_use]
extern crate lalrpop_util;

mod environment;
mod parse;

#[allow(unused)]
mod lex;


fn main() {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path).expect("Failed to read input file");
    println!("{:?}", env);

    let ast = parse::parser::StatementParser::new().parse("let x = 1;");

    println!("{:#?}", ast);
}