//! 新的模块化解释器实现
//! 
//! 本模块包含了重新设计的解释器各个组件，采用函数式设计原则。

pub mod lexer;
pub mod parser;
pub mod evaluator;

// 重新导出词法分析器的主要接口
pub use lexer::{tokenize, tokenize_string, Token, TokenType, LexError};

// 重新导出语法分析器的主要接口
pub use parser::{parse, parse_from_string, SExpr, SExprContent, Value, ParseError, ParseOutput};

// 重新导出求值器的主要接口
pub use evaluator::{
    RuntimeValue, Environment, 
    EvaluateError
};
