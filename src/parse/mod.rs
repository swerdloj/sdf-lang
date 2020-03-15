pub mod ast;

lalrpop_mod!(pub parser, "/parse/parser.rs");


mod parser_test {
    use crate::parse::parser;

    #[test]
    fn comments() {
        let ast = parser::ASTParser::new().parse("
            /*
            struct commented_out {
                field1: value1 = default,
                field2: no_default,
            }
            stuff in comment here
            */

            // Comment here

            scene main {
                let box: Box {
                    field1: value1,
                    field2: value2,
                };
                let x = 7;
                let y = x; // Another comment
            }
        ");
        
        println!("{:#?}", ast.unwrap());
    }

    #[test]
    fn ast_root_with_struct_with_function_with_scene_with_constructor() {
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
}