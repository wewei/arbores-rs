# 语法分析器设计

状态：Draft-1

## 概述

语法分析器负责将词法分析器产生的 Token 流转换为抽象语法树（AST），采用函数式设计，数据与行为分离。

## 模块职责（功能性需求）

- **语法解析**：将 Token 流转换为 Scheme 的抽象语法树
- **结构验证**：验证语法结构的正确性（括号匹配、列表结构等）
- **位置传递**：准确传递位置信息到 AST 节点
- **错误恢复**：在语法错误时能够继续解析后续内容

## 设计目标（非功能性需求）

- **语法完整性**：支持完整的 Scheme 语法结构
- **类型安全**：泛型设计支持 Value 和 MValue 两种模式
- **性能优化**：高效的递归下降解析
- **函数式设计**：纯函数实现，避免在结构体上编码业务逻辑

## 关键数据类型

### ParserState

解析器状态的纯数据结构：

```rust
/// 语法分析器状态 - 纯数据结构
pub struct ParserState {
    /// Token 流（使用索引而不是迭代器以支持回退）
    pub tokens: Vec<Token>,
    /// 当前 Token 索引
    pub current_index: usize,
    /// 解析错误收集
    pub errors: Vec<ParseError>,
}
```

### ParseError

解析错误的代数数据类型：

```rust
/// 语法解析错误 - 使用 enum 表示不同错误情况
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// 意外的 token
    UnexpectedToken {
        expected: String,
        found: Token,
    },
    /// 意外的文件结束
    UnexpectedEof,
    /// 未终止的列表
    UnterminatedList {
        start_position: Position,
    },
    /// 未终止的向量
    UnterminatedVector {
        start_position: Position,
    },
    /// 无效的点对列表
    InvalidDottedList {
        position: Position,
    },
    /// 词法错误传播
    LexError(LexError),
    /// 多个错误
    MultipleErrors(Vec<ParseError>),
}
```

### ParseResult

解析结果的代数数据类型：

```rust
/// 解析结果 - 扩展的 Result 类型
#[derive(Debug, Clone)]
pub enum ParseResult<T> {
    /// 成功解析
    Success(T),
    /// 带警告的成功
    Warning { value: T, message: String },
    /// 解析失败
    Error(ParseError),
}
```

## 关键功能函数接口

### 状态管理函数

#### new_parser_state

| 参数名 | 类型 | 描述 |
|--------|------|------|
| tokens | `Vec<Token>` | 词法分析产生的 Token 序列 |

| 类型 | 描述 |
|------|------|
| `ParserState` | 新创建的解析器状态 |

#### current_token

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&ParserState` | 解析器状态的引用 |

| 类型 | 描述 |
|------|------|
| `Option<&Token>` | 当前 Token 的引用 |

#### advance_parser

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态的可变引用 |

| 类型 | 描述 |
|------|------|
| `()` | 无返回值，前进到下一个 Token |

### 核心解析函数

#### parse_expressions

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<Vec<SExpr<V>>, ParseError>` | 表达式序列或错误 |

#### parse_expression

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<SExpr<V>, ParseError>` | 单个表达式或错误 |

#### parse_from_string

| 参数名 | 类型 | 描述 |
|--------|------|------|
| input | `&str` | 源代码字符串 |

| 类型 | 描述 |
|------|------|
| `Result<Vec<SExpr<V>>, ParseError>` | 解析结果或错误 |

### 特定结构解析函数

#### parse_list

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<SExpr<V>, ParseError>` | 列表表达式或错误 |

#### parse_vector

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<SExpr<V>, ParseError>` | 向量表达式或错误 |

#### parse_quoted_expression

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<SExpr<V>, ParseError>` | 引用表达式或错误 |

#### parse_atomic_expression

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `Result<SExpr<V>, ParseError>` | 原子表达式或错误 |

### 列表构建函数

#### build_proper_list

| 参数名 | 类型 | 描述 |
|--------|------|------|
| elements | `Vec<SExpr<V>>` | 列表元素 |

| 类型 | 描述 |
|------|------|
| `V` | 构建的列表值 |

#### build_dotted_list

| 参数名 | 类型 | 描述 |
|--------|------|------|
| elements | `Vec<SExpr<V>>` | 列表元素 |
| tail | `SExpr<V>` | 点对的尾部 |

| 类型 | 描述 |
|------|------|
| `V` | 构建的点对列表值 |

### 错误处理函数

#### recover_from_parse_error

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `()` | 无返回值，恢复到同步点 |

#### find_sync_point

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |

| 类型 | 描述 |
|------|------|
| `bool` | 是否找到同步点 |

### 验证和检查函数

#### check_token_type

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&ParserState` | 解析器状态 |
| expected | `&TokenType` | 期望的 Token 类型 |

| 类型 | 描述 |
|------|------|
| `bool` | 当前 Token 是否匹配期望类型 |

#### consume_token

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |
| expected | `TokenType` | 期望的 Token 类型 |

| 类型 | 描述 |
|------|------|
| `Result<Token, ParseError>` | 消费的 Token 或错误 |

#### expect_token

| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | `&mut ParserState` | 解析器状态 |
| expected_type | `TokenType` | 期望的 Token 类型 |
| context | `&str` | 错误上下文描述 |

| 类型 | 描述 |
|------|------|
| `Result<Token, ParseError>` | 期望的 Token 或详细错误 |

## 关键设计问题

### 问题：Value 和 MValue 泛型类型系统的设计和转换机制

TODO

### 问题：递归下降解析的深度限制和栈溢出防护

TODO

### 问题：点对列表语法的正确解析和边界情况处理

TODO

### 问题：引用表达式的标准化转换和元数据保持

TODO

### 问题：错误恢复中同步点的选择策略和恢复粒度

TODO

### 问题：向量和列表在类型系统中的区分和统一处理

TODO
