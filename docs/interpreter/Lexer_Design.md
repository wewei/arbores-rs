# 词法分析器设计

状态：Draft-1

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
    pub span: Span,
    pub raw_text: String,
}
```

### LexError

错误类型的代数数据类型：

```rust
/// 词法分析错误 - 使用 enum 表示不同错误情况
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    InvalidNumber { text: String, position: Position },
    UnterminatedString { position: Position },
    InvalidEscape { character: char, position: Position },
    InvalidCharacter { text: String, position: Position },
    IoError(std::io::Error),
    UnexpectedEof { position: Position },
}
```

### Span 和 Position

位置和范围信息：

```rust
/// 位置和范围信息
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}
```

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

### 问题：接口设计策略 - 泛型 vs 具体类型

词法分析器的接口设计需要在简洁性和灵活性之间权衡：

**采用的泛型接口方案**：
```rust
pub fn tokenize<I>(chars: I) -> impl Iterator<Item = Result<Token, LexError>>
where I: Iterator<Item = char>
```

**替代的具体类型接口方案**：
```rust
pub fn tokenize(input: &str) -> Vec<Result<Token, LexError>>
pub fn tokenize_reader<R: Read>(reader: R) -> Result<Vec<Token>, Vec<LexError>>
```

**选择泛型接口的理由**：
- 支持任意字符迭代器输入（`&str::chars()`, `BufReader` 等）
- 惰性求值，支持流式处理大文件
- 零成本抽象，编译时优化
- 函数式设计，便于组合和链式操作

**便捷函数处理策略**：
- 核心接口保持泛型设计的纯净性
- 便捷函数（如 `tokenize_string`, `tokenize_with_trivia`）作为独立模块提供
- 用户可根据需要选择合适的抽象层次

TODO

### 问题：内部状态管理和迭代器实现策略

词法分析器的内部实现需要维护状态来跟踪位置信息、缓冲字符和错误恢复。主要考虑：

**状态结构设计**：
```rust
/// 内部词法分析器状态 - 实现细节，不暴露给用户
struct LexerState<I: Iterator<Item = char>> {
    chars: Peekable<I>,
    current_pos: Position,
    errors: Vec<LexError>,
    peeked_tokens: VecDeque<Token>, // 用于 lookahead
}
```

**迭代器适配器实现**：
- 使用 `Peekable<Iterator<char>>` 支持字符前瞻
- 状态机驱动的 Token 识别逻辑
- 错误恢复时的同步点选择策略

**关键权衡**：
- 内存效率 vs 前瞻能力
- 错误恢复粒度 vs 性能开销
- 流式处理 vs 随机访问能力

### 问题：状态操作函数的内部组织

内部实现需要的辅助函数应该如何组织：

**字符流操作**：
```rust
// 内部实现函数 - 不暴露给用户
fn advance_char(state: &mut LexerState<I>) -> Option<char>
fn peek_char(state: &LexerState<I>) -> Option<char>
fn skip_whitespace(state: &mut LexerState<I>)
fn advance_position(pos: &mut Position, ch: char)
```

**Token 解析函数**：
```rust
fn parse_number(state: &mut LexerState<I>, start_char: char) -> Result<Token, LexError>
fn parse_string(state: &mut LexerState<I>) -> Result<Token, LexError>
fn parse_symbol(state: &mut LexerState<I>, start_char: char) -> Result<Token, LexError>
fn parse_comment(state: &mut LexerState<I>) -> Result<Token, LexError>
```

**组织策略**：
- 按功能分组到子模块
- 使用特征对象实现可插拔解析器
- 函数组合vs面向对象设计

### 问题：数值解析的精度和类型统一处理

Scheme 支持多种数值类型（整数、有理数、实数、复数），需要统一的解析策略：

**类型层次设计**：
- 是否在词法层面区分不同数值类型
- 精度损失的处理策略
- 科学计数法和特殊数值的支持

**解析策略**：
- 状态机 vs 正则表达式
- 错误恢复的边界处理
- Unicode 数字字符的支持

### 问题：Trivia Tokens 的处理策略和性能影响

包含 Trivia Tokens 会增加 Token 流的大小，需要考虑：

**存储策略**：
- 是否总是生成 Trivia Tokens
- 按需生成 vs 后置过滤
- 内存使用优化

**使用场景**：
- 代码格式化工具的需求
- 语法高亮的支持
- 调试信息的完整性

### 问题：Unicode 符号的高效处理和验证策略

TODO

### 问题：错误恢复的同步点选择和恢复粒度

TODO

### 问题：大文件流式处理的内存管理策略

TODO

### 问题：位置信息跟踪的性能开销优化

TODO
