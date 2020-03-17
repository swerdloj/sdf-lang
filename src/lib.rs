#[macro_use]
extern crate lalrpop_util;

pub mod environment;

#[allow(unused)]
mod lex;

pub mod parse;
pub mod translate;

pub fn exit_with_message(message: String) {
    println!("{}", message);
    std::process::exit(0);
}