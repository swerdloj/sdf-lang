// 1 - Lexer(Text) -> Tokens
// 2 - Parser(Tokens) -> AST
// 3 - ??

mod environment;
mod lexer;

fn main() {
    let env = environment::Environment::get();

    // Note that file's existence will be checked above
    let input = std::fs::read_to_string(&env.input_path).expect("Failed to read input file");
    lexer::tokenize_string(input);

    println!("{:?}", env);
}