pub mod ast;

lalrpop_mod!(pub parser, "/parse/parser.rs");


mod parser_test {
    use crate::parse::parser;

    #[test]
    fn ast_root() {
        let ast = parser::ASTParser::new().parse("
            struct something {
                field1: value1 = default,
                field2: no_default,
            }

            scene main {
                let box: Box {
                    field1: value1,
                    field2: value2,
                };
                let x = 7;
                let y = x;
            }

            fn function() {
                statements;
            }
        ");
        
        println!("{:#?}", ast.unwrap());
    }

    #[test]
    fn expression_statement() {
        let ast = parser::StatementParser::new().parse("
            expression_as_statement;
        ");
        
        println!("{:#?}", ast.unwrap());
    }

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

                let y = 4;
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