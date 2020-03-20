The following items are not yet implemented

## **Parser**
### Error locations
- Consider just creating a static span and updating via @L/R lalrpop bindings
  - Interface with this in `exit_with_message` to display the file, line, and column
### Types
- `vec` types
- `mat` types
- `sampler` types
- Arrays
### Expressions
- `if`/`else`
- `match`
- Constructors
### Statements
- `for` loop  
- `while` loop
- `do while` loop
- `break`
- `continue`
### Structs
- `impl` functions

## **Translator**
- Type inferrence
- Type casting
- Scenes

## **Runtime**
- Everything