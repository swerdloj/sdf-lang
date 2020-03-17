extern crate sdf_lang;

use sdf_lang::{
    parse, environment, exit_with_message
};

fn main() -> Result<(), std::io::Error> {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path)?;
    println!("{:#?}", env);

    // Print any parse errors, then exit
    let ast = parse::parse(&input).map_err(|e| 
        exit_with_message(format!("Parse Error: {}", e)) 
    );

    // Write the AST to a file
    if env.save_ast {
        env.save_ast(&ast.expect("Parse error"))?;
    }

    // TODO: ast -> template -> output GLSL

    Ok(())
}