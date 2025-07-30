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

使用代数数据类型表示不同的 Token 变体：

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

### LexerState

词法分析器的状态数据，不包含方法：

```rust
/// 词法分析器状态 - 纯数据结构
pub struct LexerState<R: Read> {
    /// 输入流
    pub reader: PeekableReader<R>,
    /// 当前位置
    pub current_pos: Position,
    /// 当前字节偏移
    pub byte_offset: usize,
    /// 错误收集器
    pub errors: Vec<LexError>,
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

## 关键功能函数接口

### 状态构造函数

#### new_lexer_state

| 参数名 | 类型 | 描述 |
|--------|------|------|
| reader | R | 实现了 Read trait 的输入源 |

| 类型 | 描述 |
|------|------|
| `LexerState<R>` | 新创建的词法分析器状态 |

### 核心词法分析函数

#### next_token

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态的可变引用 |

| 类型 | 描述 |
|------|------|
| `Result<Token, LexError>` | 下一个 Token 或错误 |

#### tokenize_all

| 参数名 | 类型 | 描述 |
|--------|------|------|
| input | `&str` | 源代码字符串 |

| 类型 | 描述 |
|------|------|
| `Result<Vec<Token>, Vec<LexError>>` | 所有 Token 或错误列表 |

### 特定类型解析函数

#### parse_number

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |
| start_char | `char` | 数字的起始字符 |

| 类型 | 描述 |
|------|------|
| `Result<Token, LexError>` | 数字 Token 或错误 |

#### parse_string

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<Token, LexError>` | 字符串 Token 或错误 |

#### parse_symbol

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |
| start_char | `char` | 符号的起始字符 |

| 类型 | 描述 |
|------|------|
| `Result<Token, LexError>` | 符号 Token 或错误 |

#### parse_character

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<Token, LexError>` | 字符 Token 或错误 |

### 辅助功能函数

#### advance_position

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |
| ch | `char` | 当前字符 |

| 类型 | 描述 |
|------|------|
| `()` | 无返回值，更新位置信息 |

#### peek_char

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<Option<char>, LexError>` | 下一个字符或错误 |

#### skip_whitespace

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<(), LexError>` | 成功或错误 |

### 错误处理函数

#### recover_from_error

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut LexerState<R>` | 词法分析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<(), LexError>` | 恢复结果 |

#### collect_errors

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&LexerState<R>` | 词法分析器状态 |

| 类型 | 描述 |
|------|------|
| `&[LexError]` | 当前收集的所有错误 |

## 关键设计问题

### 问题：数值解析的精度和类型统一处理

TODO

### 问题：Unicode 符号的高效处理和验证策略

TODO

### 问题：错误恢复的同步点选择和恢复粒度

TODO

### 问题：大文件流式处理的内存管理策略

TODO

### 问题：位置信息跟踪的性能开销优化

TODO
