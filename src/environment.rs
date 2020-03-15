use std::path::{Path, PathBuf};
use std::fs;
use std::io::prelude::Write;


// TODO: Find better name
#[derive(Debug)]
pub struct Environment {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub save_ast: bool,
}

impl Environment {
    pub fn save_ast(&self, ast: &crate::parse::ast::AST) -> Result<(), std::io::Error> {
        let output_path = Path::new("./output");

        if !output_path.exists() {
            fs::create_dir(output_path)?;
        }

        let mut file = fs::File::create(output_path.join("ast.txt"))?;
        file.write_fmt(format_args!("{:#?}", &ast))?;

        Ok(())
    }

    pub fn get() -> Self {
        use std::process::exit;

        // All arguments with associated indices
        // Note that arg[0] is executable's path
        let args: Vec<String> = std::env::args().collect();

        if args.len() == 1 {
            println!("Error: No arguments specified. Run with '--help' to see proper usage.");
            exit(0);
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
                    println!(
                             "sdf-lang compiler usage:\n
                              --help\t\tDisplay this message\n
                              --input PATH\tSpecify the input file path\n
                              --output PATH\tSpecify the output file path. Only specify the file to store in /output/FILE\n
                              --AST\t\tSave the AST to text file in output directory
                              "
                            );
                    exit(0);
                }

                "--input" => {
                    if input.is_some() {
                        println!("Error: Input path is redefined");
                        exit(0);
                    }
                    
                    if let Some(path) = args.get(index + 1) {
                        let p = PathBuf::from(path);
                        if p.exists() {
                            input = Some(p);
                        } else {
                            println!("Error: No such input file exists");
                            exit(0);
                        }
                    } else {
                        println!("Error: No input path specified");
                        exit(0);
                    }

                    // The next index was the path, so skip it
                    index += 1;
                }
                
                "--output" => {
                    if output.is_some() {
                        println!("Error: Output path is redefined");
                        exit(0);
                    }
                    
                    if let Some(path) = args.get(index + 1) {
                        let p = PathBuf::from(path);
                        if p.exists() {
                            // TODO: Overwrite existing??
                            unimplemented!();
                        } else {
                            // TODO: Create path if needed
                            output = Some(p);
                        }
                    } else {
                        println!("Error: No output path specified");
                        exit(0);
                    }
                    
                    // The next index was the path, so skip it
                    index += 1;
                }

                "--AST" => {
                    save_ast = true;
                }

                // Unknown
                x => {
                    println!("Error: Unknown argument '{}'. Run with '--help' to see proper usage.", x);
                    exit(0);
                }
            }

            index += 1;

            if index >= num_args {
                break;
            }
        }

        if input.is_none() {
            println!("Error: Input path was not specified. Run with '--help' to see proper usage.");
            exit(0);
        }
        
        if output.is_none() {
            println!("Error: Output path was not specified. Run with '--help' to see proper usage.");
            exit(0);
        }

        Environment {
            input_path: input.unwrap(),
            output_path: output.unwrap(),
            save_ast,
        }
    }
}