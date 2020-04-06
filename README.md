# sdf-lang
**sdf-lang** is a domain-specific language for describing signed distance fields.  

`.sdf` files are transpiled to GLSL for use with a raymarching renderer.

## Compiler
- Run `compiler` with the following arguments
  - `--help` display usage information
  - `--input` to specify the input file path
  - `--output` to specify the output file path
  - `--AST` to write the AST to a text file (if parsed without error)

## Runtime
Run `runtime PATH` where "PATH" is the relative path to the desired `.sdf` file. This will open a window and run the shader.

Within the runtime, the following features are available:
- Hot reloading
  - Press *F5* to reload the current shader
  - Errors in the `.sdf` code will appear in the console
- Timing
  - Time is passed to the shader's `time` uniform in seconds
  - Press *Spacebar* to pause/unpause time
- Window Sizing
  - The window size is passed to the shader's `window_dimensions` uniform
  - Resizing the window will update this uniform

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


### **Syntax**
Syntax is nearly identical to Rust with a few changes/additions

### Shader Type Declaration

All shaders must declare their type on the **first line**:
```Rust
@VERTEX   // Allows gl_Position, gl_VertexID, etc.
@FRAGMENT // Allows gl_FragCoord, out_color, etc.
@COMPUTE  // ...
``` 

### The Apply Operator

A *nestable* function can be applied to a collection of expressions using the *apply* operator like so:
```Rust
let mininum = min <- (a, b, c, d);
```
In this case, the apply operator is equivalent to
```glsl
min(a, min(b, min(c, d)))
```
In order to use this operator, the applied function must be *nestable*, meaning it meets the following conditions:
1. Accepts exactly **2** parameters of the same type
2. Return type is the same as these 2 parameters

This is useful in cases such as expressing the union of complex SDF types or taking the min/max of a collection of expressions.

### **Functions**
Functions in sdf-lang are almost identical to Rust. Parameters may be qualified with `in`, `out`, or `inout`, functioning the same as in GLSL. If no qualifier is given, the default of `in` will be assigned.
```Rust
fn some_function(field1: type1, in field2: type2) -> optional_return_type {
    return field1 + field2;
}
```
Note that implicit returns are not supported by sdf-lang (no final semicolon).

### **Structs**
Structs are somewhat similar to Rust, and are defined as follows:
```Rust
struct StructName {
    field1: type1,
    field2: type2 = default_value,
}
```
Fields with default values do not require the user to specify them in a constructor. Fields without defaults *must* be initialized by the user.

Constructors are structured as follows:
```Rust
let variable: StructName {
    field1: value1,
    // field2: override_default,  <-- This is optional because of the default value
};
```
Note that the constructor is **not** a method.

Methods can be attached to structs like so:
```Rust
impl StructName {
    fn method1(self) {
        self.field = something;
    }

    fn method2(in self, param: some_type) -> some_type {
        return self.field * param;
    }
}
```
If unspecified, the parameter `self` will be treated as `inout`. 

In this case, `in self` is like Rust's `&self`, and `inout self` is like `&mut self`.

Note that all methods must reference `self`.

### **Scenes** -- UNIMPLEMENTED
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
let time: int = 0;
```

The `@out` tag specifies that a variable will be an output of the shader
```Rust
@out
let output_color: vec4 = vec4(0.);
```

Note that such tagged variables are added to the global scope (required by GLSL). This means that no two tagged variables may be declared with the same name.

Furthermore, tagged variables are only accessable within their declared scope, meaning the `.sdf` file will not have any scope pollution.