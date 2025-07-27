//! # AST Calculator
//!
//! A mathematical expression parser and evaluator that builds Abstract Syntax Trees (ASTs).
//! This calculator demonstrates recursive descent parsing using the `nom` parser combinator library.
//!
//! ## Features
//! - Parses mathematical expressions with proper operator precedence
//! - Supports addition (+), subtraction (-), multiplication (*), and division (/)
//! - Handles parentheses for grouping operations
//! - Works with floating-point numbers (including decimals and negative numbers)
//! - Provides detailed error handling for invalid expressions and division by zero
//! - Interactive REPL (Read-Eval-Print Loop) for testing expressions
//!
//! ## Example Usage
//! ```
//! use ast::{parse_expression, evaluate};
//!
//! // Parse and evaluate: 3 + 4 * 2
//! let (_, ast) = parse_expression("3 + 4 * 2").unwrap();
//! let result = evaluate(&ast).unwrap();
//! assert_eq!(result, 11.0);
//!
//! // The AST structure is: Add(Float(3.0), Mul(Float(4.0), Float(2.0)))
//! ```

use nom::{
    IResult,
    character::complete::{char, multispace0},
    number::complete::double,
};
use thiserror::Error;

/// Errors that can occur during expression evaluation
#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Division by zero")]
    DivisionByZero,
}

/// Abstract Syntax Tree representation of mathematical expressions
///
/// Each variant represents a different type of mathematical operation or value:
/// - `Float`: A numeric literal (e.g., 3.14, -5.0, 42)
/// - `Add`: Addition operation (left + right)
/// - `Sub`: Subtraction operation (left - right)
/// - `Mul`: Multiplication operation (left * right)
/// - `Div`: Division operation (left / right)
///
/// Operations are stored as boxed expressions to allow for nested structures.
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Float(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

/// Parse a number into an Expr::Float (supports decimals and negative numbers)
///
/// This function handles both positive and negative floating-point numbers.
/// Examples: "42", "-3.14", "0.5", "-0.25"
///
/// # Arguments
/// * `input` - The string slice to parse
///
/// # Returns
/// * `IResult<&str, Expr>` - Parser result with remaining input and parsed expression
///
/// # Examples
/// ```
/// use ast::parse_number;
///
/// // Parse positive number
/// let (_, expr) = parse_number("42").unwrap();
/// assert_eq!(expr, ast::Expr::Float(42.0));
///
/// // Parse negative decimal
/// let (_, expr) = parse_number("-3.14").unwrap();
/// assert_eq!(expr, ast::Expr::Float(-3.14));
/// ```
pub fn parse_number(input: &str) -> IResult<&str, Expr> {
    // nom's double parser can handle negative numbers directly
    let (input, num) = double(input)?;
    Ok((input, Expr::Float(num)))
}

/// Parse an expression wrapped in parentheses
///
/// This function handles expressions like "(3 + 4)" or "((1 + 2) * 3)".
/// It recursively calls parse_expression to handle nested expressions.
///
/// # Arguments
/// * `input` - The string slice to parse
///
/// # Returns
/// * `IResult<&str, Expr>` - Parser result with remaining input and parsed expression
fn parse_parenthesized(input: &str) -> IResult<&str, Expr> {
    let (input, _) = char('(')(input)?; // Consume opening parenthesis
    let (input, expr) = parse_expression(input)?; // Parse the inner expression
    let (input, _) = char(')')(input)?; // Consume closing parenthesis
    Ok((input, expr))
}

/// Parse a factor (number or parenthesized expression)
///
/// A factor is the most basic unit in our grammar hierarchy:
/// - A number (e.g., "42", "-3.14")
/// - A parenthesized expression (e.g., "(1 + 2)")
///
/// This function tries parentheses first, then falls back to parsing a number.
///
/// # Arguments
/// * `input` - The string slice to parse
///
/// # Returns
/// * `IResult<&str, Expr>` - Parser result with remaining input and parsed expression
fn parse_factor(input: &str) -> IResult<&str, Expr> {
    let (input, _) = multispace0(input)?; // Skip any leading whitespace

    // Try parsing parenthesized expression first
    if let Ok((input, expr)) = parse_parenthesized(input) {
        Ok((input, expr))
    } else {
        // Fall back to parsing a number
        parse_number(input)
    }
}

/// Parse multiplication and division (higher precedence)
///
/// This function implements the parsing of multiplication (*) and division (/) operations.
/// These operations have higher precedence than addition and subtraction, meaning they
/// are evaluated first in expressions like "2 + 3 * 4" (which becomes "2 + (3 * 4)").
///
/// The function uses left-associativity, so "8 / 4 / 2" becomes "((8 / 4) / 2) = 1".
///
/// # Arguments
/// * `input` - The string slice to parse
///
/// # Returns
/// * `IResult<&str, Expr>` - Parser result with remaining input and parsed expression
///
/// # Grammar
/// ```text
/// term = factor (("*" | "/") factor)*
/// ```
fn parse_term(input: &str) -> IResult<&str, Expr> {
    let (mut remaining, mut left) = parse_factor(input)?;

    // Continue parsing multiplication and division operations
    loop {
        let (input_after_whitespace, _) = multispace0(remaining)?;

        // Try to parse multiplication or division operator
        if let Ok((new_input, _)) =
            char::<&str, nom::error::Error<&str>>('*')(input_after_whitespace)
        {
            let (new_input, right) = parse_factor(new_input)?;
            left = Expr::Mul(Box::new(left), Box::new(right));
            remaining = new_input;
        } else if let Ok((new_input, _)) =
            char::<&str, nom::error::Error<&str>>('/')(input_after_whitespace)
        {
            let (new_input, right) = parse_factor(new_input)?;
            left = Expr::Div(Box::new(left), Box::new(right));
            remaining = new_input;
        } else {
            break; // No more multiplication or division operators
        }
    }

    Ok((remaining, left))
}

/// Parse addition and subtraction (lower precedence)
///
/// This is the main entry point for parsing mathematical expressions.
/// It handles addition (+) and subtraction (-) operations, which have the lowest
/// precedence in our operator hierarchy.
///
/// The function implements left-associativity, so "10 - 3 - 2" becomes "((10 - 3) - 2) = 5".
///
/// # Arguments
/// * `input` - The string slice to parse
///
/// # Returns
/// * `IResult<&str, Expr>` - Parser result with remaining input and parsed expression
///
/// # Grammar
/// ```text
/// expression = term (("+" | "-") term)*
/// term = factor (("*" | "/") factor)*
/// factor = number | "(" expression ")"
/// ```
///
/// # Examples
/// ```
/// use ast::{parse_expression, Expr};
///
/// // Simple precedence: multiplication before addition
/// let (_, ast) = parse_expression("3 + 4 * 2").unwrap();
/// match ast {
///     Expr::Add(left, right) => {
///         assert!(matches!(left.as_ref(), Expr::Float(3.0)));
///         assert!(matches!(right.as_ref(), Expr::Mul(_, _)));
///     }
///     _ => panic!("Expected Add at top level"),
/// }
///
/// // Parentheses override precedence
/// let (_, ast) = parse_expression("(1 + 2) * 3").unwrap();
/// match ast {
///     Expr::Mul(left, right) => {
///         assert!(matches!(left.as_ref(), Expr::Add(_, _)));
///         assert!(matches!(right.as_ref(), Expr::Float(3.0)));
///     }
///     _ => panic!("Expected Mul at top level"),
/// }
/// ```
pub fn parse_expression(input: &str) -> IResult<&str, Expr> {
    let (mut remaining, mut left) = parse_term(input)?;

    // Continue parsing addition and subtraction operations
    loop {
        let (input_after_whitespace, _) = multispace0(remaining)?;

        // Try to parse addition or subtraction operator
        if let Ok((new_input, _)) =
            char::<&str, nom::error::Error<&str>>('+')(input_after_whitespace)
        {
            let (new_input, right) = parse_term(new_input)?;
            left = Expr::Add(Box::new(left), Box::new(right));
            remaining = new_input;
        } else if let Ok((new_input, _)) =
            char::<&str, nom::error::Error<&str>>('-')(input_after_whitespace)
        {
            let (new_input, right) = parse_term(new_input)?;
            left = Expr::Sub(Box::new(left), Box::new(right));
            remaining = new_input;
        } else {
            break; // No more addition or subtraction operators
        }
    }

    Ok((remaining, left))
}

/// Evaluate an AST expression to a numeric result
///
/// This function recursively walks through the Abstract Syntax Tree and computes
/// the final numeric value. It handles all mathematical operations defined in the
/// `Expr` enum and provides proper error handling for division by zero.
///
/// # Arguments
/// * `expr` - The AST expression to evaluate
///
/// # Returns
/// * `Result<f64, EvaluationError>` - The computed result or an error
///
/// # Errors
/// * `EvaluationError::DivisionByZero` - When attempting to divide by zero
///
/// # Examples
/// ```
/// use ast::{parse_expression, evaluate, Expr, EvaluationError};
///
/// // Successful evaluation
/// let (_, ast) = parse_expression("3 + 4 * 2").unwrap();
/// let result = evaluate(&ast).unwrap();
/// assert_eq!(result, 11.0);
///
/// // Division by zero error
/// let ast = Expr::Div(Box::new(Expr::Float(8.0)), Box::new(Expr::Float(0.0)));
/// let result = evaluate(&ast);
/// assert!(matches!(result, Err(EvaluationError::DivisionByZero)));
/// ```
pub fn evaluate(expr: &Expr) -> Result<f64, EvaluationError> {
    match expr {
        Expr::Float(value) => Ok(*value),
        Expr::Add(left, right) => Ok(evaluate(left)? + evaluate(right)?),
        Expr::Sub(left, right) => Ok(evaluate(left)? - evaluate(right)?),
        Expr::Mul(left, right) => Ok(evaluate(left)? * evaluate(right)?),
        Expr::Div(left, right) => {
            let denominator = evaluate(right)?;
            if denominator == 0.0 {
                Err(EvaluationError::DivisionByZero)
            } else {
                Ok(evaluate(left)? / denominator)
            }
        }
    }
}

/// Comprehensive test suite for the AST calculator
///
/// This module contains extensive tests that verify:
/// - Correct parsing and evaluation of various mathematical expressions
/// - Proper operator precedence (multiplication before addition)
/// - Parentheses override precedence rules
/// - Error handling for division by zero
/// - Rejection of invalid syntax
#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing and evaluation of valid mathematical expressions
    ///
    /// This test covers a wide range of valid expressions including:
    /// - Basic arithmetic operations
    /// - Operator precedence
    /// - Parentheses grouping
    /// - Decimal numbers
    /// - Negative numbers
    /// - Complex nested expressions
    #[test]
    fn test_valid_expressions() {
        let test_cases = [
            ("1 + 2 * (3 - 4) / 5", 0.6), // Complex expression with precedence
            ("42", 42.0),                 // Simple number
            ("(1 + 2) * 3", 9.0),         // Parentheses override precedence
            ("10 / 2 + 3 * 4", 17.0),     // Mixed operations
            ("1 + 2 + 3 + 4", 10.0),      // Chain of additions
            ("3.14 + 2.86", 6.0),         // Decimal numbers
            ("-5 + 10", 5.0),             // Negative numbers
            ("-3.5 * 2", -7.0),           // Negative decimal
            ("10.5 / -2.1", -5.0),        // Division with negative
            ("-1.5 + -2.5", -4.0),        // Two negative numbers
        ];

        for (expression, expected) in &test_cases {
            match parse_expression(expression) {
                Ok((remaining, ast)) => {
                    // Ensure the entire expression was parsed
                    assert!(
                        remaining.trim().is_empty(),
                        "Unparsed input: '{}'",
                        remaining
                    );

                    // Evaluate and check the result
                    match evaluate(&ast) {
                        Ok(result) => {
                            assert!(
                                (result - expected).abs() < 1e-10,
                                "Expression '{}': expected {}, got {}",
                                expression,
                                expected,
                                result
                            );
                        }
                        Err(error) => panic!("Evaluation failed for '{}': {}", expression, error),
                    }
                }
                Err(error) => panic!("Parse failed for '{}': {:?}", expression, error),
            }
        }
    }

    /// Test that division by zero is properly handled
    ///
    /// This test ensures that dividing by zero returns the appropriate error
    /// rather than causing a panic or returning an invalid result.
    #[test]
    fn test_division_by_zero() {
        match parse_expression("8 / 0") {
            Ok((_, ast)) => {
                match evaluate(&ast) {
                    Err(EvaluationError::DivisionByZero) => (), // Expected
                    Ok(result) => panic!("Expected division by zero error, got {}", result),
                }
            }
            Err(error) => panic!("Parse failed: {:?}", error),
        }
    }

    /// Test that invalid expressions are properly rejected
    ///
    /// This test ensures that malformed expressions fail to parse completely
    /// or leave significant unparsed input, indicating a syntax error.
    #[test]
    fn test_invalid_expressions() {
        let invalid_expressions = [
            "* 40 - 10",  // Invalid: starts with operator
            "5 + + 3",    // Invalid: double operator
            "(5 + 3",     // Invalid: missing closing parenthesis
            "5 + ",       // Invalid: ends with operator
            "+ 5",        // Invalid: starts with plus
            "5 * / 3",    // Invalid: consecutive operators
            "((5 + 3)",   // Invalid: unmatched parentheses
            "5 + (3 * )", // Invalid: empty expression in parentheses
            "",           // Invalid: empty string
            "   ",        // Invalid: only whitespace
            "5 + abc",    // Invalid: contains letters
            "5 ** 3",     // Invalid: double multiplication
            "(((",        // Invalid: only opening parentheses
            ")))",        // Invalid: only closing parentheses
            "5 + ()",     // Invalid: empty parentheses
        ];

        for expression in &invalid_expressions {
            match parse_expression(expression) {
                Ok((remaining, _)) => {
                    // Some expressions might partially parse, which is acceptable
                    // as long as there's significant remaining input
                    if remaining.trim().is_empty() {
                        panic!(
                            "Expression '{}' should not have parsed completely",
                            expression
                        );
                    }
                }
                Err(_) => (), // Expected failure
            }
        }
    }

    /// Test that operator precedence is correctly implemented
    ///
    /// This test verifies that multiplication has higher precedence than addition,
    /// meaning "2 + 3 * 4" should be parsed as "2 + (3 * 4)" not "(2 + 3) * 4".
    #[test]
    fn test_operator_precedence() {
        // Test that multiplication has higher precedence than addition
        match parse_expression("2 + 3 * 4") {
            Ok((_, ast)) => {
                // Should parse as Add(2, Mul(3, 4)), not Mul(Add(2, 3), 4)
                match ast {
                    Expr::Add(left, right) => {
                        assert!(matches!(left.as_ref(), Expr::Float(2.0)));
                        assert!(matches!(right.as_ref(), Expr::Mul(_, _)));
                    }
                    _ => panic!("Expected Add at top level, got {:?}", ast),
                }
            }
            Err(error) => panic!("Parse failed: {:?}", error),
        }
    }

    /// Test that parentheses can override operator precedence
    ///
    /// This test verifies that parentheses force different grouping,
    /// so "(2 + 3) * 4" should be parsed as "(2 + 3) * 4", not "2 + (3 * 4)".
    #[test]
    fn test_parentheses_override_precedence() {
        // Test that parentheses can override precedence
        match parse_expression("(2 + 3) * 4") {
            Ok((_, ast)) => {
                // Should parse as Mul(Add(2, 3), 4)
                match ast {
                    Expr::Mul(left, right) => {
                        assert!(matches!(left.as_ref(), Expr::Add(_, _)));
                        assert!(matches!(right.as_ref(), Expr::Float(4.0)));
                    }
                    _ => panic!("Expected Mul at top level, got {:?}", ast),
                }
            }
            Err(error) => panic!("Parse failed: {:?}", error),
        }
    }
}
