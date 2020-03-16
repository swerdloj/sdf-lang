// 0 - Input -> Environment/Text
// 1 - Analyze(Text) -> Lexemes
// 2 - Lexer(Lexemes) -> Tokens
// 3 - Parser(Tokens) -> AST
// 4 - AST -> GLSL

extern crate sdf_lang;

use sdf_lang::{
    parse, environment
};

fn main() -> Result<(), std::io::Error> {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path)?;
    println!("{:#?}", env);

    // Print any parse errors, then exit
    let ast = parse::parse(&input).map_err(|e| {
        println!("Parse Error: {}", e);
        std::process::exit(0);
    });

    // Write the AST to a file
    if env.save_ast {
        env.save_ast(&ast.expect("Parse error"))?;
    }

    // TODO: ast -> template -> output GLSL

    Ok(())
}