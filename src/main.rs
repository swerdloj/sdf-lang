// 0 - Input -> Environment/Text
// 1 - Analyze(Text) -> Lexemes
// 2 - Lexer(Lexemes) -> Tokens
// 3 - Parser(Tokens) -> ?? AST?
// 4 - ?? -> GLSL


mod environment;

#[allow(unused)]
mod lexer;

use lexer::tokenize::{Token, tokenize_string};

fn main() {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path).expect("Failed to read input file");
    println!("{:?}", env);

    // Transformations: String -> Lexemes -> Tokens
    let tokens: Vec<Token> = tokenize_string(input);
    print!("Tokens: {:#?}", tokens);
}