// 1 - Lexer(Text) -> Tokens
// 2 - Parser(Tokens) -> AST
// 3 - ??

extern crate regex;

mod environment;
mod lexer;

fn main() {
    let env = environment::Environment::get();

    println!("Reading from {:?} ...", &env.input_path);

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path).expect("Failed to read input file");


    let lexemes: Vec<lexer::analyze::Lexeme> = lexer::analyze::tokenize_string(input).collect();
    println!("Lexemes: {:?}", lexemes);

    println!("{:?}", env);
}