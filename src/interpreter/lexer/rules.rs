//! 状态机规则定义和 Token 生成器
//! 
//! 本模块包含词法分析器的状态机规则定义和各种 Token 生成器函数。
//! 遵循函数式设计原则，所有生成器都是纯函数。

use crate::interpreter::lexer::types::{
    Token, TokenType, LexError, LexErrorReason, Pattern, StateAction, TransitionRule, StateMachine
};

// ============================================================================
// 状态常量定义
// ============================================================================

pub const STATE_INITIAL: usize = 0;
pub const STATE_NUMBER: usize = 1;
pub const STATE_STRING: usize = 2;
pub const STATE_STRING_ESCAPE: usize = 3;
pub const STATE_SYMBOL: usize = 4;
pub const STATE_COMMENT: usize = 5;
pub const STATE_WHITESPACE: usize = 6;
pub const STATE_CHARACTER: usize = 7;
pub const STATE_NUMBER_SIGN: usize = 8;    // 处理数字的正负号
pub const STATE_NUMBER_DECIMAL: usize = 9; // 处理小数点后的数字
pub const STATE_NUMBER_EXP: usize = 10;    // 处理科学计数法的指数部分
pub const STATE_NUMBER_EXP_SIGN: usize = 11; // 处理指数的正负号

// ============================================================================
// Token 生成器函数 - 纯函数实现
// ============================================================================

/// 生成数字 Token
pub fn emit_number(raw_text: &str, position: usize) -> Result<Token, LexError> {
    parse_number(raw_text, position)
}

/// 解析数值字面量，支持整数、浮点数和科学计数法
fn parse_number(raw_text: &str, position: usize) -> Result<Token, LexError> {
    let text = raw_text.trim();
    
    // 检查是否包含小数点或科学计数法标记
    let has_decimal = text.contains('.');
    let has_exponent = text.contains('e') || text.contains('E');
    
    if has_decimal || has_exponent {
        // 解析为浮点数
        match text.parse::<f64>() {
            Ok(value) => Ok(Token::from_text(
                TokenType::Float(value),
                raw_text,
                position,
            )),
            Err(_) => Err(LexError::new(
                position,
                None,
                LexErrorReason::InvalidNumber {
                    partial_text: raw_text.to_string(),
                },
            )),
        }
    } else {
        // 解析为整数
        match text.parse::<i64>() {
            Ok(value) => Ok(Token::from_text(
                TokenType::Integer(value),
                raw_text,
                position,
            )),
            Err(_) => {
                // 如果整数解析失败，尝试作为浮点数
                match text.parse::<f64>() {
                    Ok(value) => Ok(Token::from_text(
                        TokenType::Float(value),
                        raw_text,
                        position,
                    )),
                    Err(_) => Err(LexError::new(
                        position,
                        None,
                        LexErrorReason::InvalidNumber {
                            partial_text: raw_text.to_string(),
                        },
                    )),
                }
            }
        }
    }
}

/// 生成字符串 Token
pub fn emit_string(raw_text: &str, position: usize) -> Result<Token, LexError> {
    // 去掉首尾的引号
    if raw_text.len() < 2 || !raw_text.starts_with('"') || !raw_text.ends_with('"') {
        return Err(LexError::new(
            position,
            None,
            LexErrorReason::UnterminatedString,
        ));
    }
    
    let content = &raw_text[1..raw_text.len()-1];
    
    // 处理转义序列
    let processed_content = process_string_escapes(content, position)?;
    
    Ok(Token::from_text(
        TokenType::String(processed_content),
        raw_text,
        position,
    ))
}

/// 生成符号 Token
pub fn emit_symbol(raw_text: &str, position: usize) -> Result<Token, LexError> {
    // 检查是否为布尔值
    match raw_text {
        "#t" | "#true" => Ok(Token::from_text(
            TokenType::Boolean(true),
            raw_text,
            position,
        )),
        "#f" | "#false" => Ok(Token::from_text(
            TokenType::Boolean(false),
            raw_text,
            position,
        )),
        _ => Ok(Token::from_text(
            TokenType::Symbol(raw_text.to_string()),
            raw_text,
            position,
        )),
    }
}

/// 生成注释 Token
pub fn emit_comment(raw_text: &str, position: usize) -> Result<Token, LexError> {
    // 去掉开头的分号
    let content = raw_text.strip_prefix(';').unwrap_or(raw_text);
    
    Ok(Token::from_text(
        TokenType::Comment(content.to_string()),
        raw_text,
        position,
    ))
}

/// 生成空白字符 Token
pub fn emit_whitespace(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(
        TokenType::Whitespace(raw_text.to_string()),
        raw_text,
        position,
    ))
}

/// 生成换行符 Token
pub fn emit_newline(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::Newline, raw_text, position))
}

/// 生成左括号 Token
pub fn emit_left_paren(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::LeftParen, raw_text, position))
}

/// 生成右括号 Token
pub fn emit_right_paren(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::RightParen, raw_text, position))
}

/// 生成左方括号 Token
pub fn emit_left_bracket(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::LeftBracket, raw_text, position))
}

/// 生成右方括号 Token
pub fn emit_right_bracket(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::RightBracket, raw_text, position))
}

/// 生成引号 Token
pub fn emit_quote(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::Quote, raw_text, position))
}

/// 生成反引号 Token
pub fn emit_quasiquote(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::Quasiquote, raw_text, position))
}

/// 生成逗号反引号 Token
pub fn emit_unquote_splicing(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::UnquoteSplicing, raw_text, position))
}

/// 生成逗号 Token
pub fn emit_unquote(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::Unquote, raw_text, position))
}

/// 生成点号 Token
pub fn emit_dot(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::Dot, raw_text, position))
}

/// 生成向量开始 Token
pub fn emit_vector_start(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::VectorStart, raw_text, position))
}

/// 生成字符 Token
pub fn emit_character(raw_text: &str, position: usize) -> Result<Token, LexError> {
    // 字符字面量格式：#\字符 或 #\字符名
    if !raw_text.starts_with("#\\") {
        return Err(LexError::new(
            position,
            raw_text.chars().next(),
            LexErrorReason::InvalidCharacter,
        ));
    }
    
    let char_part = &raw_text[2..];
    let character = match char_part {
        "space" => ' ',
        "newline" => '\n',
        "tab" => '\t',
        "return" => '\r',
        c if c.chars().count() == 1 => c.chars().next().unwrap(),
        _ => return Err(LexError::new(
            position,
            raw_text.chars().next(),
            LexErrorReason::InvalidCharacter,
        )),
    };
    
    Ok(Token::from_text(
        TokenType::Character(character),
        raw_text,
        position,
    ))
}

/// 生成文件结束 Token
pub fn emit_eof(_raw_text: &str, position: usize) -> Result<Token, LexError> {
    Ok(Token::from_text(TokenType::Eof, "", position))
}

/// 生成错误 Token
pub fn emit_error(raw_text: &str, position: usize) -> Result<Token, LexError> {
    Err(LexError::new(
        position,
        raw_text.chars().next(),
        LexErrorReason::InvalidCharacter,
    ))
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 处理字符串中的转义序列
fn process_string_escapes(content: &str, position: usize) -> Result<String, LexError> {
    let mut result = String::new();
    let mut chars = content.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some(escape_char) => {
                    return Err(LexError::new(
                        position,
                        Some(escape_char),
                        LexErrorReason::InvalidEscape { escape_char },
                    ));
                }
                None => {
                    return Err(LexError::new(
                        position,
                        None,
                        LexErrorReason::UnterminatedString,
                    ));
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}

// ============================================================================
// 字符类判断函数
// ============================================================================

/// 判断是否为数字字符（仅限0-9）
pub fn is_digit_char(ch: char) -> bool {
    ch.is_ascii_digit()
}

/// 判断是否为数字的起始字符（数字或正负号）
pub fn is_number_start_char(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '-' || ch == '+'
}

/// 判断是否为数字中可能出现的字符（包括小数点和科学计数法）
pub fn is_number_char(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '-' || ch == '+'
}

/// 判断是否为符号的起始字符
pub fn is_symbol_start_char(ch: char) -> bool {
    ch.is_alphabetic() || "!$%&*+-/:<=>?@^_~#".contains(ch)
}

/// 判断是否为符号的后续字符
pub fn is_symbol_char(ch: char) -> bool {
    is_symbol_start_char(ch) || ch.is_ascii_digit()
}

/// 判断是否为空白字符（不包括换行）
pub fn is_whitespace_char(ch: char) -> bool {
    ch.is_whitespace() && ch != '\n' && ch != '\r'
}

/// 判断是否为换行字符
pub fn is_newline_char(ch: char) -> bool {
    ch == '\n' || ch == '\r'
}

/// 判断是否为分隔符
pub fn is_delimiter_char(ch: char) -> bool {
    is_whitespace_char(ch) || is_newline_char(ch) || "()[];\"".contains(ch)
}

// ============================================================================
// 状态机规则定义
// ============================================================================

/// 获取 Scheme 词法分析器的状态机规则
pub fn get_scheme_state_machine() -> StateMachine {
    StateMachine {
        rules: vec![
            // STATE_INITIAL (0) 的规则
            vec![
                // 左括号
                TransitionRule::new(
                    Pattern::Char('('),
                    StateAction::new(STATE_INITIAL, Some(emit_left_paren))
                ),
                // 右括号
                TransitionRule::new(
                    Pattern::Char(')'),
                    StateAction::new(STATE_INITIAL, Some(emit_right_paren))
                ),
                // 左方括号
                TransitionRule::new(
                    Pattern::Char('['),
                    StateAction::new(STATE_INITIAL, Some(emit_left_bracket))
                ),
                // 右方括号
                TransitionRule::new(
                    Pattern::Char(']'),
                    StateAction::new(STATE_INITIAL, Some(emit_right_bracket))
                ),
                // 引号
                TransitionRule::new(
                    Pattern::Char('\''),
                    StateAction::new(STATE_INITIAL, Some(emit_quote))
                ),
                // 反引号
                TransitionRule::new(
                    Pattern::Char('`'),
                    StateAction::new(STATE_INITIAL, Some(emit_quasiquote))
                ),
                // 逗号反引号 (,@)
                TransitionRule::new(
                    Pattern::String(",@"),
                    StateAction::new(STATE_INITIAL, Some(emit_unquote_splicing))
                ),
                // 逗号
                TransitionRule::new(
                    Pattern::Char(','),
                    StateAction::new(STATE_INITIAL, Some(emit_unquote))
                ),
                // 点号
                TransitionRule::new(
                    Pattern::Char('.'),
                    StateAction::new(STATE_INITIAL, Some(emit_dot))
                ),
                // 字符串开始
                TransitionRule::new(
                    Pattern::Char('"'),
                    StateAction::new(STATE_STRING, None)
                ),
                // 注释开始
                TransitionRule::new(
                    Pattern::Char(';'),
                    StateAction::new(STATE_COMMENT, None)
                ),
                // 换行符
                TransitionRule::new(
                    Pattern::CharClass(is_newline_char),
                    StateAction::new(STATE_INITIAL, Some(emit_newline))
                ),
                // 空白字符
                TransitionRule::new(
                    Pattern::CharClass(is_whitespace_char),
                    StateAction::new(STATE_WHITESPACE, None)
                ),
                // 向量开始 (#()
                TransitionRule::new(
                    Pattern::String("#("),
                    StateAction::new(STATE_INITIAL, Some(emit_vector_start))
                ),
                // 字符字面量开始 (#\)
                TransitionRule::new(
                    Pattern::String("#\\"),
                    StateAction::new(STATE_CHARACTER, None)
                ),
                // 数字开始（正负号）
                TransitionRule::new(
                    Pattern::CharClass(|c| c == '+' || c == '-'),
                    StateAction::new(STATE_NUMBER_SIGN, None)
                ),
                // 数字开始（数字）
                TransitionRule::new(
                    Pattern::CharClass(is_digit_char),
                    StateAction::new(STATE_NUMBER, None)
                ),
                // 符号开始（但不是数字符号）
                TransitionRule::new(
                    Pattern::CharClass(|c| is_symbol_start_char(c) && c != '+' && c != '-'),
                    StateAction::new(STATE_SYMBOL, None)
                ),
            ],
            
            // STATE_NUMBER (1) 的规则
            vec![
                // 小数点
                TransitionRule::new(
                    Pattern::Char('.'),
                    StateAction::new(STATE_NUMBER_DECIMAL, None)
                ),
                // 科学计数法标记
                TransitionRule::new(
                    Pattern::CharClass(|c| c == 'e' || c == 'E'),
                    StateAction::new(STATE_NUMBER_EXP, None)
                ),
                // 继续收集数字字符
                TransitionRule::new(
                    Pattern::CharClass(is_digit_char),
                    StateAction::new(STATE_NUMBER, None)
                ),
            ],
            
            // STATE_STRING (2) 的规则
            vec![
                // 字符串结束
                TransitionRule::new(
                    Pattern::Char('"'),
                    StateAction::new(STATE_INITIAL, Some(emit_string))
                ),
                // 转义字符
                TransitionRule::new(
                    Pattern::Char('\\'),
                    StateAction::new(STATE_STRING_ESCAPE, None)
                ),
                // 普通字符（除了引号和反斜杠）
                TransitionRule::new(
                    Pattern::CharClass(|c| c != '"' && c != '\\'),
                    StateAction::new(STATE_STRING, None)
                ),
            ],
            
            // STATE_STRING_ESCAPE (3) 的规则
            vec![
                // 任何字符都回到字符串状态
                TransitionRule::new(
                    Pattern::CharClass(|_| true),
                    StateAction::new(STATE_STRING, None)
                ),
            ],
            
            // STATE_SYMBOL (4) 的规则
            vec![
                // 继续收集符号字符
                TransitionRule::new(
                    Pattern::CharClass(is_symbol_char),
                    StateAction::new(STATE_SYMBOL, None)
                ),
            ],
            
            // STATE_COMMENT (5) 的规则
            vec![
                // 注释结束（遇到换行）
                TransitionRule::new(
                    Pattern::CharClass(is_newline_char),
                    StateAction::new(STATE_INITIAL, Some(emit_comment))
                ),
                // 继续收集注释字符
                TransitionRule::new(
                    Pattern::CharClass(|c| !is_newline_char(c)),
                    StateAction::new(STATE_COMMENT, None)
                ),
            ],
            
            // STATE_WHITESPACE (6) 的规则
            vec![
                // 继续收集空白字符
                TransitionRule::new(
                    Pattern::CharClass(is_whitespace_char),
                    StateAction::new(STATE_WHITESPACE, None)
                ),
            ],
            
            // STATE_CHARACTER (7) 的规则
            vec![
                // 收集字符名（如 "space", "newline"）或单个字符
                TransitionRule::new(
                    Pattern::CharClass(|c| c.is_alphabetic()),
                    StateAction::new(STATE_CHARACTER, None)
                ),
                // 单个字符（除了字母以外的可打印字符）
                TransitionRule::new(
                    Pattern::CharClass(|c| !c.is_alphabetic() && !is_delimiter_char(c)),
                    StateAction::new(STATE_INITIAL, Some(emit_character))
                ),
            ],
            
            // STATE_NUMBER_SIGN (8) 的规则 - 处理正负号后面的数字
            vec![
                // 正负号后跟数字
                TransitionRule::new(
                    Pattern::CharClass(is_digit_char),
                    StateAction::new(STATE_NUMBER, None)
                ),
                // 如果正负号后面不是数字，则作为符号处理
            ],
            
            // STATE_NUMBER_DECIMAL (9) 的规则 - 处理小数点后的数字
            vec![
                // 小数点后的数字
                TransitionRule::new(
                    Pattern::CharClass(is_digit_char),
                    StateAction::new(STATE_NUMBER_DECIMAL, None)
                ),
                // 科学计数法标记
                TransitionRule::new(
                    Pattern::CharClass(|c| c == 'e' || c == 'E'),
                    StateAction::new(STATE_NUMBER_EXP, None)
                ),
            ],
            
            // STATE_NUMBER_EXP (10) 的规则 - 处理科学计数法的指数部分
            vec![
                // 指数后的正负号
                TransitionRule::new(
                    Pattern::CharClass(|c| c == '+' || c == '-'),
                    StateAction::new(STATE_NUMBER_EXP_SIGN, None)
                ),
                // 指数后直接跟数字
                TransitionRule::new(
                    Pattern::CharClass(is_digit_char),
                    StateAction::new(STATE_NUMBER_EXP, None)
                ),
            ],
            
            // STATE_NUMBER_EXP_SIGN (11) 的规则 - 处理指数的正负号
            vec![
                // 正负号后跟数字
                TransitionRule::new(
                    Pattern::CharClass(is_digit_char),
                    StateAction::new(STATE_NUMBER_EXP, None)
                ),
            ],
        ],
        
        fallback_rules: vec![
            // STATE_INITIAL 的 fallback
            StateAction::new(STATE_INITIAL, Some(emit_error)),
            // STATE_NUMBER 的 fallback - 数字结束
            StateAction::new(STATE_INITIAL, Some(emit_number)),
            // STATE_STRING 的 fallback - 字符串未结束错误
            StateAction::new(STATE_STRING, Some(emit_error)),
            // STATE_STRING_ESCAPE 的 fallback
            StateAction::new(STATE_STRING, None),
            // STATE_SYMBOL 的 fallback - 符号结束
            StateAction::new(STATE_INITIAL, Some(emit_symbol)),
            // STATE_COMMENT 的 fallback - 注释结束（文件结束）
            StateAction::new(STATE_INITIAL, Some(emit_comment)),
            // STATE_WHITESPACE 的 fallback - 空白字符结束
            StateAction::new(STATE_INITIAL, Some(emit_whitespace)),
            // STATE_CHARACTER 的 fallback - 字符结束
            StateAction::new(STATE_INITIAL, Some(emit_character)),
            // STATE_NUMBER_SIGN 的 fallback - 如果正负号后面不是数字，作为符号处理
            StateAction::new(STATE_INITIAL, Some(emit_symbol)),
            // STATE_NUMBER_DECIMAL 的 fallback - 小数结束
            StateAction::new(STATE_INITIAL, Some(emit_number)),
            // STATE_NUMBER_EXP 的 fallback - 科学计数法结束
            StateAction::new(STATE_INITIAL, Some(emit_number)),
            // STATE_NUMBER_EXP_SIGN 的 fallback - 指数符号必须后跟数字，否则错误
            StateAction::new(STATE_INITIAL, Some(emit_error)),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_number() {
        let pos = 0;
        
        // 测试整数
        let result = emit_number("42", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Integer(42));
        assert_eq!(result.raw_text, "42");
        
        // 测试负整数
        let result = emit_number("-123", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Integer(-123));
        
        // 测试浮点数
        let result = emit_number("3.14", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Float(3.14));
        
        // 测试负浮点数
        let result = emit_number("-123.45", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Float(-123.45));
        
        // 测试科学计数法
        let result = emit_number("1.23e4", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Float(1.23e4));
        
        let result = emit_number("-2.5E-3", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Float(-2.5E-3));
        
        // 测试无效数字
        let result = emit_number("invalid", pos);
        assert!(result.is_err());
    }

    #[test]
    fn test_emit_string() {
        let pos = 0;
        
        let result = emit_string("\"hello\"", pos).unwrap();
        assert_eq!(result.token_type, TokenType::String("hello".to_string()));
        
        let result = emit_string("\"hello\\nworld\"", pos).unwrap();
        assert_eq!(result.token_type, TokenType::String("hello\nworld".to_string()));
        
        let result = emit_string("\"unterminated", pos);
        assert!(result.is_err());
    }

    #[test]
    fn test_emit_symbol() {
        let pos = 0;
        
        let result = emit_symbol("symbol", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Symbol("symbol".to_string()));
        
        let result = emit_symbol("#t", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Boolean(true));
        
        let result = emit_symbol("#f", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Boolean(false));
    }

    #[test]
    fn test_character_predicates() {
        assert!(is_digit_char('0'));
        assert!(is_digit_char('9'));
        assert!(!is_digit_char('a'));
        assert!(!is_digit_char('.'));
        
        assert!(is_number_start_char('0'));
        assert!(is_number_start_char('+'));
        assert!(is_number_start_char('-'));
        assert!(!is_number_start_char('a'));
        
        assert!(is_number_char('0'));
        assert!(is_number_char('.'));
        assert!(is_number_char('e'));
        assert!(is_number_char('E'));
        assert!(is_number_char('+'));
        assert!(is_number_char('-'));
        assert!(!is_number_char('a'));
        
        assert!(is_symbol_start_char('a'));
        assert!(is_symbol_start_char('+'));
        assert!(!is_symbol_start_char('0'));
        
        assert!(is_whitespace_char(' '));
        assert!(is_whitespace_char('\t'));
        assert!(!is_whitespace_char('\n'));
        
        assert!(is_newline_char('\n'));
        assert!(is_newline_char('\r'));
        assert!(!is_newline_char(' '));
    }

    #[test]
    fn test_process_string_escapes() {
        let pos = 0;
        
        let result = process_string_escapes("hello", pos).unwrap();
        assert_eq!(result, "hello");
        
        let result = process_string_escapes("hello\\nworld", pos).unwrap();
        assert_eq!(result, "hello\nworld");
        
        let result = process_string_escapes("quote: \\\"", pos).unwrap();
        assert_eq!(result, "quote: \"");
        
        let result = process_string_escapes("invalid\\x", pos);
        assert!(result.is_err());
    }

    #[test]
    fn test_emit_vector_start() {
        let pos = 5;
        let result = emit_vector_start("#(", pos).unwrap();
        
        assert_eq!(result.token_type, TokenType::VectorStart);
        assert_eq!(result.span.start, pos);
        assert_eq!(result.span.end, pos + 2);
        assert_eq!(result.raw_text, "#(");
    }
}
