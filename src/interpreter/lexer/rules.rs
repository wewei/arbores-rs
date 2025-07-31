//! 状态机规则定义和 Token 生成器
//! 
//! 本模块包含词法分析器的状态机规则定义和各种 Token 生成器函数。
//! 遵循函数式设计原则，所有生成器都是纯函数。

use crate::interpreter::lexer::types::{
    Token, TokenType, LexError, Position, Pattern, StateAction, TransitionRule, StateMachine
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

// ============================================================================
// Token 生成器函数 - 纯函数实现
// ============================================================================

/// 生成数字 Token
pub fn emit_number(raw_text: &str, position: Position) -> Result<Token, LexError> {
    match raw_text.parse::<f64>() {
        Ok(value) => Ok(Token::new(
            TokenType::Number(value),
            position,
            raw_text.to_string(),
        )),
        Err(_) => Err(LexError::InvalidNumber {
            text: raw_text.to_string(),
            position,
        }),
    }
}

/// 生成字符串 Token
pub fn emit_string(raw_text: &str, position: Position) -> Result<Token, LexError> {
    // 去掉首尾的引号
    if raw_text.len() < 2 || !raw_text.starts_with('"') || !raw_text.ends_with('"') {
        return Err(LexError::UnterminatedString { position });
    }
    
    let content = &raw_text[1..raw_text.len()-1];
    
    // 处理转义序列
    let processed_content = process_string_escapes(content, position)?;
    
    Ok(Token::new(
        TokenType::String(processed_content),
        position,
        raw_text.to_string(),
    ))
}

/// 生成符号 Token
pub fn emit_symbol(raw_text: &str, position: Position) -> Result<Token, LexError> {
    // 检查是否为布尔值
    match raw_text {
        "#t" | "#true" => Ok(Token::new(
            TokenType::Boolean(true),
            position,
            raw_text.to_string(),
        )),
        "#f" | "#false" => Ok(Token::new(
            TokenType::Boolean(false),
            position,
            raw_text.to_string(),
        )),
        _ => Ok(Token::new(
            TokenType::Symbol(raw_text.to_string()),
            position,
            raw_text.to_string(),
        )),
    }
}

/// 生成注释 Token
pub fn emit_comment(raw_text: &str, position: Position) -> Result<Token, LexError> {
    // 去掉开头的分号
    let content = raw_text.strip_prefix(';').unwrap_or(raw_text);
    
    Ok(Token::new(
        TokenType::Comment(content.to_string()),
        position,
        raw_text.to_string(),
    ))
}

/// 生成空白字符 Token
pub fn emit_whitespace(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::Whitespace(raw_text.to_string()),
        position,
        raw_text.to_string(),
    ))
}

/// 生成换行符 Token
pub fn emit_newline(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::Newline,
        position,
        raw_text.to_string(),
    ))
}

/// 生成左括号 Token
pub fn emit_left_paren(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::LeftParen,
        position,
        raw_text.to_string(),
    ))
}

/// 生成右括号 Token
pub fn emit_right_paren(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::RightParen,
        position,
        raw_text.to_string(),
    ))
}

/// 生成左方括号 Token
pub fn emit_left_bracket(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::LeftBracket,
        position,
        raw_text.to_string(),
    ))
}

/// 生成右方括号 Token
pub fn emit_right_bracket(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::RightBracket,
        position,
        raw_text.to_string(),
    ))
}

/// 生成引号 Token
pub fn emit_quote(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::Quote,
        position,
        raw_text.to_string(),
    ))
}

/// 生成反引号 Token
pub fn emit_quasiquote(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::Quasiquote,
        position,
        raw_text.to_string(),
    ))
}

/// 生成逗号反引号 Token
pub fn emit_unquote_splicing(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::UnquoteSplicing,
        position,
        raw_text.to_string(),
    ))
}

/// 生成逗号 Token
pub fn emit_unquote(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::Unquote,
        position,
        raw_text.to_string(),
    ))
}

/// 生成点号 Token
pub fn emit_dot(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::Dot,
        position,
        raw_text.to_string(),
    ))
}

/// 生成字符 Token
pub fn emit_character(raw_text: &str, position: Position) -> Result<Token, LexError> {
    // 字符字面量格式：#\字符 或 #\字符名
    if !raw_text.starts_with("#\\") {
        return Err(LexError::InvalidCharacter {
            text: raw_text.to_string(),
            position,
        });
    }
    
    let char_part = &raw_text[2..];
    let character = match char_part {
        "space" => ' ',
        "newline" => '\n',
        "tab" => '\t',
        "return" => '\r',
        c if c.chars().count() == 1 => c.chars().next().unwrap(),
        _ => return Err(LexError::InvalidCharacter {
            text: raw_text.to_string(),
            position,
        }),
    };
    
    Ok(Token::new(
        TokenType::Character(character),
        position,
        raw_text.to_string(),
    ))
}

/// 生成文件结束 Token
pub fn emit_eof(_raw_text: &str, position: Position) -> Result<Token, LexError> {
    Ok(Token::new(
        TokenType::Eof,
        position,
        "".to_string(),
    ))
}

/// 生成错误 Token
pub fn emit_error(raw_text: &str, position: Position) -> Result<Token, LexError> {
    Err(LexError::InvalidCharacter {
        text: raw_text.to_string(),
        position,
    })
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 处理字符串中的转义序列
fn process_string_escapes(content: &str, position: Position) -> Result<String, LexError> {
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
                    return Err(LexError::InvalidEscape {
                        character: escape_char,
                        position,
                    });
                }
                None => {
                    return Err(LexError::UnterminatedString { position });
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

/// 判断是否为数字字符（包括小数点）
pub fn is_number_char(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+'
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
                // 字符字面量开始 (#\)
                TransitionRule::new(
                    Pattern::String("#\\"),
                    StateAction::new(STATE_CHARACTER, None)
                ),
                // 数字开始
                TransitionRule::new(
                    Pattern::CharClass(|c| c.is_ascii_digit()),
                    StateAction::new(STATE_NUMBER, None)
                ),
                // 符号开始
                TransitionRule::new(
                    Pattern::CharClass(is_symbol_start_char),
                    StateAction::new(STATE_SYMBOL, None)
                ),
            ],
            
            // STATE_NUMBER (1) 的规则
            vec![
                // 继续收集数字字符
                TransitionRule::new(
                    Pattern::CharClass(is_number_char),
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
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_number() {
        let pos = Position::start();
        
        let result = emit_number("42", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Number(42.0));
        assert_eq!(result.raw_text, "42");
        
        let result = emit_number("3.14", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Number(3.14));
        
        let result = emit_number("invalid", pos);
        assert!(result.is_err());
    }

    #[test]
    fn test_emit_string() {
        let pos = Position::start();
        
        let result = emit_string("\"hello\"", pos).unwrap();
        assert_eq!(result.token_type, TokenType::String("hello".to_string()));
        
        let result = emit_string("\"hello\\nworld\"", pos).unwrap();
        assert_eq!(result.token_type, TokenType::String("hello\nworld".to_string()));
        
        let result = emit_string("\"unterminated", pos);
        assert!(result.is_err());
    }

    #[test]
    fn test_emit_symbol() {
        let pos = Position::start();
        
        let result = emit_symbol("symbol", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Symbol("symbol".to_string()));
        
        let result = emit_symbol("#t", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Boolean(true));
        
        let result = emit_symbol("#f", pos).unwrap();
        assert_eq!(result.token_type, TokenType::Boolean(false));
    }

    #[test]
    fn test_character_predicates() {
        assert!(is_number_char('0'));
        assert!(is_number_char('.'));
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
        let pos = Position::start();
        
        let result = process_string_escapes("hello", pos).unwrap();
        assert_eq!(result, "hello");
        
        let result = process_string_escapes("hello\\nworld", pos).unwrap();
        assert_eq!(result, "hello\nworld");
        
        let result = process_string_escapes("quote: \\\"", pos).unwrap();
        assert_eq!(result, "quote: \"");
        
        let result = process_string_escapes("invalid\\x", pos);
        assert!(result.is_err());
    }
}
