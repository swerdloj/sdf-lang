@echo off

:: Which program to test
set which=%1

if not defined which (
    echo Please specifiy which test
    goto end
)
    

:: Build flag (like '--release')
set cargoflags=%2

if "%cargoflags%" == "_" (
    set cargoflags=
)

if not defined cargoflags (
    set cargoflags=
)


:: Which '.sdf' file to use
set target=%3

if not defined target (
    set target=fragment
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