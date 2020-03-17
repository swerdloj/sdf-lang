extern crate lalrpop;

fn main() {
    println!("cargo:rerun-if-changed=src/parse/parser.lalrpop");

    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .process_file("src/parse/parser.lalrpop")
        .unwrap();
}