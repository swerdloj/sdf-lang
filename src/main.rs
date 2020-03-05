// 1 - Lexer(Text) -> Tokens
// 2 - Parser(Tokens) -> AST
// 3 - ??

mod environment;
mod lexer;

use lexer::analyze;

fn main() {
    let env = environment::Environment::get();

    println!("Reading from {:?} ...", &env.input_path);

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path).expect("Failed to read input file");


    let mut lexemes: Vec<analyze::Lexeme> = lexer::analyze::tokenize_string(input).collect();
    analyze::strip(&mut lexemes);

    println!("Lexemes: {:#?}", lexemes);

    // println!("{:?}", env);
}