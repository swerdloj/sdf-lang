extern crate sdf_lang;

use sdf_lang::{
    parse, environment, exit_with_message, translate
};

fn main() -> Result<(), std::io::Error> {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path)?;
    println!("{:#?}", env);

    // Stores information about structs, scenes, functions, and identifiers
    let mut context = sdf_lang::translate::Context::new();

    // Print any parse errors, then exit
    let ast = parse::parse(&input, &mut context).map_err(|e| 
        exit_with_message(format!("Parse Error: {}", e)) 
    ).unwrap();

    // Write the AST to a file
    if env.save_ast {
        env.save_ast(&ast)?;
    }

    // ast -> template -> output GLSL
    let output = translate::transpile(&ast, &context);

    env.save_output(output)?;

    Ok(())
}