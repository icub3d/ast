use ast::{evaluate, parse_expression};
use std::io::{self, Write};

/// Main function - Entry point for the interactive REPL
///
/// This function provides a Read-Eval-Print Loop (REPL) interface that allows users
/// to interactively test mathematical expressions. For each input:
/// 1. Parses the expression into an AST
/// 2. Displays the AST structure
/// 3. Evaluates the AST to get the numeric result
/// 4. Shows any errors that occur during parsing or evaluation
///
/// The REPL continues until the user types "quit" or "exit".
fn main() {
    println!("🧮 AST Calculator REPL");
    println!("Enter mathematical expressions to see the AST and result.");
    println!("Examples: '3 + 4 * 2', '(5 - 3) * 2.5', '-10 + 5'");
    println!("Type 'quit' or 'exit' to close.\n");

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                // Handle exit commands
                if input.is_empty() {
                    continue;
                }
                if input == "quit" || input == "exit" {
                    println!("👋");
                    break;
                }

                // Parse and evaluate the expression
                match parse_expression(input) {
                    Ok((remaining, ast)) => {
                        // Show the Abstract Syntax Tree for educational purposes
                        println!("🌳 AST: {:?}", ast);

                        // Evaluate the AST to get the numeric result
                        match evaluate(&ast) {
                            Ok(result) => println!("✅ result: {}", result),
                            Err(error) => println!("❌ evaluating: {}", error),
                        }

                        // Warn if there's unparsed input (indicates syntax error)
                        if !remaining.trim().is_empty() {
                            println!("⚠️ unparsed input: '{}'", remaining);
                        }
                    }
                    Err(error) => {
                        println!("🚫 parsing: {:?}", error);
                    }
                }
                println!(); // Add spacing between inputs
            }
            Err(error) => {
                println!("❌ reading input: {}", error);
                break;
            }
        }
    }
}
