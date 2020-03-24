**The following items are not yet implemented**

## Other
- GLSL does not allow structs without any fields
  - Could get around this by adding a dummy field when parsing the .sdf file

## **Parser**
### Error locations
- Span via @L/R lalrpop bindings
### Types
- `mat` types
- `sampler` types
- Arrays
### Expressions
- `if`/`else` as expressions with return types
- `match`
- Constructors
### Statements
- `for` loop  
- `do while` loop
### Structs
- member function calling (currently broken for chaining)

## **Translator**
- Type inferrence beyond initial assignment
- Uniform defaults
- Scenes

## **Runtime**
- Everything