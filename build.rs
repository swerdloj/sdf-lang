extern crate lalrpop;

fn main() {
    // lalrpop files -> Rust files
    lalrpop::process_root().unwrap();
}