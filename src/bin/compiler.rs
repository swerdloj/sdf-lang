extern crate sdf_lang;

use sdf_lang::{
    parse, environment, exit_with_message, translate
};

fn main() -> Result<(), std::io::Error> {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path)?;
    println!("{:#?}\n", env);

    // Stores information about structs, scenes, functions, and identifiers
    let mut context = parse::context::Context::new();

    // Print any parse errors, then exit. Otherwise, return AST
    let mut ast = parse::parse(&input).map_err(|e| 
        exit_with_message(format!("Parse Error: {}", e)) 
    ).unwrap();

    translate::validate(&mut ast, &mut context);

    // Write AST to a file
    if env.save_ast {
        env.save_ast(&ast)?;
    }

    // AST -> templates -> GLSL
    let output = translate::translate(&ast, &context);

    env.save_output(output)?;

    Ok(())
}