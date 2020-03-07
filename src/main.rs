// 0 - Input -> Environment/Text
// 1 - Analyze(Text) -> Lexemes
// 2 - Lexer(Lexemes) -> Tokens
// 3 - Parser(Tokens) -> ?? AST?
// 4 - ?? -> GLSL


mod environment;

#[allow(unused)]
mod lexer;

use lexer::{analyze, tokenize};

fn main() {
    let env = environment::Environment::get();
    println!("Reading from {:?} ...", &env.input_path);

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path).expect("Failed to read input file");
    println!("{:?}", env);


    let mut lexemes: Vec<analyze::Lexeme> = analyze::analyze_string(input).collect();
    analyze::strip(&mut lexemes);
    println!("Lexemes: {:#?}", lexemes);

    let tokens: Vec<tokenize::Token> = tokenize::tokenize_lexemes(lexemes).collect();
    print!("Tokens: {:#?}", tokens);
}