**The following items are not yet implemented**

## Other
### Empty Structs
- GLSL does not allow structs without any fields
  - Could get around this by adding a dummy field when parsing the .sdf file
### Enums
- Implement enums for `match`

## **Parser**
### Error locations
- Span via @L/R lalrpop bindings
### Types
- `vec` type casting (same rules as normal types)
- `mat` types
- `sampler` types
- Arrays
### Expressions
- `if`/`else` as expressions with return types
- `match`
- Constructors as expressions
### Statements
- `for` loop  
- `do while` loop
### Structs
- member chaining

## **Translator**
- Type inferrence beyond initial assignment
- Uniform defaults
- Scenes

## **Runtime**
- Everything