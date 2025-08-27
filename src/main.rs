use ast::{evaluate, parse};
use std::io::{self, Write};

/// Main function - Entry point for the interactive REPL
///
/// The REPL continues until the user types "quit" or "exit".
fn main() {
    println!("🧮 AST Calculator REPL");
    println!("Enter mathematical expressions to see the AST and result.");
    println!("Examples: '3 + 4 * 2', '(5 - 3) * 2.5', '-10 + 5'");
    println!("Type 'quit' or 'exit' to close.\n");

    loop {
        print!(">>> ");
        if io::stdout().flush().is_err() {
            // Stop the REPL if we can't write to stdout
            break;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached (e.g., piped input finished)
                break;
            }
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
                match parse(input) {
                    Ok(ast) => {
                        println!("🌳 AST: {:?}", ast);
                        match evaluate(&ast) {
                            Ok(result) => println!("✅ Result: {}", result),
                            Err(error) => println!("❌ Evaluation error: {}", error),
                        }
                    }
                    Err(error) => {
                        println!("❌ Error: {}", error);
                    }
                }
                println!();
            }
            Err(error) => {
                println!("❌ Error reading input: {}", error);
                break;
            }
        }
    }
}
