//! 新的模块化词法分析器
//! 
//! 本模块实现了基于状态机的词法分析器，采用函数式设计原则。

pub mod types;
pub mod char_stream;
pub mod pattern_matcher;
pub mod rules;
pub mod engine;
pub mod demo;

#[cfg(test)]
pub mod number_tests;

pub mod number_demo;

// 重新导出主要的公共类型和函数
pub use types::{Token, TokenType, LexError, LexErrorReason, Span};
pub use char_stream::CharStream;
pub use pattern_matcher::{MatchResult, match_pattern, match_char_class_sequence};
pub use engine::{tokenize, tokenize_string, tokenize_to_vec, filter_trivia_tokens};
