use std::fmt;
use crate::types::Position;

/// 带位置信息的 Token
#[derive(Debug, Clone, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub position: Position,
}

impl LocatedToken {
    pub fn new(token: Token, position: Position) -> Self {
        LocatedToken { token, position }
    }
}

/// Token 类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 分隔符
    LeftParen,          // (
    RightParen,         // )
    Quote,              // '
    Quasiquote,         // `
    Unquote,            // ,
    UnquoteSplicing,    // ,@
    
    // 字面量
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    Boolean(bool),
    
    // 特殊
    Dot,                // .
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Quote => write!(f, "'"),
            Token::Quasiquote => write!(f, "`"),
            Token::Unquote => write!(f, ","),
            Token::UnquoteSplicing => write!(f, ",@"),
            Token::Integer(n) => write!(f, "{n}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::String(s) => write!(f, "\"{s}\""),
            Token::Symbol(s) => write!(f, "{s}"),
            Token::Boolean(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Token::Dot => write!(f, "."),
            Token::EOF => write!(f, "<EOF>"),
        }
    }
}

/// 词法分析器
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    /// 创建新的词法分析器
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();
        
        Lexer {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            column: 1,
        }
    }

    /// 获取当前位置
    #[allow(dead_code)]
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    /// 移动到下一个字符
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// 跳过注释（以 ; 开头的行）
    fn skip_comment(&mut self) {
        if self.current_char == Some(';') {
            while let Some(ch) = self.current_char {
                self.advance();
                if ch == '\n' {
                    break;
                }
            }
        }
    }

    /// 读取数字（整数或浮点数）
    fn read_number(&mut self) -> Token {
        let mut number_str = String::new();
        let mut is_float = false;

        // 处理负号
        if self.current_char == Some('-') {
            number_str.push('-');
            self.advance();
        }

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(number_str.parse().unwrap_or(0.0))
        } else {
            Token::Integer(number_str.parse().unwrap_or(0))
        }
    }

    /// 读取字符串字面量
    fn read_string(&mut self) -> Result<Token, String> {
        let mut string_val = String::new();
        self.advance(); // 跳过开始的引号

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // 跳过结束的引号
                return Ok(Token::String(string_val));
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => string_val.push('\n'),
                    Some('t') => string_val.push('\t'),
                    Some('r') => string_val.push('\r'),
                    Some('\\') => string_val.push('\\'),
                    Some('"') => string_val.push('"'),
                    Some(c) => string_val.push(c),
                    None => return Err("Unexpected end of input in string".to_string()),
                }
                self.advance();
            } else {
                string_val.push(ch);
                self.advance();
            }
        }

        Err("Unterminated string".to_string())
    }

    /// 读取符号或关键字
    fn read_symbol(&mut self) -> Token {
        let mut symbol = String::new();

        while let Some(ch) = self.current_char {
            // 根据 Scheme R5RS 标准，标识符可以包含：
            // 字母、数字和特殊字符: ! $ % & * + - . / : < = > ? @ ^ _ ~
            if ch.is_alphanumeric() || "!$%&*+-./:<=>?@^_~#".contains(ch) {
                symbol.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // 检查是否为布尔值
        match symbol.as_str() {
            "#t" => Token::Boolean(true),
            "#f" => Token::Boolean(false),
            _ => Token::Symbol(symbol),
        }
    }

    /// 获取下一个 token
    pub fn next_token(&mut self) -> Result<Token, String> {
        loop {
            match self.current_char {
                None => return Ok(Token::EOF),
                Some(ch) if ch.is_whitespace() => self.skip_whitespace(),
                Some(';') => self.skip_comment(),
                Some('(') => {
                    self.advance();
                    return Ok(Token::LeftParen);
                },
                Some(')') => {
                    self.advance();
                    return Ok(Token::RightParen);
                },
                Some('\'') => {
                    self.advance();
                    return Ok(Token::Quote);
                },
                Some('`') => {
                    self.advance();
                    return Ok(Token::Quasiquote);
                },
                Some(',') => {
                    self.advance();
                    if self.current_char == Some('@') {
                        self.advance();
                        return Ok(Token::UnquoteSplicing);
                    } else {
                        return Ok(Token::Unquote);
                    }
                },
                Some('.') => {
                    self.advance();
                    return Ok(Token::Dot);
                },
                Some('"') => return self.read_string(),
                Some(ch) if ch.is_ascii_digit() => return Ok(self.read_number()),
                Some('-') => {
                    // 检查 '-' 后面是否是数字
                    if let Some(next_char) = self.input.get(self.position + 1) {
                        if next_char.is_ascii_digit() {
                            return Ok(self.read_number());
                        }
                    }
                    return Ok(self.read_symbol());
                },
                Some(ch) if ch.is_alphabetic() || "+*/<>=!?_#".contains(ch) => {
                    return Ok(self.read_symbol());
                },
                Some(ch) => {
                    return Err(format!("Unexpected character: {ch}"));
                }
            }
        }
    }

    /// 获取所有 tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::EOF);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }

    /// 获取下一个 token 的位置（跳过空白字符后的位置）
    fn next_token_position(&mut self) -> Position {
        // 跳过空白字符和注释
        loop {
            match self.current_char {
                None => break,
                Some(ch) if ch.is_whitespace() => self.advance(),
                Some(';') => {
                    while let Some(ch) = self.current_char {
                        self.advance();
                        if ch == '\n' {
                            break;
                        }
                    }
                },
                _ => break,
            }
        }
        Position::new(self.line, self.column)
    }

    /// 生成带位置信息的token列表
    pub fn tokenize_with_positions(&mut self) -> Result<Vec<LocatedToken>, String> {
        let mut tokens = Vec::new();
        
        loop {
            // 先获取 token 的正确位置（跳过空白字符后）
            let pos = self.next_token_position();
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::EOF);
            tokens.push(LocatedToken::new(token, pos));
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let mut lexer = Lexer::new("(+ 1 2)");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::LeftParen,
            Token::Symbol("+".to_string()),
            Token::Integer(1),
            Token::Integer(2),
            Token::RightParen,
            Token::EOF,
        ]);
    }

    #[test]
    fn test_lexer_string() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::String("hello world".to_string()),
            Token::EOF,
        ]);
    }

    #[test]
    fn test_lexer_boolean() {
        let mut lexer = Lexer::new("#t #f");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Boolean(true),
            Token::Boolean(false),
            Token::EOF,
        ]);
    }

    #[test]
    fn test_lexer_quote() {
        let mut lexer = Lexer::new("'(a b c)");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Quote,
            Token::LeftParen,
            Token::Symbol("a".to_string()),
            Token::Symbol("b".to_string()),
            Token::Symbol("c".to_string()),
            Token::RightParen,
            Token::EOF,
        ]);
    }

    #[test]
    fn test_lexer_negative_numbers() {
        let mut lexer = Lexer::new("-5 -3.14 - (- 1 2)");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Integer(-5),
            Token::Float(-3.14),
            Token::Symbol("-".to_string()),
            Token::LeftParen,
            Token::Symbol("-".to_string()),
            Token::Integer(1),
            Token::Integer(2),
            Token::RightParen,
            Token::EOF,
        ]);
    }

    #[test]
    fn test_lexer_colon_in_symbols() {
        let mut lexer = Lexer::new("arb:create arb:search my:var test:func:");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Symbol("arb:create".to_string()),
            Token::Symbol("arb:search".to_string()),
            Token::Symbol("my:var".to_string()),
            Token::Symbol("test:func:".to_string()),
            Token::EOF,
        ]);
    }

    #[test]
    fn test_lexer_extended_symbol_chars() {
        // 测试 R5RS 标准允许的特殊字符
        let mut lexer = Lexer::new("func! var$ mod% obj& mul* add+ sub- field. div/ ns:name lt< eq= gt> pred? email@ pow^ under_ tilde~");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Symbol("func!".to_string()),
            Token::Symbol("var$".to_string()),
            Token::Symbol("mod%".to_string()),
            Token::Symbol("obj&".to_string()),
            Token::Symbol("mul*".to_string()),
            Token::Symbol("add+".to_string()),
            Token::Symbol("sub-".to_string()),
            Token::Symbol("field.".to_string()),
            Token::Symbol("div/".to_string()),
            Token::Symbol("ns:name".to_string()),
            Token::Symbol("lt<".to_string()),
            Token::Symbol("eq=".to_string()),
            Token::Symbol("gt>".to_string()),
            Token::Symbol("pred?".to_string()),
            Token::Symbol("email@".to_string()),
            Token::Symbol("pow^".to_string()),
            Token::Symbol("under_".to_string()),
            Token::Symbol("tilde~".to_string()),
            Token::EOF,
        ]);
    }
}
