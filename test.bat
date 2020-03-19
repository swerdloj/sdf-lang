set cargoflags=%1

cargo run --bin compiler %cargoflags% -- --input tests/test.sdf --output output/test.glsl --AST

:: type output\ast.txt