set cargoflags=%1

cargo run --bin compiler %cargoflags% -- --input tests/simple.sdf --output output/simple.glsl --AST

:: type output\ast.txt