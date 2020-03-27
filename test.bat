set cargoflags=%1

::cargo run --bin compiler %cargoflags% -- --input tests/fragment.sdf --output output/test.glsl --AST
cargo run --bin runtime %cargoflags%