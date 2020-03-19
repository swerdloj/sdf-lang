#[macro_use]
extern crate lalrpop_util;

pub mod environment;

// #[allow(unused)]
// mod lex;

pub mod parse;
pub mod translate;

#[allow(unreachable_code)]
pub fn exit_with_message(message: String) {
    #[cfg(not(test))]
    {
        println!("{}", message);
        std::process::exit(0);
    }

    panic!("{}", message);
}