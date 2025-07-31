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
- **源码追踪**：每个S表达式都能追踪到原始AST节点
- **性能优化**：高效的递归下降解析
- **函数式设计**：纯函数实现，避免在结构体上编码业务逻辑

## 关键数据类型

### AST 核心数据结构

#### SExpr - S表达式

S表达式是Scheme语言的核心数据结构，表示所有可能的语法构造：

```rust
/// S表达式 - Scheme语言的核心数据结构
/// 使用 Rc 支持宏展开时的有向无环图结构
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    /// 原子值（数字、字符串、布尔值、符号等）
    Atom(Value),
    /// 列表结构 (car . cdr)
    Cons { car: Rc<SExprNode>, cdr: Rc<SExprNode> },
    /// 空列表
    Nil,
    /// 向量（数组）
    Vector(Vec<Rc<SExprNode>>),
}
```

#### SExprNode - 带源追踪的S表达式

结合S表达式和AST节点信息的互递归结构：

```rust
/// 带源追踪的S表达式节点
#[derive(Debug, Clone)]
pub struct SExprNode {
    /// S表达式内容
    pub expr: SExpr,
    /// 对应的AST节点（用于源追踪）
    pub ast_node: Option<Weak<ASTNode>>,
}
```

#### ASTNode - AST节点

AST节点包含一个S表达式和其在源代码中的位置信息：

```rust
/// AST节点 - 包含S表达式和位置信息
#[derive(Debug, Clone)]
pub struct ASTNode {
    /// S表达式内容
    pub expr: SExprNode,
    /// 源代码位置范围
    pub span: Span,
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
```

### ParseResult

解析结果包含AST和重建的源代码内容：

```rust
/// 解析结果类型 - 包含AST和源码内容
pub type ParseResult = Result<(Vec<ASTNode>, String), (ParseError, String)>;
```

## 核心函数接口（对外接口）

**重要说明**：本节只记录对外暴露的主要接口函数，不包括内部实现函数、私有方法和辅助函数。

### parse

Parser的核心函数，从token流解析AST并重建源代码内容。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| tokens | `impl Iterator<Item = Result<Token, LexError>>` | 来自Lexer的token流 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `ParseResult` | 成功时返回`(Vec<ASTNode>, String)`，失败时返回`(ParseError, String)` |

#### 说明
- 成功时：返回解析得到的AST节点序列和从token流重建的源代码字符串
- 失败时：返回解析错误和已处理token重建的源代码字符串（用于错误报告）

## 内部实现结构

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

## 关键设计问题

### 问题：SExpr与ASTNode的关系设计和源追踪机制

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
