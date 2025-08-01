//! 语法分析器模块
//! 
//! 本模块实现了Scheme语言的语法分析器，采用简化的递归下降解析算法。
//! 
//! ## 设计特点
//! 
//! - **函数式设计**：纯函数实现，数据与行为分离
//! - **简化架构**：基于Scheme语法简单性，不使用复杂的规则系统  
//! - **源码追踪**：每个AST节点都包含准确的位置信息
//! - **错误友好**：详细的错误信息和源码重建
//! 
//! ## 基本用法
//! 
//! ```rust,no_run
//! use arbores::interpreter::lexer;
//! use arbores::interpreter::parser;
//! 
//! // 从源代码解析S表达式
//! let source = "(define x 42)";
//! let tokens = lexer::tokenize(source.chars());
//! let parse_output = parser::parse(tokens);
//! 
//! match parse_output.result {
//!     Ok(expressions) => {
//!         println!("Parsed {} expressions", expressions.len());
//!         for expr in expressions {
//!             println!("Expression: {:?}", expr);
//!         }
//!     },
//!     Err(error) => {
//!         eprintln!("Parse error: {}", error);
//!         eprintln!("Source: {}", parse_output.source_text);
//!     }
//! }
//! ```
//! 
//! ## 支持的语法
//! 
//! - **原子值**：数字、字符串、字符、布尔值、符号
//! - **列表**：`(expr1 expr2 ...)`
//! - **点对列表**：`(expr1 . expr2)`
//! - **引用语法**：`'expr`、`` `expr ``、`,expr`、`,@expr`
//! - **向量**：`#(expr1 expr2 ...)` (预留，待lexer支持)
//! 
//! ## 模块结构
//! 
//! - `types`：核心数据类型定义（AST、错误类型等）
//! - `engine`：解析器引擎实现
//! - `utils`：辅助工具（源码重建、类型转换等）

// 导出公共类型
pub use types::{
    SExpr, SExprContent, Value, 
    ParseError, ParseResult, ParseOutput,
    UnexpectedTokenReason, DottedListError, QuoteType
};

// 导出主要API
pub use engine::parse;

// 导出辅助工具（供高级用户使用）
pub use utils::{SourceBuilder, token_to_value, is_atom_token};

// 内部模块
mod types;
mod engine;
mod utils;

// 测试模块
#[cfg(test)]
mod integration_tests;

// ============================================================================
// 便利函数
// ============================================================================

/// 从字符串源代码直接解析S表达式
/// 
/// 这是一个便利函数，将词法分析和语法分析组合在一起。
/// 
/// # 参数
/// - `source`: 源代码字符串
/// 
/// # 返回值
/// 包含解析结果和重建源码的 `ParseOutput`
/// 
/// # 示例
/// ```rust,no_run
/// use arbores::interpreter::parser::parse_from_string;
/// let result = parse_from_string("(+ 1 2)");
/// match result.result {
///     Ok(exprs) => println!("Parsed: {:?}", exprs),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn parse_from_string(source: &str) -> ParseOutput {
    let tokens = crate::interpreter::lexer::tokenize(source.chars());
    parse(tokens)
}

/// 检查解析结果是否成功
pub fn is_parse_success(output: &ParseOutput) -> bool {
    output.result.is_ok()
}

/// 获取解析错误的简短描述
pub fn get_error_summary(output: &ParseOutput) -> Option<String> {
    match &output.result {
        Err(error) => Some(format!("{}", error)),
        Ok(_) => None,
    }
}

// ============================================================================
// 测试支持
// ============================================================================

#[cfg(test)]
pub mod test_utils {
    use crate::interpreter::lexer::types::{Token, TokenType, Span, LexError};

    /// 创建测试用的token
    pub fn create_test_token(token_type: TokenType, start: usize, text: &str) -> Result<Token, LexError> {
        let span = Span::new(start, start + text.len());
        Ok(Token::new(token_type, span, text.to_string()))
    }

    /// 创建测试用的简单token序列
    pub fn create_simple_tokens() -> Vec<Result<Token, LexError>> {
        vec![
            create_test_token(TokenType::LeftParen, 0, "("),
            create_test_token(TokenType::Symbol("define".to_string()), 1, "define"),
            create_test_token(TokenType::Symbol("x".to_string()), 8, "x"),
            create_test_token(TokenType::Integer(42), 10, "42"),
            create_test_token(TokenType::RightParen, 12, ")"),
        ]
    }
}
