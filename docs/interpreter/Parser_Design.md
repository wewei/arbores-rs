# 语法分析器设计

状态：Completed

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

语法分析器模块采用简化的分层架构设计，基于 Scheme 语法的简单性特点：

```text
/src/interpreter/parser/
├── mod.rs        # 模块接口和公共 API  
├── types.rs      # AST 类型和错误类型定义
├── engine.rs     # 简化的递归下降解析器
└── utils.rs      # 辅助函数（源码重建等）
```

**模块职责说明**：

- **`types.rs`**：定义语法分析器的核心数据结构
  - `SExpr`, `SExprContent` - AST 节点类型
  - `ParseError`, `ParseResult`, `ParseOutput` - 错误和结果类型
  - 类型相关的辅助方法和特征实现

- **`engine.rs`**：实现简化的递归下降解析器
  - `SimpleParser` - 主要解析器结构体
  - `parse_expression()` - 表达式解析的核心分发逻辑
  - `parse_list()`, `parse_dotted_list()` - 列表解析函数
  - `parse_quoted()` - 引用语法解析
  - `parse_atom()`, `parse_vector()` - 原子和向量解析
  - Token 流操作和错误处理

- **`utils.rs`**：辅助功能实现
  - `SourceBuilder` - 源代码重建工具
  - 位置信息处理函数
  - 通用验证和转换函数

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

### 问题：递归下降算法状态机如何设计，规约规则如何定义？

**结论**：对于 Scheme 这样的简单语法，不需要复杂的规则系统和状态机。

**设计原则**：
- **直接递归下降**：根据当前 token 直接分发到对应解析函数
- **函数式设计**：每个解析函数都是纯函数，接受 token 流返回 AST
- **简单明了**：避免过度设计，让代码更容易理解和维护

**为什么 Scheme 语法适合简单的递归下降？**

1. **LL(1) 特性**：每种语法结构都有唯一的起始标记
   ```scheme
   (expr ...)    ; '(' 开始 → 列表
   #(expr ...)   ; '#(' 开始 → 向量  
   'expr         ; ''' 开始 → 引用
   42            ; 数字 → 原子
   "hello"       ; 字符串 → 原子
   symbol        ; 符号 → 原子
   ```

2. **无歧义语法**：不需要优先级消歧或复杂的前瞻
3. **规整结构**：所有复杂结构都是简单结构的递归组合

## 状态机核心数据结构

### ParserState - 解析器状态

```rust
/// 递归下降解析器的不可变状态
/// 只包含解析过程中累积的信息，不包含输入流
#[derive(Debug, Clone)]
pub struct ParserState {
    /// 已解析的S表达式序列
    pub parsed_exprs: Vec<SExpr>,
    /// 已处理的源代码文本（用于重建）
    pub consumed_text: String,
    /// 当前解析深度（用于错误恢复和调试）
    pub depth: usize,
    /// 解析上下文栈（用于跟踪嵌套结构）
    pub context_stack: Vec<ParseContext>,
}

/// Token输入接口 - 由Engine管理
/// Engine负责从Iterator<Token>中读取并传递给状态转换函数
pub trait TokenInput {
    /// 查看下一个Token但不消费
    fn peek(&self) -> Option<&Result<Token, LexError>>;
    /// 消费并返回下一个Token
    fn next(&mut self) -> Option<Result<Token, LexError>>;
    /// 获取当前位置（用于错误报告）
    fn position(&self) -> usize;
}

/// 解析上下文 - 跟踪当前解析的语法结构
#[derive(Debug, Clone, PartialEq)]
pub enum ParseContext {
    /// 顶层表达式解析
    TopLevel,
    /// 列表内部解析
    List { 
        start_token: Token,     // 开始的 '(' token
        elements: Vec<SExpr>,   // 已解析的元素
        state: ListState,       // 列表解析状态
    },
    /// 向量内部解析
    Vector { 
        start_token: Token,     // 开始的 '#(' token
        elements: Vec<SExpr>,   // 已解析的元素
    },
    /// 引用表达式解析
    Quote { 
        quote_token: Token,     // 引用符号token
        quote_type: QuoteType,  // 引用类型
    },
}

/// 列表解析状态
#[derive(Debug, Clone, PartialEq)]
pub enum ListState {
    /// 正常列表解析
    Normal,
    /// 发现点号，等待尾部表达式
    ExpectingTail { dot_token: Token },
    /// 点对列表完成
    DottedComplete { tail: SExpr },
}

/// 引用类型
#[derive(Debug, Clone, PartialEq)]
pub enum QuoteType {
    Quote,           // '
    Quasiquote,      // `
    Unquote,         // ,
    UnquoteSplicing, // ,@
}
```

### 简化的递归下降设计

对于 Scheme 这样的简单语法，我们不需要复杂的规则系统。递归下降算法可以直接根据当前 token 类型进行分发：

```rust
/// 简化的解析器 - 直接递归下降，无需规则抽象
/// Scheme 语法足够简单，不需要复杂的规则匹配系统
pub struct SimpleParser<I> 
where 
    I: Iterator<Item = Result<Token, LexError>>
{
    tokens: std::iter::Peekable<I>,
    position: usize,
}

impl<I> SimpleParser<I> 
where 
    I: Iterator<Item = Result<Token, LexError>>
{
    /// 解析表达式 - 根据第一个 token 直接分发
    fn parse_expression(&mut self) -> Result<SExpr, ParseError> {
        match self.peek_token()? {
            TokenType::LeftParen => self.parse_list(),
            TokenType::VectorStart => self.parse_vector(),
            TokenType::Quote => self.parse_quoted(QuoteType::Quote),
            TokenType::Quasiquote => self.parse_quoted(QuoteType::Quasiquote),
            TokenType::Unquote => self.parse_quoted(QuoteType::Unquote),
            TokenType::UnquoteSplicing => self.parse_quoted(QuoteType::UnquoteSplicing),
            TokenType::Number | TokenType::String | TokenType::Symbol | 
            TokenType::Boolean | TokenType::Character => self.parse_atom(),
            _ => Err(ParseError::unexpected_token(self.current_token()?)),
        }
    }
}

```

### 为什么 Scheme 不需要复杂的规则系统？

1. **语法无歧义**：每种语法结构都有唯一的起始标记
   - `(` → 列表
   - `#(` → 向量  
   - `'` → 引用
   - 原子 token → 原子值

2. **LL(1) 特性**：只需要一个前瞻 token 就能确定解析路径
   ```scheme
   (define x 42)  ; 看到 '(' 就知道是列表
   'expr          ; 看到 ''' 就知道是引用
   42             ; 看到数字就知道是原子
   ```

3. **结构递归**：所有复杂结构都是简单结构的递归组合
   ```scheme
   (f (g x) (h y))  ; 嵌套列表，每层都用相同规则解析
   ```

### 简化后的解析流程

```rust
impl<I> SimpleParser<I> {
    /// 解析单个表达式
    fn parse_expression(&mut self) -> Result<SExpr, ParseError> {
        match self.peek_token()? {
            TokenType::LeftParen => self.parse_list(),
            TokenType::VectorStart => self.parse_vector(), 
            TokenType::Quote => self.parse_quoted(QuoteType::Quote),
            // ... 其他引用类型
            _ if self.is_atom_token() => self.parse_atom(),
            _ => Err(ParseError::unexpected_token(self.current_token()?)),
        }
    }
    
    /// 解析列表
    fn parse_list(&mut self) -> Result<SExpr, ParseError> {
        let start_token = self.consume_token()?; // 消费 '('
        let mut elements = Vec::new();
        
        loop {
            match self.peek_token()? {
                TokenType::RightParen => {
                    self.consume_token()?; // 消费 ')'
                    return Ok(self.build_list(elements, start_token));
                },
                TokenType::Dot => {
                    return self.parse_dotted_list(elements, start_token);
                },
                _ => {
                    elements.push(self.parse_expression()?);
                }
            }
        }
    }
    
    /// 解析点对列表的尾部
    fn parse_dotted_list(&mut self, elements: Vec<SExpr>, start_token: Token) 
        -> Result<SExpr, ParseError> {
        let dot_token = self.consume_token()?; // 消费 '.'
        let tail = self.parse_expression()?;   // 解析尾部表达式
        
        // 验证后面是 ')'
        match self.peek_token()? {
            TokenType::RightParen => {
                self.consume_token()?;
                Ok(self.build_dotted_list(elements, tail, start_token))
            },
            _ => Err(ParseError::invalid_dotted_list(dot_token)),
        }
    }
}
```

### 删除的复杂设计

以下设计在 Scheme 解析器中是不必要的：

1. **ParseRule 系统**：Scheme 语法简单，直接的 match 分发更清晰
2. **Priority 字段**：没有语法歧义，不需要优先级消歧
3. **TokenMatcher**：直接匹配 TokenType 即可，不需要复杂的匹配器
4. **RuleAction**：解析动作可以直接内联在对应函数中
5. **RuleSet**：不需要规则集合管理

### 简化后的优势

1. **代码更简洁**：减少了大量抽象层次
2. **性能更好**：直接分发比规则匹配更快
3. **易于理解**：解析逻辑更直观
4. **易于调试**：调用栈更清晰
5. **易于扩展**：新增语法特性只需要添加新的 match 分支

### 什么时候需要复杂的规则系统？

复杂的规则系统适用于：
1. **有歧义的语法**：需要优先级来消歧
2. **复杂的前瞻需求**：需要 LL(k) 或 LR 解析
3. **运算符优先级**：如中缀表达式解析
4. **语法扩展频繁**：需要插件化的语法规则

Scheme 的语法非常规整，不属于上述任何情况，所以简单的递归下降就足够了。

## 简化后的核心 API

基于简化的架构设计，解析器提供清晰的函数式接口：

```rust
/// 解析器主函数 - 对外暴露的唯一入口
pub fn parse(tokens: impl Iterator<Item = Result<Token, LexError>>) -> ParseOutput {
    let mut parser = SimpleParser::new(tokens);
    parser.parse_all()
}

/// 核心解析器结构
struct SimpleParser<I> {
    tokens: std::iter::Peekable<I>,
    source_builder: SourceBuilder, // 用于重建源码
    position: usize,               // 当前位置（用于错误报告）
}

impl<I> SimpleParser<I> 
where I: Iterator<Item = Result<Token, LexError>>
{
    /// 解析所有顶层表达式
    fn parse_all(&mut self) -> ParseOutput {
        let mut expressions = Vec::new();
        
        while !self.is_at_end() {
            match self.parse_expression() {
                Ok(expr) => {
                    expressions.push(expr);
                    self.source_builder.add_expression(&expr);
                },
                Err(error) => return ParseOutput {
                    result: Err(error),
                    source_text: self.source_builder.build(),
                },
            }
        }
        
        ParseOutput {
            result: Ok(expressions),
            source_text: self.source_builder.build(),
        }
    }
    
    /// 解析单个表达式 - 核心分发逻辑
    fn parse_expression(&mut self) -> Result<SExpr, ParseError> {
        match self.peek_token_type()? {
            TokenType::LeftParen => self.parse_list(),
            TokenType::VectorStart => self.parse_vector(),
            TokenType::Quote => self.parse_quoted(QuoteType::Quote),
            TokenType::Quasiquote => self.parse_quoted(QuoteType::Quasiquote),
            TokenType::Unquote => self.parse_quoted(QuoteType::Unquote),
            TokenType::UnquoteSplicing => self.parse_quoted(QuoteType::UnquoteSplicing),
            token_type if self.is_atom_type(token_type) => self.parse_atom(),
            _ => Err(self.unexpected_token_error()),
        }
    }
    
    /// Token 流操作辅助方法
    fn peek_token_type(&mut self) -> Result<TokenType, ParseError> {
        match self.tokens.peek() {
            Some(Ok(token)) => Ok(token.token_type.clone()),
            Some(Err(e)) => Err(ParseError::LexError(e.clone())),
            None => Err(ParseError::unexpected_eof()),
        }
    }
    
    fn consume_token(&mut self) -> Result<Token, ParseError> {
        match self.tokens.next() {
            Some(Ok(token)) => {
                self.position += 1;
                self.source_builder.add_token(&token);
                Ok(token)
            },
            Some(Err(e)) => Err(ParseError::LexError(e)),
            None => Err(ParseError::unexpected_eof()),
        }
    }
    
    fn is_at_end(&mut self) -> bool {
        matches!(self.tokens.peek(), None | Some(Ok(Token { token_type: TokenType::Eof, .. })))
    }
}
```

**设计优势总结**：

1. **简洁性**：去除了不必要的抽象层次，代码更直观
2. **性能**：直接函数调用比规则匹配系统更高效
3. **可维护性**：解析逻辑清晰，易于理解和调试
4. **可扩展性**：新增语法特性只需添加新的分支，不需要修改框架
5. **类型安全**：充分利用 Rust 的类型系统保证正确性

