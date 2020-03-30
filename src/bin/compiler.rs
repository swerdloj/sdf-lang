extern crate sdf_lang;

use sdf_lang::{
    parse, environment, translate, exit
};

fn main() -> Result<(), std::io::Error> {
    let env = environment::Environment::get();
    println!("{:#?}\n", env);
    
    // Note that file's existence will be checked already
    let input = parse::Input::from_path(&env.input_path)?;
    
    // Print any parse errors, then exit. Otherwise, return AST
    let mut ast = parse::parse(&input).map_err(|e| 
        exit!(format!("Parse Error: {}", e)) 
    ).unwrap();
    
    // Stores information about structs, scenes, functions, and identifiers
    let context = translate::validate(&mut ast, &input).map_err(|e| 
        exit!(format!("Semantic Error: {}", e)) 
    ).unwrap();

    // Write AST to a file
    if env.save_ast {
        env.save_ast(&ast)?;
    }

    // AST -> templates -> GLSL
    let output = translate::translate(&ast, &context);

    env.save_output(output)?;

    Ok(())
}