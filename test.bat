@echo off

set target=fragment

set which=%1

if not defined which (
    echo Please specifiy which test
    goto end
)
    

set cargoflags=%2

if not defined cargoflags (
    set cargoflags=
)


if "%which%" == "compiler" (
    cargo run --bin compiler %cargoflags% -- --input ./tests/%target%.sdf --output ./output/%target%.glsl --AST
    goto end
)

if "%which%" == "runtime" (
    cargo run --bin runtime %cargoflags% -- tests/%target%.sdf
    goto end
)

echo Unknown setting "%which%"

:end