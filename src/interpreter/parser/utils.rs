//! 语法分析器辅助工具
//! 
//! 本模块提供语法分析器的辅助功能，包括：
//! - 源代码重建工具（SourceBuilder）
//! - 位置信息处理函数
//! - 通用验证和转换函数
//! - AST 构建辅助函数

use crate::interpreter::lexer::types::{Token, TokenType};
use super::types::{SExpr, SExprContent, Value};

// ============================================================================
// 源代码重建工具
// ============================================================================

/// 源代码重建器 - 从 Token 流重建源代码文本
/// 
/// 这个工具保存经过解析器处理的所有 token，用于在错误发生时
/// 重建源代码的一部分，帮助用户定位问题。
#[derive(Debug, Clone)]
pub struct SourceBuilder {
    /// 累积的源代码文本
    text: String,
    /// 是否需要在下个token前添加空格
    need_space: bool,
}

impl SourceBuilder {
    /// 创建新的源代码重建器
    pub fn new() -> Self {
        Self {
            text: String::new(),
            need_space: false,
        }
    }
    
    /// 添加一个 token 到重建的源代码中
    /// 
    /// 使用 Token 的 raw_text 字段来重建源代码，确保保持原始文本格式，
    /// 避免 span 错位问题。例如 "+1" 和 "1" 会保持原始表示形式。
    pub fn add_token(&mut self, token: &Token) {
        // 对于 Trivia tokens（空白、换行、注释），直接添加原始文本
        if token.token_type.is_trivia() {
            self.text.push_str(token.raw_text());
            // Trivia tokens 不影响空格状态
            return;
        }
        
        // 根据 token 类型决定是否需要空格分隔
        let needs_spacing = self.should_add_space(&token.token_type);
        
        if needs_spacing && self.need_space && !self.text.is_empty() {
            self.text.push(' ');
        }
        
        // 直接使用 token 的原始文本内容，避免重新构造导致的格式丢失
        self.text.push_str(token.raw_text());
        
        // 更新下次是否需要空格的状态
        self.need_space = self.should_space_after(&token.token_type);
    }
    
    /// 完成构建，返回重建的源代码文本
    pub fn build(self) -> String {
        self.text
    }
    
    /// 获取当前构建的文本长度
    pub fn len(&self) -> usize {
        self.text.len()
    }
    
    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
    
    /// 判断是否应该在当前 token 前添加空格
    fn should_add_space(&self, token_type: &TokenType) -> bool {
        !matches!(token_type, 
            TokenType::RightParen | 
            TokenType::RightBracket | 
            TokenType::Dot |
            TokenType::Whitespace(_) |
            TokenType::Newline |
            TokenType::Comment(_)
        )
    }
    
    /// 判断是否应该在当前 token 后添加空格
    fn should_space_after(&self, token_type: &TokenType) -> bool {
        !matches!(token_type, 
            TokenType::LeftParen | 
            TokenType::LeftBracket |
            TokenType::Quote |
            TokenType::Quasiquote |
            TokenType::Unquote |
            TokenType::UnquoteSplicing |
            TokenType::Whitespace(_) |
            TokenType::Newline |
            TokenType::Comment(_)
        )
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 从 TokenType 创建对应的 Value
pub fn token_to_value(token_type: TokenType) -> Option<Value> {
    match token_type {
        TokenType::Integer(n) => Some(Value::Number(n as f64)),
        TokenType::Float(n) => Some(Value::Number(n)),
        TokenType::String(s) => Some(Value::String(s)),
        TokenType::Character(c) => Some(Value::Character(c)),
        TokenType::Boolean(b) => Some(Value::Boolean(b)),
        TokenType::Symbol(s) => Some(Value::Symbol(s)),
        _ => None,
    }
}

/// 判断 TokenType 是否表示原子值
pub fn is_atom_token(token_type: &TokenType) -> bool {
    matches!(token_type,
        TokenType::Integer(_) |
        TokenType::Float(_) |
        TokenType::String(_) |
        TokenType::Character(_) |
        TokenType::Boolean(_) |
        TokenType::Symbol(_)
    )
}

/// 构建列表的 S表达式结构
/// 
/// 将元素列表转换为嵌套的 Cons 结构，最后一个 cdr 指向 Nil
pub fn build_list(elements: Vec<SExpr>) -> SExprContent {
    elements.into_iter()
        .rev()
        .fold(SExprContent::Nil, |acc, elem| {
            SExprContent::Cons {
                car: std::rc::Rc::new(elem),
                cdr: std::rc::Rc::new(SExpr::without_span(acc)),
            }
        })
}

/// 构建点对列表的 S表达式结构
/// 
/// 将元素列表和尾部表达式转换为嵌套的 Cons 结构，
/// 最后一个 cdr 指向尾部表达式而非 Nil
pub fn build_dotted_list(elements: Vec<SExpr>, tail: SExpr) -> SExprContent {
    elements.into_iter()
        .rev()
        .fold(tail.content, |acc, elem| {
            SExprContent::Cons {
                car: std::rc::Rc::new(elem),
                cdr: std::rc::Rc::new(SExpr::without_span(acc)),
            }
        })
}

/// 验证点对列表的合法性
pub fn validate_dotted_list_structure(
    elements: &[SExpr], 
    dot_position: usize
) -> Result<(), super::types::DottedListError> {
    use super::types::DottedListError;
    
    // 点号不能在开头
    if dot_position == 0 {
        return Err(DottedListError::InvalidDotPosition);
    }
    
    // 点号前必须至少有一个元素
    if elements.is_empty() {
        return Err(DottedListError::InsufficientElements);
    }
    
    // 点号位置不能超出元素范围
    if dot_position > elements.len() {
        return Err(DottedListError::InvalidDotPosition);
    }
    
    Ok(())
}

// ============================================================================
// Default 实现
// ============================================================================

impl Default for SourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::lexer::types::{Token, Span};

    #[test]
    fn test_source_builder_basic() {
        let mut builder = SourceBuilder::new();
        
        let token1 = Token::new(
            TokenType::LeftParen,
            Span::new(0, 1),
            "(".to_string(),
        );
        let token2 = Token::new(
            TokenType::Symbol("define".to_string()),
            Span::new(1, 7),
            "define".to_string(),
        );
        let token3 = Token::new(
            TokenType::Symbol("x".to_string()),
            Span::new(8, 9),
            "x".to_string(),
        );
        let token4 = Token::new(
            TokenType::Integer(42),
            Span::new(10, 12),
            "42".to_string(),
        );
        let token5 = Token::new(
            TokenType::RightParen,
            Span::new(12, 13),
            ")".to_string(),
        );
        
        builder.add_token(&token1);
        builder.add_token(&token2);
        builder.add_token(&token3);
        builder.add_token(&token4);
        builder.add_token(&token5);
        
        let result = builder.build();
        assert_eq!(result, "(define x 42)");
    }

    #[test]
    fn test_source_builder_preserves_original_text() {
        let mut builder = SourceBuilder::new();
        
        // 测试原始文本保留：+1 应该保持为 "+1" 而不是 "1"
        let token_plus_one = Token::new(
            TokenType::Integer(1),
            Span::new(0, 2),
            "+1".to_string(),  // 原始文本是 "+1"
        );
        
        // 测试布尔值的不同表示形式
        let token_true_short = Token::new(
            TokenType::Boolean(true),
            Span::new(3, 5),
            "#t".to_string(),  // 原始文本是短形式
        );
        
        // 测试浮点数的不同表示形式
        let token_float = Token::new(
            TokenType::Float(1.0),
            Span::new(6, 10),
            "1.00".to_string(),  // 原始文本保留小数点后的零
        );
        
        builder.add_token(&token_plus_one);
        builder.add_token(&token_true_short);
        builder.add_token(&token_float);
        
        let result = builder.build();
        // 验证原始文本格式被保留
        assert_eq!(result, "+1 #t 1.00");
    }

    #[test]
    fn test_source_builder_with_trivia_tokens() {
        let mut builder = SourceBuilder::new();
        
        let token1 = Token::new(
            TokenType::LeftParen,
            Span::new(0, 1),
            "(".to_string(),
        );
        let whitespace = Token::new(
            TokenType::Whitespace("  ".to_string()),
            Span::new(1, 3),
            "  ".to_string(),
        );
        let token2 = Token::new(
            TokenType::Symbol("x".to_string()),
            Span::new(3, 4),
            "x".to_string(),
        );
        let comment = Token::new(
            TokenType::Comment(" this is x".to_string()),
            Span::new(4, 16),
            "; this is x".to_string(),
        );
        let newline = Token::new(
            TokenType::Newline,
            Span::new(16, 17),
            "\n".to_string(),
        );
        let token3 = Token::new(
            TokenType::RightParen,
            Span::new(17, 18),
            ")".to_string(),
        );
        
        builder.add_token(&token1);
        builder.add_token(&whitespace);
        builder.add_token(&token2);
        builder.add_token(&comment);
        builder.add_token(&newline);
        builder.add_token(&token3);
        
        let result = builder.build();
        assert_eq!(result, "(  x; this is x\n)");
    }

    #[test]
    fn test_token_to_value() {
        assert_eq!(
            token_to_value(TokenType::Integer(42)),
            Some(Value::Number(42.0))
        );
        assert_eq!(
            token_to_value(TokenType::Float(3.14)),
            Some(Value::Number(3.14))
        );
        assert_eq!(
            token_to_value(TokenType::String("hello".to_string())),
            Some(Value::String("hello".to_string()))
        );
        assert_eq!(
            token_to_value(TokenType::Boolean(true)),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            token_to_value(TokenType::LeftParen),
            None
        );
    }

    #[test]
    fn test_is_atom_token() {
        assert!(is_atom_token(&TokenType::Integer(42)));
        assert!(is_atom_token(&TokenType::Float(3.14)));
        assert!(is_atom_token(&TokenType::String("hello".to_string())));
        assert!(is_atom_token(&TokenType::Symbol("x".to_string())));
        assert!(!is_atom_token(&TokenType::LeftParen));
        assert!(!is_atom_token(&TokenType::Quote));
    }
}
