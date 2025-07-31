//! 词法分析器核心数据类型定义
//! 
//! 本模块定义了词法分析器使用的所有数据结构，遵循函数式设计原则：
//! - 使用代数数据类型 (ADT) 表示复杂状态
//! - 纯数据结构，不包含业务逻辑方法
//! - 通过独立函数实现行为

use crate::lexer::char_stream::CharStream;

/// 词法单元类型 - 使用 enum 表示不同的 Token 类型
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 字面量值
    Number(f64),
    String(String),
    Character(char),
    Boolean(bool),
    
    // 标识符和符号
    Symbol(String),
    
    // 分隔符
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    
    // 特殊符号
    Quote,          // '
    Quasiquote,     // `
    Unquote,        // ,
    UnquoteSplicing, // ,@
    Dot,            // .
    
    // Trivia Tokens (用于还原程序原貌)
    Whitespace(String),     // 空格、制表符等
    Newline,               // 换行符
    Comment(String),       // 注释内容
    
    // 控制符号
    Eof,
}

/// 带位置信息的词法单元 - 纯数据结构
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Position,  // Token 起始位置
    pub raw_text: String,    // 原始文本，可用于计算结束位置
}

/// 词法分析错误 - 使用 enum 表示不同错误情况
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    InvalidNumber { text: String, position: Position },
    UnterminatedString { position: Position },
    InvalidEscape { character: char, position: Position },
    InvalidCharacter { text: String, position: Position },
    UnexpectedEof { position: Position },
}

/// Token 位置信息 - 存储在 Token 中
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// 状态机匹配模式
#[derive(Debug, Clone)]
pub enum Pattern {
    Char(char),                         // 单字符匹配
    String(&'static str),               // 字符串常量匹配
    CharClass(fn(char) -> bool),        // 字符类匹配（如数字、字母）
}

/// 状态机转移动作
#[derive(Debug, Clone)]
pub struct StateAction {
    pub next_state: usize,                  // 转移到的新状态
    pub emit_token: Option<TokenEmitter>,   // 可选的 Token 生成器
}

/// 状态机转移规则
#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub pattern: Pattern,                    // 匹配模式
    pub action: StateAction,                 // 状态转移动作
}

/// Token 生成器函数类型
pub type TokenEmitter = fn(&str, Position) -> Result<Token, LexError>;

/// 状态机规则集：状态 -> 转移规则列表
#[derive(Debug, Clone)]
pub struct StateMachine {
    pub rules: Vec<Vec<TransitionRule>>,    // rules[state_id] = 该状态的规则列表
    pub fallback_rules: Vec<StateAction>,   // 每个状态的失配处理规则
}

/// 内部词法分析器状态 - 实现细节，不暴露给用户
pub struct LexerState<I: Iterator<Item = char>> {
    pub chars: CharStream<I>,               // 字符流，支持前瞻
    pub current_pos: Position,              // 当前位置信息
    pub state: usize,                       // 当前状态机状态
    pub buffer: String,                     // 缓冲的字符串
    pub state_machine: &'static StateMachine, // 状态机规则集
}

// ============================================================================
// 构造函数 - 遵循函数式规范，仅提供简单的数据构造
// ============================================================================

impl Position {
    /// 创建新的位置信息
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            byte_offset: 0,
        }
    }

    /// 创建起始位置
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            byte_offset: 0,
        }
    }
}

impl Token {
    /// 创建新的 Token
    pub fn new(token_type: TokenType, position: Position, raw_text: String) -> Self {
        Self {
            token_type,
            position,
            raw_text,
        }
    }
}

impl StateAction {
    /// 创建状态转移动作
    pub fn new(next_state: usize, emit_token: Option<TokenEmitter>) -> Self {
        Self {
            next_state,
            emit_token,
        }
    }
}

impl TransitionRule {
    /// 创建转移规则
    pub fn new(pattern: Pattern, action: StateAction) -> Self {
        Self {
            pattern,
            action,
        }
    }
}

impl<I: Iterator<Item = char>> LexerState<I> {
    /// 创建新的词法分析器状态
    pub fn new(chars: I, state_machine: &'static StateMachine) -> Self {
        Self {
            chars: CharStream::new(chars),
            current_pos: Position::start(),
            state: 0, // 从初始状态开始
            buffer: String::new(),
            state_machine,
        }
    }
}

// ============================================================================
// 辅助类型方法 - 纯函数实现
// ============================================================================

/// 判断 Token 是否为 Trivia（用于过滤）
pub fn is_trivia_token(token_type: &TokenType) -> bool {
    matches!(token_type, 
        TokenType::Whitespace(_) | 
        TokenType::Newline | 
        TokenType::Comment(_)
    )
}

/// 根据文本内容推进位置
pub fn advance_position_by_text(pos: Position, text: &str) -> Position {
    let mut new_pos = pos;
    for ch in text.chars() {
        if ch == '\n' {
            new_pos.line += 1;
            new_pos.column = 1;
        } else {
            new_pos.column += 1;
        }
        new_pos.byte_offset += ch.len_utf8();
    }
    new_pos
}

/// 推进位置一个字符
pub fn advance_position_by_char(pos: Position, ch: char) -> Position {
    let mut new_pos = pos;
    if ch == '\n' {
        new_pos.line += 1;
        new_pos.column = 1;
    } else {
        new_pos.column += 1;
    }
    new_pos.byte_offset += ch.len_utf8();
    new_pos
}

// ============================================================================
// 错误处理辅助函数
// ============================================================================

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::InvalidNumber { text, position } => {
                write!(f, "Invalid number '{}' at line {}, column {}", 
                       text, position.line, position.column)
            }
            LexError::UnterminatedString { position } => {
                write!(f, "Unterminated string at line {}, column {}", 
                       position.line, position.column)
            }
            LexError::InvalidEscape { character, position } => {
                write!(f, "Invalid escape character '{}' at line {}, column {}", 
                       character, position.line, position.column)
            }
            LexError::InvalidCharacter { text, position } => {
                write!(f, "Invalid character '{}' at line {}, column {}", 
                       text, position.line, position.column)
            }
            LexError::UnexpectedEof { position } => {
                write!(f, "Unexpected end of file at line {}, column {}", 
                       position.line, position.column)
            }
        }
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(1, 1);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.byte_offset, 0);
    }

    #[test]
    fn test_position_start() {
        let pos = Position::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.byte_offset, 0);
    }

    #[test]
    fn test_advance_position_by_char() {
        let pos = Position::start();
        
        // 普通字符
        let pos2 = advance_position_by_char(pos, 'a');
        assert_eq!(pos2.line, 1);
        assert_eq!(pos2.column, 2);
        assert_eq!(pos2.byte_offset, 1);
        
        // 换行符
        let pos3 = advance_position_by_char(pos2, '\n');
        assert_eq!(pos3.line, 2);
        assert_eq!(pos3.column, 1);
        assert_eq!(pos3.byte_offset, 2);
    }

    #[test]
    fn test_advance_position_by_text() {
        let pos = Position::start();
        let text = "hello\nworld";
        
        let new_pos = advance_position_by_text(pos, text);
        assert_eq!(new_pos.line, 2);
        assert_eq!(new_pos.column, 6); // "world" 有 5 个字符，所以列是 6
        assert_eq!(new_pos.byte_offset, 11);
    }

    #[test]
    fn test_is_trivia_token() {
        assert!(is_trivia_token(&TokenType::Whitespace(" ".to_string())));
        assert!(is_trivia_token(&TokenType::Newline));
        assert!(is_trivia_token(&TokenType::Comment("comment".to_string())));
        
        assert!(!is_trivia_token(&TokenType::Number(42.0)));
        assert!(!is_trivia_token(&TokenType::Symbol("symbol".to_string())));
        assert!(!is_trivia_token(&TokenType::LeftParen));
    }

    #[test]
    fn test_token_creation() {
        let pos = Position::start();
        let token = Token::new(
            TokenType::Number(42.0),
            pos,
            "42".to_string()
        );
        
        assert_eq!(token.token_type, TokenType::Number(42.0));
        assert_eq!(token.position, pos);
        assert_eq!(token.raw_text, "42");
    }
}
