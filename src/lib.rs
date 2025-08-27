//! AST Calculator
//!
//! A mathematical expression parser and evaluator that builds Abstract Syntax Trees (ASTs).
//! This calculator demonstrates recursive descent parsing using the `nom` parser combinator library.
//!
//! # Example
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

use nom::Finish;
use nom::{
    IResult,
    branch::alt,
    character::complete::{char, multispace0, one_of},
    combinator::{all_consuming, map},
    multi::fold_many0,
    number::complete::double,
    sequence::{delimited, preceded, terminated},
};
use thiserror::Error;

/// Errors that can occur during expression evaluation
#[derive(Error, Debug, PartialEq)]
pub enum EvaluationError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Modulo by zero")]
    ModuloByZero,
}

/// A top-level error type that encapsulates all possible failures
#[derive(Error, Debug)]
pub enum CalcError {
    /// An error that occurs during the evaluation of the AST
    #[error(transparent)]
    Evaluation(#[from] EvaluationError),
    /// An error that occurs during the parsing of an expression
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Abstract Syntax Tree representation of mathematical expressions
///
/// This enum represents the structure of mathematical expressions as a tree,
/// where each node is either a value or an operation with child nodes.
/// Operations are stored as boxed expressions to allow for nested structures.
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    /// A floating-point numeric literal
    ///
    /// Examples: `42.0`, `-3.14`, `0.5`
    Float(f64),

    /// Addition operation: left + right
    ///
    /// Represents the sum of two expressions. Both operands are evaluated
    /// and their results are added together.
    Add(Box<Expr>, Box<Expr>),

    /// Subtraction operation: left - right
    ///
    /// Represents the difference between two expressions. The right operand
    /// is subtracted from the left operand.
    Sub(Box<Expr>, Box<Expr>),

    /// Multiplication operation: left * right
    ///
    /// Represents the product of two expressions. Both operands are evaluated
    /// and their results are multiplied together.
    Mul(Box<Expr>, Box<Expr>),

    /// Division operation: left / right
    ///
    /// Represents the quotient of two expressions. The left operand is divided
    /// by the right operand. Division by zero will result in an evaluation error.
    Div(Box<Expr>, Box<Expr>),

    /// Power operation: left ^ right
    ///
    /// Represents exponentiation. This operation is right-associative.
    Power(Box<Expr>, Box<Expr>),

    /// Modulo operation: left % right
    ///
    /// Represents the remainder of a division.
    Modulo(Box<Expr>, Box<Expr>),

    /// Negation operation: -expr
    ///
    /// Represents the negation of an expression (unary minus).
    /// Example: `-x` or `-(2 / 1)`
    Neg(Box<Expr>),
}

/// Parse a number into an Expr::Float (supports decimals and negative numbers)
///
/// This function handles both positive and negative floating-point numbers.
/// Examples: "42", "-3.14", "0.5", "-0.25"
///
/// # Example
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

use nom::Parser;

/// A top-level parser that consumes the entire input and provides a clean error API
///
/// This function is the public entry point for parsing expressions. It ensures that
/// the entire input string is consumed, and it converts `nom`'s internal error
/// types into a user-friendly `CalcError::Parse`.
pub fn parse(input: &str) -> Result<Expr, CalcError> {
    match all_consuming(parse_expression).parse(input).finish() {
        Ok((_, ast)) => Ok(ast),
        Err(e) => Err(CalcError::Parse(format!(
            "Invalid syntax near: '{}'",
            e.input
        ))),
    }
}

/// Parse an expression wrapped in parentheses, handling surrounding whitespace
///
/// This function handles expressions like `(3 + 4)` or `( (1 + 2) * 3 )`.
/// It uses `nom`'s `delimited` combinator to handle the parentheses and
/// `preceded`/`terminated` to manage whitespace within the parentheses.
fn parse_parenthesized(input: &str) -> IResult<&str, Expr> {
    delimited(
        char('('),
        preceded(multispace0, terminated(parse_expression, multispace0)),
        char(')'),
    )
    .parse(input)
}

/// Parse a primary expression (a number or a parenthesized expression)
///
/// This is the most basic unit in the grammar.
fn parse_primary(input: &str) -> IResult<&str, Expr> {
    preceded(multispace0, alt((parse_parenthesized, parse_number))).parse(input)
}

/// Parse the exponentiation operator (right-associative)
fn parse_power(input: &str) -> IResult<&str, Expr> {
    let (input, left) = parse_primary(input)?;
    let (input, maybe_op) = nom::combinator::opt(preceded(multispace0, char('^'))).parse(input)?;
    if maybe_op.is_some() {
        let (input, right) = parse_power(input)?;
        Ok((input, Expr::Power(Box::new(left), Box::new(right))))
    } else {
        Ok((input, left))
    }
}

/// Parse unary minus.
fn parse_unary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            preceded(preceded(multispace0, char('-')), parse_unary),
            |expr| Expr::Neg(Box::new(expr)),
        ),
        parse_power,
    ))
    .parse(input)
}

/// Parse multiplication, division, and modulo (higher precedence than addition)
fn parse_term(input: &str) -> IResult<&str, Expr> {
    let (input, left) = parse_unary(input)?;

    fold_many0(
        (
            preceded(multispace0, one_of("*/%")),
            preceded(multispace0, parse_unary),
        ),
        move || left.clone(),
        |acc, (op, right)| match op {
            '*' => Expr::Mul(Box::new(acc), Box::new(right)),
            '/' => Expr::Div(Box::new(acc), Box::new(right)),
            '%' => Expr::Modulo(Box::new(acc), Box::new(right)),
            _ => unreachable!(),
        },
    )
    .parse(input)
}

/// Parse addition and subtraction (lower precedence)
///
/// This is the main entry point for parsing mathematical expressions. It uses
/// `fold_many0` to parse a left-associative chain of addition and subtraction
/// operations. It starts by parsing a `term` and then repeatedly parses `+` or
/// `-` followed by another `term`.
///
/// # Example
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
    let (input, left) = parse_term(input)?;

    fold_many0(
        (
            preceded(multispace0, one_of("+-")),
            preceded(multispace0, parse_term),
        ),
        move || left.clone(),
        |acc, (op, right)| match op {
            '+' => Expr::Add(Box::new(acc), Box::new(right)),
            '-' => Expr::Sub(Box::new(acc), Box::new(right)),
            _ => unreachable!(),
        },
    )
    .parse(input)
}

/// Evaluate an AST expression to a numeric result
///
/// This function recursively walks through the Abstract Syntax Tree and computes
/// the final numeric value. It handles all mathematical operations defined in the
/// `Expr` enum and provides proper error handling for division by zero.
///
/// # Example
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
        Expr::Power(left, right) => {
            let left_val = evaluate(left)?;
            let right_val = evaluate(right)?;
            Ok(left_val.powf(right_val))
        }
        Expr::Modulo(left, right) => {
            let right_val = evaluate(right)?;
            if right_val == 0.0 {
                Err(EvaluationError::ModuloByZero)
            } else {
                Ok(evaluate(left)? % right_val)
            }
        }
        Expr::Neg(inner) => Ok(-evaluate(inner)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing and evaluation of valid mathematical expressions
    #[test]
    fn test_valid_expressions() {
        let test_cases = [
            // Basic operations
            ("42", 42.0),
            ("1 + 2", 3.0),
            ("10 - 3", 7.0),
            ("5 * 4", 20.0),
            ("20 / 4", 5.0),
            ("10 % 3", 1.0),
            // Operator precedence
            ("10 / 2 + 3 * 4", 17.0),
            ("10 % 3 * 2", 2.0),
            // Parentheses
            ("(1 + 2) * 3", 9.0),
            // Negative numbers and unary minus
            ("-5 + 10", 5.0),
            ("10 - -5", 15.0),
            ("-3.5 * 2", -7.0),
            ("10.5 / -2.1", -5.0),
            ("-1.5 + -2.5", -4.0),
            ("5 * -3", -15.0),
            ("5 * - 3", -15.0),
            ("- 5", -5.0),
            // Negation of subexpressions
            ("2 + -(2 / 1)", 0.0),
            ("-(3 + 4) * 2", -14.0),
            ("-(-5)", 5.0),
            // Exponentiation
            ("2^3", 8.0),
            ("2 ^ 3", 8.0),
            ("-2^4", -16.0), // -(2^4)
            ("(-2)^4", 16.0),
            ("2^3^2", 512.0),  // Right-associativity: 2^(3^2)
            ("(2^3)^2", 64.0), // vs. (2^3)^2
            ("10 / 2^2", 2.5), // Precedence: 10 / (2^2)
            ("2 * 3^2", 18.0), // Precedence: 2 * (3^2)
            ("2 + 3^2", 11.0), // Precedence: 2 + (3^2)
            // Complex expressions
            ("1 + 2 * (3 - 4) / 5", 0.6),
            ("3.14 + 2.86", 6.0),
        ];

        for (expression, expected) in &test_cases {
            match parse(expression) {
                Ok(ast) => match evaluate(&ast) {
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
                },
                Err(error) => panic!("Parse failed for '{}': {}", expression, error),
            }
        }
    }

    /// Test that division by zero is properly handled
    #[test]
    fn test_division_by_zero() {
        let result = parse("8 / 0").and_then(|ast| evaluate(&ast).map_err(CalcError::from));
        match result {
            Err(CalcError::Evaluation(EvaluationError::DivisionByZero)) => {} // Expected
            _ => panic!("Expected a division by zero evaluation error"),
        }
    }

    /// Test that modulo by zero is properly handled
    #[test]
    fn test_modulo_by_zero() {
        let result = parse("10 % 0").and_then(|ast| evaluate(&ast).map_err(CalcError::from));
        match result {
            Err(CalcError::Evaluation(EvaluationError::ModuloByZero)) => {} // Expected
            _ => panic!("Expected a modulo by zero evaluation error"),
        }
    }

    /// Test that invalid expressions are properly rejected
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
            "5 ^ ^ 3",    // Invalid: double exponent
        ];

        for expression in &invalid_expressions {
            assert!(
                parse(expression).is_err(),
                "Expression '{}' should have failed to parse",
                expression
            );
        }
    }

    /// Test that operator precedence is correctly implemented
    #[test]
    fn test_operator_precedence() {
        // Test: 2 + 3 * 4 -> Add(2, Mul(3, 4))
        let ast = parse("2 + 3 * 4").unwrap();
        match ast {
            Expr::Add(left, right) => {
                assert!(matches!(*left, Expr::Float(2.0)));
                assert!(matches!(*right, Expr::Mul(_, _)));
            }
            _ => panic!("Expected Add at top level, got {:?}", ast),
        }

        // Test: 2 * 3 ^ 2 -> Mul(2, Power(3, 2))
        let ast = parse("2 * 3^2").unwrap();
        match ast {
            Expr::Mul(left, right) => {
                assert!(matches!(*left, Expr::Float(2.0)));
                assert!(matches!(*right, Expr::Power(_, _)));
            }
            _ => panic!("Expected Mul at top level, got {:?}", ast),
        }
    }

    /// Test that parentheses can override operator precedence
    #[test]
    fn test_parentheses_override_precedence() {
        // Test: (2 + 3) * 4 -> Mul(Add(2, 3), 4)
        let ast = parse("(2 + 3) * 4").unwrap();
        match ast {
            Expr::Mul(left, right) => {
                assert!(matches!(*left, Expr::Add(_, _)));
                assert!(matches!(*right, Expr::Float(4.0)));
            }
            _ => panic!("Expected Mul at top level, got {:?}", ast),
        }
    }
}
