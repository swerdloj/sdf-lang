// 0 - Input -> Environment/Text
// 1 - Analyze(Text) -> Lexemes
// 2 - Lexer(Lexemes) -> Tokens
// 3 - Parser(Tokens) -> ?? (AST?)
// 4 - ?? -> GLSL

#[macro_use]
extern crate lalrpop_util;

mod environment;
#[allow(unused)]
mod parse;
#[allow(unused)]
mod lex;


use std::fs;
use std::io::prelude::Write;

fn main() {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = fs::read_to_string(&env.input_path).expect("Failed to read input file");
    println!("---ENVIRONMENT---\n{:#?}\n\n", env);

    // TODO: What about comments? Should they just be stripped before parsing?
    let ast = parse::parser::ASTParser::new().parse(&input);

    // Write the AST to a file
    if env.save_ast {
        fs::create_dir("output/");
        let mut file = fs::File::create("output/ast.txt").unwrap();
        file.write_fmt(format_args!("{:#?}", &ast));
    }

    // TODO: ast -> template -> output GLSL
}