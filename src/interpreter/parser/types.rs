//! 语法分析器核心数据类型定义
//! 
//! 本模块定义了语法分析器的核心数据结构，遵循函数式设计原则：
//! - 使用代数数据类型 (ADT) 表示复杂状态
//! - 纯数据结构，不包含业务逻辑方法
//! - 通过独立函数实现行为

use std::rc::Rc;

use crate::interpreter::lexer::types::{LexError, Token, Span};

// ============================================================================
// AST 核心数据结构
// ============================================================================

/// S表达式 - 带源追踪的Scheme核心数据结构
/// 使用 Rc 支持宏展开时的有向无环图结构
#[derive(Debug, PartialEq)]
pub struct SExpr {
    /// S表达式的具体内容
    pub content: SExprContent,
    /// 源代码位置信息
    pub span: Rc<Span>,
}

/// S表达式内容 - 表示所有可能的Scheme语法构造
#[derive(Debug, Clone, PartialEq)]
pub enum SExprContent {
    /// 原子值（数字、字符串、布尔值、符号等）
    Atom(Value),
    /// 列表结构 (car . cdr)
    Cons { car: Rc<SExpr>, cdr: Rc<SExpr> },
    /// 空列表
    Nil,
    /// 向量（数组）
    Vector(Vec<Rc<SExpr>>),
}

// ============================================================================
// 错误类型定义
// ============================================================================

/// 语法解析错误 - 使用 enum 表示不同错误情况
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// 意外的 token
    UnexpectedToken {
        found: Token,
        reason: UnexpectedTokenReason,
    },
    /// 词法错误传播
    LexError(LexError),
}

/// 意外Token的具体原因
#[derive(Debug, Clone, PartialEq)]
pub enum UnexpectedTokenReason {
    /// 期望特定类型的token
    Expected(String),
    /// 意外的文件结束
    UnexpectedEof { expected: String },
    /// 未终止的列表
    UnterminatedList {
        start_token: Token, // 列表开始的 '(' token
    },
    /// 未终止的向量
    UnterminatedVector {
        start_token: Token, // 向量开始的 '#(' token
    },
    /// 无效的点对列表
    InvalidDottedList {
        dot_token: Token,        // 点号的位置
        context: DottedListError, // 具体的错误类型
    },
    /// 其他语法错误
    Other(String),
}

/// 点对列表的具体错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum DottedListError {
    /// 点号位置错误（如在开头、连续点号等）
    InvalidDotPosition,
    /// 点号后缺少元素
    MissingTailElement,
    /// 点号后有多个元素（应该只有一个）
    MultipleTailElements,
    /// 点号前没有足够的元素
    InsufficientElements,
}

/// 引用类型
#[derive(Debug, Clone, PartialEq)]
pub enum QuoteType {
    Quote,           // '
    Quasiquote,      // `
    Unquote,         // ,
    UnquoteSplicing, // ,@
}

// ============================================================================
// 结果类型定义
// ============================================================================

/// 解析结果类型 - 包含S表达式序列或错误
pub type ParseResult = Result<Vec<SExpr>, ParseError>;

/// Parser的完整输出结构
#[derive(Debug)]
pub struct ParseOutput {
    /// 解析结果（AST或错误）
    pub result: ParseResult,
    /// 重建的源代码文本
    pub source_text: String,
}

// ============================================================================
// 构造函数 - 遵循函数式规范，仅提供简单的数据构造
// ============================================================================

impl SExpr {
    /// 创建带位置信息的S表达式
    pub fn with_span(content: SExprContent, span: Span) -> Self {
        Self {
            content,
            span: Rc::new(span),
        }
    }
    
    /// 创建不带位置信息的S表达式（用于运行时计算结果）
    pub fn without_span(content: SExprContent) -> Self {
        Self {
            content,
            span: Rc::new(Span::empty(0)), // 使用空span
        }
    }
    
    /// 继承位置信息创建新的S表达式
    pub fn inherit_span(&self, content: SExprContent) -> Self {
        Self {
            content,
            span: self.span.clone(),
        }
    }
}

impl ParseError {
    /// 创建意外token错误
    pub fn unexpected_token(found: Token, reason: UnexpectedTokenReason) -> Self {
        Self::UnexpectedToken { found, reason }
    }
    
    /// 创建意外EOF错误
    pub fn unexpected_eof() -> Self {
        // 创建一个空的EOF token作为占位符
        let eof_token = Token::new(
            crate::interpreter::lexer::types::TokenType::Eof,
            Span::empty(0),
            String::new(),
        );
        Self::UnexpectedToken {
            found: eof_token,
            reason: UnexpectedTokenReason::UnexpectedEof { 
                expected: "expression".to_string() 
            },
        }
    }
    
    /// 创建无效点对列表错误
    pub fn invalid_dotted_list(dot_token: Token, context: DottedListError) -> Self {
        Self::UnexpectedToken {
            found: dot_token.clone(),
            reason: UnexpectedTokenReason::InvalidDottedList {
                dot_token,
                context,
            },
        }
    }
    
    /// 创建未终止列表错误
    pub fn unterminated_list(start_token: Token) -> Self {
        Self::UnexpectedToken {
            found: start_token.clone(),
            reason: UnexpectedTokenReason::UnterminatedList { start_token },
        }
    }
    
    /// 创建未终止向量错误
    pub fn unterminated_vector(start_token: Token) -> Self {
        Self::UnexpectedToken {
            found: start_token.clone(),
            reason: UnexpectedTokenReason::UnterminatedVector { start_token },
        }
    }
}

impl QuoteType {
    /// 获取引用类型对应的符号名称
    pub fn symbol_name(&self) -> &'static str {
        match self {
            QuoteType::Quote => "quote",
            QuoteType::Quasiquote => "quasiquote",
            QuoteType::Unquote => "unquote",
            QuoteType::UnquoteSplicing => "unquote-splicing",
        }
    }
}

impl ParseOutput {
    /// 创建成功的解析输出
    pub fn success(expressions: Vec<SExpr>, source_text: String) -> Self {
        Self {
            result: Ok(expressions),
            source_text,
        }
    }
    
    /// 创建失败的解析输出
    pub fn error(error: ParseError, source_text: String) -> Self {
        Self {
            result: Err(error),
            source_text,
        }
    }
}

// ============================================================================
// Display 实现 - 用于错误报告
// ============================================================================

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { found, reason } => {
                write!(f, "Unexpected token at position {}: ", found.span.start)?;
                match reason {
                    UnexpectedTokenReason::Expected(expected) => {
                        write!(f, "expected {}, found {:?}", expected, found.token_type)
                    },
                    UnexpectedTokenReason::UnexpectedEof { expected } => {
                        write!(f, "unexpected end of file, expected {}", expected)
                    },
                    UnexpectedTokenReason::UnterminatedList { .. } => {
                        write!(f, "unterminated list")
                    },
                    UnexpectedTokenReason::UnterminatedVector { .. } => {
                        write!(f, "unterminated vector")
                    },
                    UnexpectedTokenReason::InvalidDottedList { context, .. } => {
                        write!(f, "invalid dotted list: {:?}", context)
                    },
                    UnexpectedTokenReason::Other(msg) => {
                        write!(f, "{}", msg)
                    },
                }
            },
            ParseError::LexError(lex_error) => {
                write!(f, "Lexical error: {:?}", lex_error)
            },
        }
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// Value 类型定义 - Scheme 的基础数据类型
// ============================================================================

/// Scheme 值类型 - 表示所有可能的 Scheme 数据类型
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 数字（统一使用 f64）
    Number(f64),
    /// 字符串
    String(String),
    /// 字符
    Character(char),
    /// 布尔值
    Boolean(bool),
    /// 符号（标识符）
    Symbol(String),
}
