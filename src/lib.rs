#[macro_use]
extern crate lalrpop_util;

pub mod environment;
pub mod parse;
pub mod translate;

// #[cfg(runtime)]
pub mod runtime;

#[macro_export]
macro_rules! exit {
    ($m:expr) => {{
        println!("{}", $m);
        std::process::exit(0);
    }};
}