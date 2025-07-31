//! Arbores - A Scheme-compatible Lisp interpreter written in Rust

// Legacy implementation - will be replaced by new modular design
pub mod legacy;

// New modular implementation (under development)
pub mod interpreter;

pub mod arbores;

// Re-export commonly used types and functions from legacy
pub use legacy::{Value, SchemeError, Result, run_repl};
pub use legacy::eval::{Evaluator, EvaluationContext};
pub use legacy::parser::Parser;
pub use legacy::repl::Repl;
pub use legacy::types::Position;
pub use arbores::Arbores;

// Re-export eval function with different name to avoid conflict
pub use legacy::eval as eval_string;

// For backward compatibility, expose eval submodule through a nested module
pub mod eval {
    pub use crate::legacy::eval::*;
}

/// Convenience function to evaluate a Scheme expression from a string
pub fn eval(input: &str) -> Result<Value> {
    let evaluator = legacy::eval::Evaluator::new();
    evaluator.eval_string(input, None)
}

/// Convenience function to parse a Scheme expression from a string
pub fn parse(input: &str) -> Result<Value> {
    legacy::parser::Parser::parse(input)
}
