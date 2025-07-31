use ast::{evaluate, parse_expression};
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
        io::stdout().flush().unwrap();

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
                match parse_expression(input) {
                    Ok((remaining, ast)) => {
                        println!("🌳 AST: {:?}", ast);

                        match evaluate(&ast) {
                            Ok(result) => println!("✅ result: {}", result),
                            Err(error) => println!("❌ evaluating: {}", error),
                        }

                        if !remaining.trim().is_empty() {
                            println!("⚠️ unparsed input: '{}'", remaining);
                        }
                    }
                    Err(error) => {
                        println!("🚫 parsing: {:?}", error);
                    }
                }
                println!(); 
            }
            Err(error) => {
                println!("❌ reading input: {}", error);
                break;
            }
        }
    }
}
