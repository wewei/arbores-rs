# 语法分析器设计

状态：Draft-2

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

解析结果类型，只包含AST或错误信息：

```rust
/// 解析结果类型 - 只包含AST或错误
pub type ParseResult = Result<Vec<ASTNode>, ParseError>;
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

Parser的核心函数，从token流解析AST并重建源代码内容。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| tokens | `impl Iterator<Item = Result<Token, LexError>>` | 来自Lexer的token流 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `ParseOutput` | 包含解析结果和重建的源代码文本的结构体 |

#### 说明
- `result` 字段：成功时包含解析得到的AST节点序列，失败时包含解析错误
- `source_text` 字段：从token流重建的源代码字符串（无论成功或失败都会包含已处理的部分）

## 关键设计问题

### 问题：SExpr与ASTNode的关系设计和源追踪机制

`ASTNode` 会持有解析出来的 `SExpr`，而每一个 `SExpr`，无论是在编译期从源代码生成的 `SExpr`，还是在运行时计算出来的中间值，都可以用一个弱引用追溯回源代码。

- 对于编译期生成的 `SExpr`，关联到持有它的 `ASTNode`
- 对于运行时计算出的 `SExpr`，关联到计算出它的 `SExpr` 关联的 `ASTNode`。

例如 `(+ 1 2)` 对应的 `SExpr` 和 `ASTNode` 相互关联。计算出的结果 `3`，也关联到同一个 `ASTNode`。

### 问题：采用什么的语法分析算法？

S-Expression 结构比较简单，应该可以采用递归下降算法。

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
fn parse_quoted_expression(quote_token: Token, expr: SExprNode) -> SExprNode {
    // 创建 quote 符号节点
    let quote_symbol = SExprNode {
        expr: SExpr::Atom(Value::Symbol("quote".to_string())),
        ast_node: None, // 将在创建 AST 节点时设置
    };
    
    // 创建尾部 nil 节点
    let nil_node = SExprNode {
        expr: SExpr::Nil,
        ast_node: None, // 将在创建 AST 节点时设置
    };
    
    // 构建内层列表：(expr)
    let inner_cons = SExprNode {
        expr: SExpr::Cons {
            car: Rc::new(expr),
            cdr: Rc::new(nil_node),
        },
        ast_node: None, // 将在创建 AST 节点时设置
    };
    
    // 构建外层列表：(quote (expr))
    let quoted_expr = SExprNode {
        expr: SExpr::Cons {
            car: Rc::new(quote_symbol),
            cdr: Rc::new(inner_cons),
        },
        ast_node: None, // 将在上层函数中设置到对应的 ASTNode
    };
    
    quoted_expr
}

fn create_ast_node_for_quote(quoted_expr: SExprNode, quote_span: Span) -> Rc<ASTNode> {
    // 创建 AST 节点
    let ast_node = Rc::new(ASTNode {
        expr: quoted_expr.clone(),
        span: quote_span,
    });
    
    // 建立 SExprNode 到 ASTNode 的弱引用
    // 这里需要递归设置所有嵌套的 SExprNode 的 ast_node 字段
    set_ast_node_recursive(&quoted_expr, &ast_node);
    
    ast_node
}

fn set_ast_node_recursive(sexpr_node: &SExprNode, ast_node: &Rc<ASTNode>) {
    // 注意：这里需要使用内部可变性来修改 ast_node 字段
    // 实际实现中可能需要使用 RefCell 或其他机制
    match &sexpr_node.expr {
        SExpr::Cons { car, cdr } => {
            set_ast_node_recursive(car, ast_node);
            set_ast_node_recursive(cdr, ast_node);
        }
        SExpr::Vector(elements) => {
            for elem in elements {
                set_ast_node_recursive(elem, ast_node);
            }
        }
        _ => {} // Atom 和 Nil 不需要递归处理
    }
}
```

**元数据保持原则**：
1. **源位置追踪**：转换后的AST节点保持原始位置信息
2. **语法糖展开**：保留展开前后的对应关系
3. **调试信息**：错误报告时显示原始语法形式

**AST节点关联策略**：
- **统一追踪**：所有由引用语法糖展开的S表达式都关联到同一个AST节点
- **位置保持**：AST节点的`span`字段包含整个引用表达式的范围（从引用符号到被引用表达式结束）
- **错误定位**：当展开后的表达式出错时，能够准确定位到原始的引用语法位置

**完整的引用解析流程**：
```rust
fn parse_quote_syntax(state: &mut ParserState) -> Result<Rc<ASTNode>, ParseError> {
    let quote_token = expect_token(state, TokenType::Quote)?; // 获取 ' token
    let quoted_expr_node = parse_expression(state)?; // 解析被引用的表达式
    
    // 构建展开后的 (quote expr) 结构
    let expanded_sexpr = parse_quoted_expression(quote_token.clone(), quoted_expr_node);
    
    // 创建包含完整位置信息的 AST 节点
    let full_span = Span::new(
        quote_token.start_pos(), 
        expanded_sexpr.span.end_pos()
    );
    
    let ast_node = create_ast_node_for_quote(expanded_sexpr, full_span);
    
    Ok(ast_node)
}
```

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
// 在SExpr中明确区分
pub enum SExpr {
    Cons { car: Rc<SExprNode>, cdr: Rc<SExprNode> }, // 列表
    Vector(Vec<Rc<SExprNode>>),                       // 向量
    Nil,                                              // 空列表（非空向量）
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
    fn get(&self, index: usize) -> Option<&SExprNode>;
    fn iter(&self) -> Box<dyn Iterator<Item = &SExprNode>>;
}

impl Sequential for SExpr {
    fn length(&self) -> usize {
        match self {
            SExpr::Vector(vec) => vec.len(),
            SExpr::Cons { .. } => self.list_length(),
            SExpr::Nil => 0,
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
fn parse_list(state: &mut ParserState) -> Result<SExprNode, ParseError> {
    let start_token = expect_token(state, TokenType::LeftParen)?;
    let mut elements = Vec::new();
    let mut source_parts = vec![start_token.raw_text.clone()];
    
    loop {
        // 检查列表结束
        if let Some(token) = peek_token(state) {
            if token.token_type == TokenType::RightParen {
                let end_token = advance_token(state).unwrap();
                source_parts.push(end_token.raw_text.clone());
                break;
            }
        }
        
        // 检查点对列表
        if let Some(token) = peek_token(state) {
            if token.token_type == TokenType::Dot {
                return parse_dotted_list(state, elements, source_parts);
            }
        }
        
        // 解析下一个表达式
        let expr = parse_expression(state)?;
        source_parts.push(expr.source_text());
        elements.push(expr);
    }
    
    // 构建列表SExpr
    let list_sexpr = build_proper_list(elements);
    let span = Span::new(start_token.start_pos(), state.last_token_end());
    let source_text = source_parts.join("");
    
    Ok(SExprNode {
        expr: list_sexpr,
        ast_node: None, // 将在上层设置
        span,
        source_text,
    })
}

fn build_proper_list(elements: Vec<SExprNode>) -> SExpr {
    elements.into_iter().rev().fold(SExpr::Nil, |acc, elem| {
        SExpr::Cons {
            car: Rc::new(elem),
            cdr: Rc::new(SExprNode {
                expr: acc,
                ast_node: None,
                span: Span::default(),
                source_text: String::new(),
            }),
        }
    })
}
```

### 源码追踪和AST构建

```rust
fn create_ast_node(expr_node: SExprNode) -> Rc<ASTNode> {
    let ast_node = Rc::new(ASTNode {
        expr: expr_node.clone(),
        span: expr_node.span,
    });
    
    // 建立双向弱引用关系
    if let Some(sexpr_node) = &expr_node.expr {
        sexpr_node.ast_node = Some(Rc::downgrade(&ast_node));
    }
    
    ast_node
}

fn parse_with_source_tracking(tokens: Vec<Token>) -> ParseOutput {
    let mut state = ParserState::new(tokens);
    let mut ast_nodes = Vec::new();
    let mut source_buffer = String::new();
    
    while !is_at_end(&state) {
        match parse_expression(&mut state) {
            Ok(expr_node) => {
                source_buffer.push_str(&expr_node.source_text());
                let ast_node = create_ast_node(expr_node);
                ast_nodes.push((*ast_node).clone());
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
        result: Ok(ast_nodes),
        source_text: source_buffer,
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
    elements_before_dot: &[SExprNode],
    tail_expr: Option<&SExprNode>
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

