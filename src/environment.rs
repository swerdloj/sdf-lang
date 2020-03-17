use std::path::{Path, PathBuf};
use std::fs;
use std::io::prelude::Write;

use crate::exit_with_message;

// TODO: Find better name
#[derive(Debug)]
pub struct Environment {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub save_ast: bool,
}

// TODO: save_output/ast have weird formatting on their path `println!`s
impl Environment {
    pub fn save_output(&self, output_glsl: String) -> Result<(), std::io::Error> {       
        if !self.output_path.parent().expect("Cannot write to executable directory").exists() {
            fs::create_dir(self.output_path.parent().unwrap())?;
        }

        let mut file = fs::File::create(&self.output_path)?;
        file.write_fmt(format_args!("{}", output_glsl))?;

        println!("GLSL saved to ./{:?}", &self.output_path);

        Ok(())
    }

    pub fn save_ast(&self, ast: &crate::parse::ast::AST) -> Result<(), std::io::Error> {
        let output_path = Path::new("./output");

        if !output_path.exists() {
            fs::create_dir(output_path)?;
        }

        let ast_path = output_path.join("ast.txt");

        let mut file = fs::File::create(&ast_path)?;
        file.write_fmt(format_args!("{:#?}", &ast))?;

        println!("AST saved to {:?}", &ast_path);

        Ok(())
    }

    pub fn get() -> Self {
        // All arguments with associated indices
        // Note that arg[0] is executable's path
        let args: Vec<String> = std::env::args().collect();

        if args.len() == 1 {
            exit_with_message(format!("Error: No arguments specified. Run with '--help' to see proper usage."));
        }

        // DEFAULTS
        let mut input: Option<PathBuf> = None;
        let mut output: Option<PathBuf> = None;
        let mut save_ast = false;

        let mut index = 1;
        let num_args = args.len();
        loop {
            match args[index].as_str() {
                "--help" => {
                    exit_with_message(format!(
                             "sdf-lang compiler usage:\n
                              --help\t\tDisplay this message\n
                              --input PATH\tSpecify the input file path\n
                              --output PATH\tSpecify the output file path. Only specify the file to store in /output/FILE\n
                              --AST\t\tSave the AST to text file in output directory
                              "
                            )
                        );
                }

                "--input" => {
                    if input.is_some() {
                        exit_with_message(format!("Error: Input path is redefined"));
                    }
                    
                    if let Some(path) = args.get(index + 1) {
                        let p = PathBuf::from(path);
                        if p.exists() {
                            input = Some(p);
                        } else {
                            exit_with_message(format!("Error: No such input file exists"));
                        }
                    } else {
                        exit_with_message(format!("Error: No input path specified"));
                    }

                    // The next index was the path, so skip it
                    index += 1;
                }
                
                "--output" => {
                    if output.is_some() {
                        exit_with_message(format!("Error: Output path is redefined"));
                    }
                    
                    if let Some(path) = args.get(index + 1) {
                        let p = PathBuf::from(path);
                        if p.exists() {
                            // TODO: Overwrite existing??
                            println!("Overwriting previous...");
                            output = Some(p);
                        } else {
                            // TODO: Create path if needed
                            output = Some(p);
                        }
                    } else {
                        exit_with_message(format!("Error: No output path specified"));
                    }
                    
                    // The next index was the path, so skip it
                    index += 1;
                }

                "--AST" => {
                    save_ast = true;
                }

                // Unknown
                x => {
                    exit_with_message(format!("Error: Unknown argument '{}'. Run with '--help' to see proper usage.", x));
                }
            }

            index += 1;

            if index >= num_args {
                break;
            }
        }

        if input.is_none() {
            exit_with_message(format!("Error: Input path was not specified. Run with '--help' to see proper usage."));
        }
        
        if output.is_none() {
            exit_with_message(format!("Error: Output path was not specified. Run with '--help' to see proper usage."));
        }

        Environment {
            input_path: input.unwrap(),
            output_path: output.unwrap(),
            save_ast,
        }
    }
}