# 语法分析器设计

状态：Reviewed

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

## 源代码结构

语法分析器模块按照职责分离的原则组织，采用分层架构设计：

```text
/src/interpreter/parser/
├── mod.rs        # 模块接口和公共 API
├── types.rs      # 核心数据类型定义
├── rules.rs      # 递归下降规则定义
└── engine.rs     # 递归下降解析引擎
```

**模块职责说明**：

- **`types.rs`**：定义语法分析器的核心数据结构
  - `SExpr`, `SExprContent` - AST 节点类型
  - `ParseError`, `ParseResult`, `ParseOutput` - 错误和结果类型
  - `ParserState` - 内部解析状态管理
  - 类型相关的辅助方法和特征实现

- **`rules.rs`**：集中管理递归下降解析规则
  - `parse_expression()` - 表达式解析的主入口
  - `parse_list()`, `parse_dotted_list()` - 列表解析规则
  - `parse_quoted_expression()` - 引用语法解析
  - `parse_atom()`, `parse_vector()` - 原子和向量解析
  - 各种验证和辅助解析函数

- **`engine.rs`**：实现递归下降解析核心逻辑
  - `ParserState` 状态管理和 Token 流操作
  - `parse()` 主要 API 函数和解析流程控制
  - 错误处理和恢复机制（MVP 阶段简化）
  - 源代码重建和位置信息传递

- **`mod.rs`**：模块对外接口
  - 导出公共类型和函数
  - `parse()` 主要 API 函数
  - 模块级文档和使用示例

## 关键数据类型

### AST 核心数据结构

#### SExpr - 带源追踪的S表达式

S表达式是Scheme语言的核心数据结构，包含表达式内容和可选的源码位置信息：

```rust
/// S表达式 - 带源追踪的Scheme核心数据结构
/// 使用 Rc 支持宏展开时的有向无环图结构
#[derive(Debug, Clone)]
pub struct SExpr {
    /// S表达式的具体内容
    pub content: SExprContent,
    /// 源代码位置信息
    pub span: Rc<Span>,
}
```

#### SExprContent - S表达式内容

S表达式的具体内容类型，表示所有可能的语法构造：

```rust
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

解析结果类型，包含解析得到的S表达式序列或错误信息：

```rust
/// 解析结果类型 - 包含S表达式序列或错误
pub type ParseResult = Result<Vec<SExpr>, ParseError>;
```

### ParseOutput

Parser的完整输出结构，包含解析结果和重建的源代码：

```rust
/// Parser的完整输出结构
pub struct ParseOutput {
    /// 解析结果（AST或错误）
    pub result: ParseResult,
    /// 重建的源代码文本
    pub source_text: String,
}
```

## 核心函数接口（对外接口）

**重要说明**：本节只记录对外暴露的主要接口函数，不包括内部实现函数、私有方法和辅助函数。

### parse

Parser的核心函数，从token流解析S表达式并重建源代码内容。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| tokens | `impl Iterator<Item = Result<Token, LexError>>` | 来自Lexer的token流 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `ParseOutput` | 包含解析结果和重建的源代码文本的结构体 |

#### 说明
- `result` 字段：成功时包含解析得到的S表达式序列，失败时包含解析错误
- `source_text` 字段：从token流重建的源代码字符串（无论成功或失败都会包含已处理的部分）

## 关键设计问题

### 问题：SExpr的源追踪机制和位置信息传递

简化后的设计使用 `Rc<Span>` 来记录每个S表达式的源码位置：

- 对于从源代码直接解析的S表达式，`span` 字段包含对应的源码位置信息
- 对于运行时计算生成的S表达式，`span` 字段可以继承自计算它的表达式
- 对于内置或系统生成的S表达式，`span` 字段使用空 span `Span::empty(n)`，`n` 是系统生成S表达式插入的位置

**位置信息继承规则**：
```rust
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

例如 `(+ 1 2)` 对应的 `SExpr` 包含整个表达式的 `span`。计算出的结果 `3` 可以继承同样的 `span`，用于错误追踪。

### 问题：点对列表语法的正确解析和边界情况处理

**点对列表语法规则**：
```scheme
;; 正确的点对列表格式
(a . b)           ; 基本点对
(a b . c)         ; 列表 + 尾部点对
(a b c . d)       ; 多元素 + 尾部点对
```

**解析策略**：
1. **识别模式**：在列表解析过程中检测到 `Dot` token
2. **位置验证**：点号不能出现在列表开头或连续出现
3. **尾部处理**：点号后必须恰好有一个表达式，然后是右括号

**边界情况处理**：
```rust
// 错误案例和处理
fn validate_dotted_list(elements: &[SExpr], dot_pos: usize, tail: &SExpr) -> Result<(), DottedListError> {
    match dot_pos {
        0 => Err(DottedListError::InvalidDotPosition), // ". a)" 
        pos if pos >= elements.len() => Err(DottedListError::InsufficientElements), // "()"内的点号
        _ => Ok(())
    }
}
```

**实现要点**：
- 使用状态机跟踪解析状态：`Normal` -> `FoundDot` -> `ExpectingTail` -> `Complete`
- 构建时将普通列表转换为嵌套的 `Cons` 结构
- 点对列表的最后一个 `cdr` 指向尾部表达式而非 `Nil`

### 问题：引用表达式的标准化转换和元数据保持

**引用语法转换规则**：
```scheme
'expr    => (quote expr)
`expr    => (quasiquote expr)  
,expr    => (unquote expr)
,@expr   => (unquote-splicing expr)
```

**转换实现策略**：
```rust
fn parse_quoted_expression(quote_token: Token, expr_node: SExpr) -> SExpr {
    // 创建 quote 符号 - 关联到 quote_token 的位置
    let quote_symbol = SExpr::with_span(
        SExprContent::Atom(Value::Symbol("quote".to_string())),
        quote_token.span
    );
    
    // 创建尾部 nil - 位置为 expr_node 结束位置的空span
    let nil_span = Span::empty(expr_node.span.end);
    let nil_node = SExpr {
        content: SExprContent::Nil,
        span: Rc::new(nil_span),
    };
    
    // 构建内层 cons：(expr . ()) - 应该关联到 expr_node 的位置
    let inner_cons = SExpr::inherit_span(&expr_node, SExprContent::Cons {
        car: Rc::new(expr_node.clone()),
        cdr: Rc::new(nil_node),
    });
    
    // 构建外层 cons：(quote . (expr . ())) - 关联到整个 'expr 表达式的位置
    let full_span = Span::new(
        quote_token.span.start, 
        expr_node.span.end
    );
    
    SExpr::with_span(SExprContent::Cons {
        car: Rc::new(quote_symbol),
        cdr: Rc::new(inner_cons),
    }, full_span)
}
```

**元数据保持原则**：
1. **源位置追踪**：转换后的S表达式保持原始位置信息
2. **语法糖展开**：保留展开前后的对应关系
3. **调试信息**：错误报告时显示原始语法形式

**位置信息分配策略**：
- **分层追踪**：
  - 外层 cons `(quote . (expr . ()))` 关联到整个引用表达式的位置范围（从 `'` 到 `expr` 结束）
  - 内层 cons `(expr . ())` 关联到被引用表达式的位置范围（与 `expr` 相同）
  - `quote` 符号关联到引用符号的位置
  - `expr` 保持其原始的位置关联
  - 尾部 `nil` 关联到 `expr` 结束位置的空span（start == end == expr.end）
- **位置保持**：每个 cons 结构的位置信息精确对应其在源代码中的语义范围
- **错误定位**：当展开后的不同部分出错时，能够准确定位到对应的源代码位置

### 问题：递归下降解析的深度限制和栈溢出防护

**深度限制策略**：
```rust
const MAX_PARSE_DEPTH: usize = 1000; // 最大递归深度

struct ParserState {
    tokens: Vec<Token>,
    current_index: usize,
    errors: Vec<ParseError>,
    depth: usize, // 当前递归深度
}

fn parse_expression_with_depth_check(state: &mut ParserState) -> Result<SExpr, ParseError> {
    if state.depth >= MAX_PARSE_DEPTH {
        return Err(ParseError::UnexpectedToken {
            found: current_token(state)?.clone(),
            reason: UnexpectedTokenReason::Other("Maximum parsing depth exceeded".to_string()),
        });
    }
    
    state.depth += 1;
    let result = parse_expression_impl(state);
    state.depth -= 1;
    result
}
```

**栈溢出防护机制**：
1. **深度计数器**：跟踪当前递归层级
2. **提前终止**：超过限制时返回错误而非继续递归
3. **迭代化改造**：对于线性结构（如长列表）使用迭代解析
4. **错误恢复**：深度超限时尝试跳转到同步点

**性能优化**：
- 使用 `Vec` 模拟栈来处理深度嵌套的列表
- 尾递归优化：识别尾递归模式并转换为循环

### 问题：向量和列表在类型系统中的区分和统一处理

**类型区分原则**：
```rust
// 在SExprContent中明确区分
pub enum SExprContent {
    Cons { car: Rc<SExpr>, cdr: Rc<SExpr> }, // 列表
    Vector(Vec<Rc<SExpr>>),                   // 向量
    Nil,                                      // 空列表（非空向量）
    // ...
}
```

**解析差异**：
```scheme
(a b c)     ; 列表：递归的 Cons 结构
#(a b c)    ; 向量：连续内存的数组结构
```

**统一处理接口**：
```rust
trait Sequential {
    fn length(&self) -> usize;
    fn get(&self, index: usize) -> Option<&SExpr>;
    fn iter(&self) -> Box<dyn Iterator<Item = &SExpr>>;
}

impl Sequential for SExpr {
    fn length(&self) -> usize {
        match &self.content {
            SExprContent::Vector(vec) => vec.len(),
            SExprContent::Cons { .. } => self.list_length(),
            SExprContent::Nil => 0,
            _ => 0,
        }
    }
    // ...
}
```

**性能考量**：
- 向量：O(1) 随机访问，适合索引操作
- 列表：O(1) 头部操作，适合递归处理
- 统一迭代器：提供一致的遍历接口

### 问题：错误恢复中同步点的选择策略和恢复粒度

**同步点选择策略**：
1. **表达式边界**：括号、分号、关键字等明确的语法分隔符
2. **嵌套层级**：回退到较低的嵌套层级继续解析
3. **Token类型**：特定类型的Token（如`RightParen`）作为恢复点

**错误恢复粒度**：
- **表达式级别**：跳过当前错误表达式，继续解析下一个
- **列表级别**：在列表解析失败时跳转到列表结尾
- **顶层级别**：严重错误时回退到顶层继续解析

**MVP阶段限制**：暂时不实现错误恢复，遇到错误即停止解析并返回错误信息。

## 实现示例

### 基本列表解析流程

```rust
fn parse_list(state: &mut ParserState) -> Result<SExpr, ParseError> {
    let start_token = expect_token(state, TokenType::LeftParen)?;
    let mut elements = Vec::new();
    
    loop {
        // 检查列表结束
        if let Some(token) = peek_token(state) {
            if token.token_type == TokenType::RightParen {
                let end_token = advance_token(state).unwrap();
                break;
            }
        }
        
        // 检查点对列表
        if let Some(token) = peek_token(state) {
            if token.token_type == TokenType::Dot {
                return parse_dotted_list(state, elements, start_token);
            }
        }
        
        // 解析下一个表达式
        let expr = parse_expression(state)?;
        elements.push(expr);
    }
    
    // 构建列表SExpr
    let list_content = build_proper_list(elements);
    let span = Span::new(start_token.span.start, state.last_token_end());
    
    Ok(SExpr::with_span(list_content, span))
}

fn build_proper_list(elements: Vec<SExpr>) -> SExprContent {
    elements.into_iter().rev().fold(SExprContent::Nil, |acc, elem| {
        // 为尾部创建正确的 nil span（在当前元素的结尾位置）
        let nil_span = Span::empty(elem.span.end);
        let cdr_node = if matches!(acc, SExprContent::Nil) {
            SExpr {
                content: acc,
                span: Rc::new(nil_span),
            }
        } else {
            SExpr::without_span(acc)
        };
        
        SExprContent::Cons {
            car: Rc::new(elem),
            cdr: Rc::new(cdr_node),
        }
    })
}
```

### 源码追踪和解析流程

```rust
fn parse_with_source_tracking(tokens: Vec<Token>) -> ParseOutput {
    let mut state = ParserState::new(tokens);
    let mut sexpr_nodes = Vec::new();
    let mut source_buffer = String::new();
    
    while !is_at_end(&state) {
        match parse_expression(&mut state) {
            Ok(sexpr) => {
                // 从token重建源代码文本
                source_buffer.push_str(&reconstruct_source_text(&sexpr));
                sexpr_nodes.push(sexpr);
            }
            Err(error) => {
                return ParseOutput {
                    result: Err(error),
                    source_text: source_buffer,
                };
            }
        }
    }
    
    ParseOutput {
        result: Ok(sexpr_nodes),
        source_text: source_buffer,
    }
}

fn reconstruct_source_text(sexpr: &SExpr) -> String {
    // 基于span信息和token内容重建源代码文本
    // 实际实现中可能需要维护token到源文本的映射
    match &sexpr.content {
        SExprContent::Atom(value) => value.to_string(),
        SExprContent::Cons { car, cdr } => {
            format!("({} . {})", 
                reconstruct_source_text(car), 
                reconstruct_source_text(cdr))
        }
        SExprContent::Nil => "()".to_string(),
        SExprContent::Vector(elements) => {
            let inner = elements.iter()
                .map(|e| reconstruct_source_text(e))
                .collect::<Vec<_>>()
                .join(" ");
            format!("#({})", inner)
        }
    }
}
```

### 错误处理示例

```rust
fn handle_unexpected_token(
    state: &ParserState, 
    found: Token, 
    expected: &str
) -> ParseError {
    ParseError::UnexpectedToken {
        found,
        reason: UnexpectedTokenReason::Expected(expected.to_string()),
    }
}

fn validate_dotted_list_syntax(
    dot_token: &Token,
    elements_before_dot: &[SExpr],
    tail_expr: Option<&SExpr>
) -> Result<(), ParseError> {
    if elements_before_dot.is_empty() {
        return Err(ParseError::UnexpectedToken {
            found: dot_token.clone(),
            reason: UnexpectedTokenReason::InvalidDottedList {
                dot_token: dot_token.clone(),
                context: DottedListError::InvalidDotPosition,
            },
        });
    }
    
    if tail_expr.is_none() {
        return Err(ParseError::UnexpectedToken {
            found: dot_token.clone(),
            reason: UnexpectedTokenReason::InvalidDottedList {
                dot_token: dot_token.clone(),
                context: DottedListError::MissingTailElement,
            },
        });
    }
    
    Ok(())
}
```
