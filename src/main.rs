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


fn main() {
    let env = environment::Environment::get();

    // Note that file's existence will be checked already
    let input = std::fs::read_to_string(&env.input_path).expect("Failed to read input file");
    println!("ENVIRONMENT:\n{:#?}", env);

    // TODO: What about comments? Should they just be stripped before parsing?
    let ast = parse::parser::StatementParser::new().parse(&input);
    println!("AST:\n{:#?}", ast);

    // TODO: ast -> template -> output GLSL
}


mod parser_test {
    use crate::parse::parser;

    #[test]
    fn let_statement() {
        let ast = parser::StatementParser::new().parse("
            let x = 1;
        ");
        
        println!("{:#?}", ast.unwrap());
    }

    #[test]
    fn let_constructor() {
        let ast = parser::StatementParser::new().parse("
            let x: y { 
                f1: 1, 
                f2: 2,
            };
        ");
        
        println!("{:#?}", ast.unwrap());
    }

    #[test]
    fn scene_with_constructor() {
        let ast = parser::SceneParser::new().parse("
            scene main {
                let x = 7;
                
                let cube: Box {
                    field_name: x,
                };
            }
        ");
        
        println!("{:#?}", ast.unwrap());
    }

    #[test]
    fn function_with_params_and_return_type() {
        let ast = parser::FunctionParser::new().parse("
            fn name(param1: type1, param2: type2) -> return_type {
                let x = 12;
            }
        ");
        
        println!("{:#?}", ast.unwrap());
    }

    #[test]
    fn void_function_no_params() {
        let ast = parser::FunctionParser::new().parse("
            fn name() {
                let x = 12;
            }
        ");
        
        println!("{:#?}", ast.unwrap());
    }
}