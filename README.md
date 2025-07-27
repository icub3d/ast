# 🧮 AST Calculator

An example calculator meant to illustrate Abstract Syntax Trees (AST, https://en.wikipedia.org/wiki/Abstract_syntax_tree). This is meant to be a first look at what ASTs are and how they can be used.

## Example Usage 

Run the REPL:
```sh
cargo run
```
Sample session:
```
🧮 AST Calculator REPL
Enter mathematical expressions to see the AST and result.
Examples: '3 + 4 * 2', '(5 - 3) * 2.5', '-10 + 5'
Type 'quit' or 'exit' to close.

>>> 3 + 4 * 2
🌳 AST: Add(Float(3.0), Mul(Float(4.0), Float(2.0)))
✅ Result: 11

>>> (5 - 3) * 2.5
🌳 AST: Mul(Sub(Float(5.0), Float(3.0)), Float(2.5))
✅ Result: 5

>>> 8 / 0
🌳 AST: Div(Float(8.0), Float(0.0))
❌ Evaluation error: Division by zero

>>> quit
👋
```

## Documentation
Generate docs with:
```sh
cargo doc --open
```