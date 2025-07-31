//! Legacy implementation of the Arbores Scheme interpreter
//! 
//! This module contains the existing implementation that will be gradually
//! replaced by the new modular design. It provides all the functionality
//! needed for the current REPL and evaluation system.

pub mod builtins;
pub mod env;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod storage;
pub mod types;

// Re-export commonly used types and functions for easier access
pub use eval::Evaluator;
pub use repl::run_repl;
pub use types::*;
