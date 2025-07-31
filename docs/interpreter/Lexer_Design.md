# 词法分析器设计

状态：Reviewed

## 概述

词法分析器负责将源代码文本转换为 Token 流，是解释器的第一个处理阶段。采用函数式设计，避免在数据结构上编码业务逻辑。

## 模块职责（功能性需求）

- **词法分析**：将字符流转换为结构化的 Token 序列
- **位置跟踪**：准确记录每个 Token 的源码位置信息
- **错误检测**：识别词法层面的语法错误
- **流式处理**：支持大文件的逐步解析

## 设计目标（非功能性需求）

- **完整语法支持**：支持 R7RS Scheme 标准的所有词法元素
- **精确位置跟踪**：提供字节级和行列级的位置信息
- **错误恢复**：在遇到词法错误时能够继续分析
- **性能优化**：高效的字符流处理和内存使用
- **函数式设计**：纯函数实现，数据与行为分离

## 源代码结构

词法分析器模块按照职责分离的原则组织，采用分层架构设计：

```text
/src/lexer/
├── char_stream.rs    # 字符流抽象和前瞻操作
├── types.rs          # 核心数据类型定义
├── pattern_matcher.rs # 模式匹配器
├── rules.rs          # 状态机规则定义和 Token 生成器
├── engine.rs         # 词法分析执行引擎
└── mod.rs            # 模块接口和公共 API
```

**模块职责说明**：

- **`char_stream.rs`**：封装字符流操作，提供统一的字符前瞻和推进接口
  - `peek()`, `advance()`, `advance_many()` 等字符流操作函数
  - 基于 VecDeque 的高效缓冲机制
  - 支持多字符前瞻的字符流状态管理

- **`pattern_matcher.rs`**：实现基于 CharStream 的模式匹配
  - `match_pattern()` 主要匹配接口
  - `match_char()`, `match_string()`, `match_char_class()` 等具体匹配函数
  - `match_char_class_sequence()` 连续字符类匹配

- **`types.rs`**：定义词法分析器的核心数据结构
  - `TokenType`, `Token`, `LexError`, `Position` 等公共类型
  - `TransitionRule`, `StateAction`, `Pattern` 等状态机类型
  - 类型相关的辅助方法和特征实现

- **`rules.rs`**：集中管理词法规则和 Token 生成逻辑
  - 状态机规则定义 (`SCHEME_STATE_MACHINE`)
  - Token 生成器函数集合 (`emit_*` 函数)
  - 模式匹配函数和字符类判断

- **`engine.rs`**：实现状态机驱动的词法分析核心逻辑
  - `LexerState` 状态管理
  - 状态转移执行逻辑
  - Iterator 特征实现和主要执行循环

- **`mod.rs`**：模块对外接口
  - 导出公共类型和函数
  - `tokenize()` 主要 API 函数
  - 模块级文档和使用示例

## 关键数据类型

### TokenType

使用代数数据类型表示不同的 Token 变体，包含 Trivia Tokens 以支持程序原貌还原：

```rust
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
```

### Token

纯数据结构，仅用于聚合相关数据：

```rust
/// 带位置信息的词法单元 - 纯数据结构
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,             // Token 位置范围
    pub raw_text: String,       // 原始文本内容
}

impl Token {
    /// 创建新的Token
    pub fn new(token_type: TokenType, span: Span, raw_text: String) -> Self {
        Self { token_type, span, raw_text }
    }
    
    /// 从文本和起始位置创建Token
    pub fn from_text(token_type: TokenType, text: &str, start: usize) -> Self {
        let char_count = text.chars().count();
        let span = Span::from_char_range(start, char_count);
        Self::new(token_type, span, text.to_string())
    }
    
    /// 获取Token的长度（字符数）
    pub fn len(&self) -> usize {
        self.span.len()
    }
}
```

### LexError

规范化的词法错误类型：

```rust
/// 词法分析错误 - 统一的错误结构
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    /// 出问题的位置（字符偏移量）
    pub position: usize,
    /// 导致错误的字符
    pub found_char: Option<char>,  // None 表示EOF
    /// 错误的具体原因
    pub reason: LexErrorReason,
    /// 尚未被tokenize的字符串（包括buffer中的内容）
    pub remaining_chars: Box<dyn Iterator<Item = char>>,
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

impl LexError {
    /// 创建新的词法错误
    pub fn new(
        position: usize, 
        found_char: Option<char>, 
        reason: LexErrorReason,
        remaining_chars: Box<dyn Iterator<Item = char>>
    ) -> Self {
        Self { position, found_char, reason, remaining_chars }
    }
    
    /// 获取错误的显示文本
    pub fn found_text(&self) -> String {
        match self.found_char {
            Some(ch) => ch.to_string(),
            None => "EOF".to_string(),
        }
    }
    
    /// 获取错误位置
    pub fn position(&self) -> usize {
        self.position
    }
}
```

### Position

位置信息设计（已弃用，使用字符偏移量代替）：

```rust
/// Token 位置信息 - 已弃用，使用 usize 字符偏移量代替
/// 保留此定义仅用于向后兼容和转换目的
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

impl Position {
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
```

### Span

位置范围信息，基于字符偏移量的左闭右开区间设计：

```rust
/// 源码位置范围 - 基于字符偏移量的左闭右开区间 [start, end)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// 起始位置（字符偏移量，包含）
    pub start: usize,
    /// 结束位置（字符偏移量，不包含）
    pub end: usize,
}

impl Span {
    /// 创建新的 Span
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "Span start must be <= end");
        Self { start, end }
    }
    
    /// 创建单个字符位置的 Span
    pub fn single_char(position: usize) -> Self {
        Self::new(position, position + 1)
    }
    
    /// 创建空 Span（零长度区间）
    pub fn empty(position: usize) -> Self {
        Self::new(position, position)
    }
    
    /// 从文本长度和起始位置创建 Span
    pub fn from_char_range(start: usize, char_count: usize) -> Self {
        Self::new(start, start + char_count)
    }
    
    /// 获取 Span 的长度（字符数）
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// 判断 Span 是否为空
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    
    /// 获取起始位置
    pub fn start(&self) -> usize {
        self.start
    }
    
    /// 获取结束位置
    pub fn end(&self) -> usize {
        self.end
    }
}
}

### 辅助类型方法

```rust
/// 判断 Token 是否为 Trivia（用于过滤）
impl TokenType {
    pub fn is_trivia(&self) -> bool {
        matches!(self, 
            TokenType::Whitespace(_) | 
            TokenType::Newline | 
            TokenType::Comment(_)
        )
    }
}
```

## 核心函数接口（对外接口）

**重要说明**：本节只记录对外暴露的主要接口函数，不包括内部实现函数、私有方法和辅助函数。

### tokenize

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| chars | `I: Iterator<Item = char>` | 字符迭代器，可以是任何产生字符的迭代器 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `impl Iterator<Item = Result<Token, LexError>>` | Token 迭代器，每个元素可能是成功的 Token 或错误 |

## 关键设计问题

### 问题：内部状态管理和迭代器实现策略

词法分析器采用状态机驱动的迭代器适配器模式，使用规则化的状态转移和 Token 生成：

**状态机规则结构设计**：
```rust
/// 状态机转移规则
struct TransitionRule {
    pattern: Pattern,                    // 匹配模式（正则表达式或字符串常量）
    action: StateAction,                 // 状态转移动作
}

enum Pattern {
    Char(char),                         // 单字符匹配
    String(&'static str),               // 字符串常量匹配
    Regex(&'static str),                // 正则表达式匹配
    CharClass(fn(char) -> bool),        // 字符类匹配（如数字、字母）
}

struct StateAction {
    next_state: usize,                  // 转移到的新状态
    emit_token: Option<TokenEmitter>,   // 可选的 Token 生成器
}

type TokenEmitter = Box<dyn Fn(&str, usize) -> Result<Token, LexError>>;

/// 状态机规则集：状态 -> 转移规则列表
struct StateMachine {
    rules: Vec<Vec<TransitionRule>>,    // rules[state_id] = 该状态的规则列表
    fallback_rules: Vec<StateAction>,   // 每个状态的失配处理规则
}
```

**内部状态结构设计**：
```rust
/// 内部词法分析器状态 - 实现细节，不暴露给用户
struct LexerState<I: Iterator<Item = char>> {
    chars: Peekable<I>,                 // 字符流，支持前瞻
    current_pos: usize,                 // 当前字符偏移量位置
    state: usize,                       // 当前状态机状态
    buffer: String,                     // 缓冲的字符串
    state_machine: &'static StateMachine, // 状态机规则集
}
```

**迭代器执行逻辑**：
1. 获取当前状态对应的规则列表
2. 按顺序匹配规则，找到第一个能与 `Peekable<char>` 匹配的规则
3. 执行状态转移：更新状态、推进字符流、更新缓冲区
4. 如果规则包含 `emit_token`，则生成 Token 并返回
5. 否则继续迭代到下一个字符
6. 如果所有规则都不匹配，执行 fallback 规则

**关键优势**：
- **可扩展性**：新增 Token 类型只需添加规则，无需修改核心逻辑
- **可读性**：状态转移规则清晰明确，易于理解和维护
- **性能**：规则按优先级排序，支持快速匹配
- **错误处理**：fallback 规则统一处理异常情况

### 问题：状态操作函数的内部组织

基于状态机驱动的设计，内部函数按照职责分层组织：

**字符流操作层**：
```rust
// 内部实现函数 - 不暴露给用户
fn peek_char<I>(stream: &mut CharStream<I>) -> Option<&char>
fn advance_char<I>(stream: &mut CharStream<I>) -> Option<char>
fn advance_position(pos: &mut usize, ch: char)
fn match_pattern<I>(stream: &mut CharStream<I>, pattern: &Pattern) -> MatchResult
```

**状态机执行层**：
```rust
fn execute_transition<I>(
    state: &mut LexerState<I>, 
    rule: &TransitionRule
) -> Option<Result<Token, LexError>>

fn find_matching_rule<I>(
    state: &LexerState<I>, 
    rules: &[TransitionRule]
) -> Option<&TransitionRule>

fn apply_fallback<I>(
    state: &mut LexerState<I>, 
    fallback: &StateAction
) -> Option<Result<Token, LexError>>
```

**Token 生成器集合**：
```rust
// TokenEmitter 函数的具体实现
fn emit_number(raw_text: &str, position: usize) -> Result<Token, LexError>
fn emit_string(raw_text: &str, position: usize) -> Result<Token, LexError>
fn emit_symbol(raw_text: &str, position: usize) -> Result<Token, LexError>
fn emit_comment(raw_text: &str, position: usize) -> Result<Token, LexError>
```

**状态机定义**：
```rust
// 状态常量定义
const STATE_INITIAL: usize = 0;
const STATE_NUMBER: usize = 1;
const STATE_STRING: usize = 2;
const STATE_SYMBOL: usize = 3;
const STATE_COMMENT: usize = 4;
const STATE_STRING_ESCAPE: usize = 5;

// 预定义的状态机规则集
static SCHEME_STATE_MACHINE: StateMachine = StateMachine {
    rules: vec![
        // STATE_INITIAL 的规则
        vec![
            TransitionRule { 
                pattern: Pattern::Char('('), 
                action: StateAction { 
                    next_state: STATE_INITIAL, 
                    emit_token: Some(Box::new(|raw_text, position| {
                        Ok(Token {
                            token_type: TokenType::LeftParen,
                            span: Span::from_char_range(position, 1),
                            raw_text: raw_text.to_string(),
                        })
                    }))
                }
            },
            TransitionRule { 
                pattern: Pattern::CharClass(|c| c.is_ascii_digit()), 
                action: StateAction { 
                    next_state: STATE_NUMBER, 
                    emit_token: None  // 进入数字状态，继续收集
                }
            },
            TransitionRule { 
                pattern: Pattern::Char('"'), 
                action: StateAction { 
                    next_state: STATE_STRING, 
                    emit_token: None  // 进入字符串状态
                }
            },
            // ... 更多规则
        ],
        // STATE_NUMBER 的规则
        vec![
            TransitionRule { 
                pattern: Pattern::CharClass(|c| c.is_ascii_digit() || c == '.'), 
                action: StateAction { 
                    next_state: STATE_NUMBER, 
                    emit_token: None  // 继续收集数字字符
                }
            },
            // 数字结束的规则（遇到非数字字符）
            TransitionRule { 
                pattern: Pattern::CharClass(|c| c.is_whitespace() || "()[]".contains(c)), 
                action: StateAction { 
                    next_state: STATE_INITIAL, 
                    emit_token: Some(Box::new(emit_number))  // 生成数字Token
                }
            },
        ],
        // 其他状态的规则...
    ],
    fallback_rules: vec![
        // STATE_INITIAL 的 fallback
        StateAction { 
            next_state: STATE_INITIAL, 
            emit_token: Some(Box::new(|raw_text, position| {
                Err(LexError::new(
                    position,
                    raw_text.chars().next(),
                    LexErrorReason::InvalidCharacter,
                    Box::new(std::iter::empty())
                ))
            }))
        },
        // 其他状态的 fallback...
    ],
};
```

**组织策略**：
- **分层设计**：字符操作 → 状态机执行 → Token 生成
- **规则驱动**：核心逻辑通过状态机规则表达，易于修改和扩展
- **函数式设计**：TokenEmitter 为纯函数，便于测试和组合

### 问题：数值解析的精度和类型统一处理
MVP 版本，数值解析精度和底层 Rust 的数值类型精度一致。暂不考虑对大数、超高精度浮点数的支持。

### 问题：Trivia Tokens 的处理策略和性能影响

Trivia Tokens 要生成，是否保存，如何保存，由后续的 Parser 决定。

### 问题：错误恢复的同步点选择和恢复粒度

MVP 暂不考虑，MVP 不做错误恢复

### 问题：大文件流式处理的内存管理策略

MVP 暂不考虑

### 问题：位置信息跟踪的性能开销优化

**新的简化策略**：
- 使用字符偏移量 `usize` 作为唯一的位置标识
- Token 存储起始位置和原始文本，结束位置通过计算得出
- Span 计算简单：`Span::new(start, start + char_count)`
- 避免了复杂的行列计算和 Unicode 字符处理

**性能优势**：
- 位置推进：O(1) 操作，简单的整数加法
- 位置比较：直接的整数比较
- 内存占用：每个位置只需一个 `usize`
- 无需处理换行符、制表符等特殊字符的位置计算

**按需转换策略**：
- 内部使用字符偏移量进行所有计算
- 错误报告时使用 `PositionConverter` 转换为行列信息
- 避免存储冗余的位置信息，减少内存开销



