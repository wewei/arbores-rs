//! 词法分析器核心数据类型定义
//! 
//! 本模块定义了词法分析器使用的所有数据结构，遵循函数式设计原则：
//! - 使用代数数据类型 (ADT) 表示复杂状态
//! - 纯数据结构，不包含业务逻辑方法
//! - 通过独立函数实现行为

use crate::interpreter::lexer::char_stream::CharStream;

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
    pub span: Span,             // Token 位置范围
    pub raw_text: String,       // 原始文本内容
}

/// 词法分析错误 - 统一的错误结构
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    /// 出问题的位置
    pub position: Position,
    /// 导致错误的字符
    pub found_char: Option<char>,  // None 表示EOF
    /// 错误的具体原因
    pub reason: LexErrorReason,
}

/// 词法错误的具体原因
#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorReason {
    /// 无效的数字格式
    InvalidNumber { partial_text: String },
    /// 未终止的字符串
    UnterminatedString,
    /// 无效的转义字符
    InvalidEscape { escape_char: char },
    /// 无效的字符
    InvalidCharacter,
    /// 意外的文件结束
    UnexpectedEof { expected: String },
    /// 其他词法错误
    Other(String),
}

/// Token 位置信息 - 存储在 Token 中
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// 源码位置范围 - 包含起始和结束位置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
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
    pub fn new(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line,
            column,
            byte_offset,
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

    /// 根据文本内容推进位置
    pub fn advance_by_text(&self, text: &str) -> Position {
        let mut pos = *self;
        for ch in text.chars() {
            if ch == '\n' {
                pos.line += 1;
                pos.column = 1;
            } else {
                pos.column += 1;
            }
            pos.byte_offset += ch.len_utf8();
        }
        pos
    }
}

impl Span {
    /// 创建新的Span
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
    
    /// 从文本和起始位置创建Span
    pub fn from_text(text: &str, start: Position) -> Self {
        let end = start.advance_by_text(text);
        Self { start, end }
    }
    
    /// 获取Span的长度（字节数）
    pub fn len(&self) -> usize {
        self.end.byte_offset - self.start.byte_offset
    }
    
    /// 判断Span是否为空
    pub fn is_empty(&self) -> bool {
        self.start.byte_offset == self.end.byte_offset
    }
}

impl Token {
    /// 创建新的Token
    pub fn new(token_type: TokenType, span: Span, raw_text: String) -> Self {
        Self { token_type, span, raw_text }
    }
    
    /// 从文本和起始位置创建Token
    pub fn from_text(token_type: TokenType, text: &str, start: Position) -> Self {
        let span = Span::from_text(text, start);
        Self::new(token_type, span, text.to_string())
    }
    
    /// 获取Token的起始位置
    pub fn start_pos(&self) -> Position {
        self.span.start
    }
    
    /// 获取Token的结束位置
    pub fn end_pos(&self) -> Position {
        self.span.end
    }
}

impl LexError {
    /// 创建新的词法错误
    pub fn new(
        position: Position, 
        found_char: Option<char>, 
        reason: LexErrorReason,
    ) -> Self {
        Self { position, found_char, reason }
    }
    
    /// 获取错误的显示文本
    pub fn found_text(&self) -> String {
        match self.found_char {
            Some(ch) => ch.to_string(),
            None => "EOF".to_string(),
        }
    }
}

impl TokenType {
    /// 判断 Token 是否为 Trivia（用于过滤）
    pub fn is_trivia(&self) -> bool {
        matches!(self, 
            TokenType::Whitespace(_) | 
            TokenType::Newline | 
            TokenType::Comment(_)
        )
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
// 错误处理
// ============================================================================

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            LexErrorReason::InvalidNumber { partial_text } => {
                write!(f, "Invalid number '{}' at line {}, column {}", 
                       partial_text, self.position.line, self.position.column)
            }
            LexErrorReason::UnterminatedString => {
                write!(f, "Unterminated string at line {}, column {}", 
                       self.position.line, self.position.column)
            }
            LexErrorReason::InvalidEscape { escape_char } => {
                write!(f, "Invalid escape character '{}' at line {}, column {}", 
                       escape_char, self.position.line, self.position.column)
            }
            LexErrorReason::InvalidCharacter => {
                let found = self.found_text();
                write!(f, "Invalid character '{}' at line {}, column {}", 
                       found, self.position.line, self.position.column)
            }
            LexErrorReason::UnexpectedEof { expected } => {
                write!(f, "Unexpected end of file (expected '{}') at line {}, column {}", 
                       expected, self.position.line, self.position.column)
            }
            LexErrorReason::Other(msg) => {
                write!(f, "{} at line {}, column {}", 
                       msg, self.position.line, self.position.column)
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
        let pos = Position::new(1, 1, 0);
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
    fn test_position_advance_by_text() {
        let pos = Position::start();
        let text = "hello\nworld";
        
        let new_pos = pos.advance_by_text(text);
        assert_eq!(new_pos.line, 2);
        assert_eq!(new_pos.column, 6); // "world" 有 5 个字符，所以列是 6
        assert_eq!(new_pos.byte_offset, 11);
    }

    #[test]
    fn test_span_creation() {
        let start = Position::start();
        let end = Position::new(1, 6, 5);
        let span = Span::new(start, end);
        
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_from_text() {
        let start = Position::start();
        let text = "hello";
        let span = Span::from_text(text, start);
        
        assert_eq!(span.start, start);
        assert_eq!(span.len(), 5);
        assert_eq!(span.end.column, 6);
    }

    #[test]
    fn test_token_is_trivia() {
        assert!(TokenType::Whitespace(" ".to_string()).is_trivia());
        assert!(TokenType::Newline.is_trivia());
        assert!(TokenType::Comment("comment".to_string()).is_trivia());
        
        assert!(!TokenType::Number(42.0).is_trivia());
        assert!(!TokenType::Symbol("symbol".to_string()).is_trivia());
        assert!(!TokenType::LeftParen.is_trivia());
    }

    #[test]
    fn test_token_creation() {
        let start = Position::start();
        let text = "42";
        let token = Token::from_text(TokenType::Number(42.0), text, start);
        
        assert_eq!(token.token_type, TokenType::Number(42.0));
        assert_eq!(token.start_pos(), start);
        assert_eq!(token.raw_text, "42");
        assert_eq!(token.span.len(), 2);
    }

    #[test]
    fn test_lex_error_creation() {
        let pos = Position::start();
        let error = LexError::new(
            pos,
            Some('!'),
            LexErrorReason::InvalidCharacter,
        );
        
        assert_eq!(error.position, pos);
        assert_eq!(error.found_char, Some('!'));
        assert_eq!(error.found_text(), "!");
    }
}
