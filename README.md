# sdf-lang
**sdf-lang** is a domain-specific language for describing signed distance fields.  

`.sdf` files are transpiled to GLSL for use with a raymarching renderer.

## Compiler
- Run `sdf-lang` with the following arguments
  - `--help` display usage information
  - `--input` to specify the input file path
  - `--output` to specify the output file path
  - `--AST` to write the AST to a text file (if parsed without error)

## Language
sdf-lang has syntax inspired by Rust and is compiled to GLSL. The langauge may be extended to compile to various shader types in the future, but it currently targets fragment shaders exclusively.

Therefore, all code in a `.sdf` will be run **per-pixel** just like a typical fragment shader. There are some exceptions:
- Textures become uniforms. They are instantiated on the CPU.
- Variables tagged with `@uniform` *must* be initialized with a constant value. They will then be left to the user to implement on the CPU.

## Language Structure
`.sdf` files are composed of the following items:
- Functions
- Structs
- Scenes
- (TODO:) Enums 

TODO: in -> copy
      inout -> &mut

### **Functions**
Functions in sdf-lang are identical to Rust:
```Rust
fn some_function(field1: type1, field2: type2) -> optional_return_type {
    return field1 + field2;
}
```
Note that implicit returns are not supported by sdf-lang (no final semicolon).

### **Structs**
Structs are somewhat similar to Rust, and are defined as follows:
```Rust
struct struct_name {
  field1: type1,
  field2: type2 = default_value,
}
```
Fields with default values do not require the user to specify them in a constructor. Fields without defaults *must* be initialized by the user.

Constructors are structured as follows:
```Rust
let variable: struct_name {
    field1: value1,
    // field2: override_default,  <-- This is optional because of the default value
};
```
Note that the constructor is **not** a method.

TODO:
Methods can be attached to structs like so:
```Rust
impl struct_name {
  fn ...
}
```

### **Scenes**
The purpose of sdf-lang is to construct signed distance fields. Scenes interact with a signed distance field's geometry.

Scenes are rendered via raymarching and therefore have access to the rays as they are being cast:
```Rust
scene scene_name {
    let cube: Box {
      length: 1,
      width: 1,
      height: 1,
    };

    cube.xz *= rotate(time);
}
```
Scenes implicitly take in the `vec3` corresponding to the current pixel being rendered. Scenes implicitly return a struct containing information such as which object was hit the ray and the distance to that object.

Scenes have access to the current `point`, `time` since startup in ms, and scene `distance`.

Structs and functions may be accessed like normal from within a scene.

Note that vector types allow for swizzling just like GLSL.



### **Tags**
Tags are denoted by the `@` symbol. 

The most common tag is `@uniform` which specifies that a variable will be modified via the CPU. All such variables are required to have their type specified with an initial value.
```Rust
@uniform
let x: int = 7;
```

Furthermore, such `@uniform` variables are added to the globabl scope.