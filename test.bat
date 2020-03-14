set cargoflags=%1

cargo run %cargoflags% -- --input tests/simple.sdf --output tests/simple.glsl --AST

type output\ast.txt