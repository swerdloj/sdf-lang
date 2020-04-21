**The following items are not yet implemented**

## Language
### Multiple Files
- Implement a module system with namespacing and public/private
### Empty Structs
- GLSL does not allow structs without any fields
  - Could get around this by adding a dummy field when parsing the .sdf file
### Enums
- Implement enums for `match`
### Default Function Parameter Values
- Consider allowing defaults for function parameters like Python

## **Parser**
### Types
- `vec` type casting (same rules as normal types)
- `mat` types
- `sampler` types
- Multi dimensional arrays
### Expressions
- `if`/`else` as expressions with return types
- `match`
- Constructors as expressions
### Statements
- `for` loop  
### Structs
- Member chaining

## **Translator**
- Type inferrence beyond initial assignment
- Uniform defaults
- Scenes

## **Runtime**
- Access to user-declared uniforms
- Console
  - Allow users to peek/set uniforms via the console