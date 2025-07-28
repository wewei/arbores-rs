/// Arbores - A Scheme-compatible Lisp interpreter written in Rust

pub mod types;
pub mod lexer;
pub mod parser;
pub mod env;
pub mod eval;
pub mod builtins;
pub mod repl;

// Re-export commonly used types and functions
pub use types::{Value, SchemeError, Result};
pub use eval::Evaluator;
pub use repl::Repl;
pub use parser::Parser;

/// Convenience function to evaluate a Scheme expression from a string
pub fn eval(input: &str) -> Result<Value> {
    let evaluator = Evaluator::new();
    evaluator.eval_string(input)
}

/// Convenience function to parse a Scheme expression from a string
pub fn parse(input: &str) -> Result<Value> {
    Parser::parse(input)
}
