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

### 问题：递归下降算法状态机如何设计，规约规则如何定义？

**设计原则**：
- **函数式不可变**：状态机状态完全不可变，每次迭代产生新状态
- **声明式规则**：规约规则采用声明式描述，与具体实现分离
- **组合式设计**：复杂规则由简单规则组合而成
- **类型安全**：利用Rust类型系统确保状态转换的正确性

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

### ParseRule - 解析规则抽象

```rust
/// 解析规则的抽象表示
/// 采用声明式设计，将规则定义与实现分离
#[derive(Debug, Clone)]
pub struct ParseRule {
    /// 规则名称（用于调试和错误报告）
    pub name: &'static str,
    /// 规则优先级（用于消歧）
    pub priority: u8,
    /// 前瞻Token匹配条件
    pub lookahead: TokenMatcher,
    /// 规则动作类型
    pub action: RuleAction,
}

/// Token匹配器 - 声明式描述Token匹配条件
#[derive(Debug, Clone)]
pub enum TokenMatcher {
    /// 精确匹配特定Token类型
    Exact(TokenType),
    /// 匹配多个可能的Token类型之一
    OneOf(Vec<TokenType>),
    /// 匹配原子类型Token（数字、字符串、符号等）
    Atom,
    /// 匹配引用前缀Token
    Quote,
    /// 自定义匹配函数
    Custom(fn(&Token) -> bool),
    /// 任意Token（用作fallback）
    Any,
}

/// Token类型枚举（简化版，对应Lexer的Token类型）
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    VectorStart,    // #(
    Dot,            // .
    Quote,          // '
    Quasiquote,     // `
    Unquote,        // ,
    UnquoteSplicing,// ,@
    Number,         // 数字
    String,         // 字符串
    Symbol,         // 符号
    Boolean,        // 布尔值
    Character,      // 字符
    Eof,            // 文件结束
}

/// 规则动作类型 - 定义规则匹配后的行为
#[derive(Debug, Clone)]
pub enum RuleAction {
    /// 解析原子表达式
    ParseAtom,
    /// 开始列表解析
    StartList,
    /// 结束列表解析
    EndList,
    /// 开始向量解析
    StartVector,
    /// 结束向量解析
    EndVector,
    /// 处理点号（点对列表）
    HandleDot,
    /// 处理引用语法
    HandleQuote(QuoteType),
    /// 递归解析子表达式
    ParseSubExpression,
    /// 错误恢复
    ErrorRecovery,
}
```

### ParseStep - 解析步骤结果

```rust
/// 单步解析的结果类型
/// 包含新状态和Token消费信息
#[derive(Debug, Clone)]
pub struct ParseStep {
    /// 解析后的新状态
    pub new_state: ParserState,
    /// 解析结果
    pub result: StepResult,
    /// 本次解析消费的Token数量
    pub tokens_consumed: usize,
}

/// 单步解析的具体结果
#[derive(Debug, Clone)]
pub enum StepResult {
    /// 成功解析了一个完整的表达式
    Expression(SExpr),
    /// 状态转换但未完成表达式（如进入列表）
    Continue,
    /// 到达输入结束
    EndOfInput,
    /// 解析错误
    Error(ParseError),
    /// 需要更多输入才能继续
    NeedMoreInput,
}
```

## 规约规则系统设计

### RuleSet - 规则集合

```rust
/// 解析规则集合 - 管理所有语法规则
#[derive(Debug)]
pub struct RuleSet {
    /// 表达式级别的规则
    pub expression_rules: Vec<ParseRule>,
    /// 列表内部的规则
    pub list_rules: Vec<ParseRule>,
    /// 向量内部的规则  
    pub vector_rules: Vec<ParseRule>,
    /// 引用处理规则
    pub quote_rules: Vec<ParseRule>,
    /// 错误恢复规则
    pub recovery_rules: Vec<ParseRule>,
}

impl RuleSet {
    /// 根据当前状态和前瞻Token选择合适的规则
    pub fn select_rule(&self, state: &ParserState, lookahead: &Token) -> Option<&ParseRule> {
        let rules = match state.current_context() {
            ParseContext::TopLevel => &self.expression_rules,
            ParseContext::List { .. } => &self.list_rules,
            ParseContext::Vector { .. } => &self.vector_rules,
            ParseContext::Quote { .. } => &self.quote_rules,
        };
        
        rules.iter()
            .filter(|rule| rule.lookahead.matches(lookahead))
            .max_by_key(|rule| rule.priority)
    }
}
```

### 标准规则定义示例

```rust
/// 构建标准Scheme语法规则集
pub fn build_scheme_rules() -> RuleSet {
    RuleSet {
        expression_rules: vec![
            // 原子表达式规则
            ParseRule {
                name: "atom",
                priority: 10,
                lookahead: TokenMatcher::Atom,
                action: RuleAction::ParseAtom,
            },
            
            // 列表开始规则
            ParseRule {
                name: "list_start",
                priority: 20,
                lookahead: TokenMatcher::Exact(TokenType::LeftParen),
                action: RuleAction::StartList,
            },
            
            // 向量开始规则
            ParseRule {
                name: "vector_start", 
                priority: 20,
                lookahead: TokenMatcher::Exact(TokenType::VectorStart),
                action: RuleAction::StartVector,
            },
            
            // 引用语法规则
            ParseRule {
                name: "quote",
                priority: 30,
                lookahead: TokenMatcher::Exact(TokenType::Quote),
                action: RuleAction::HandleQuote(QuoteType::Quote),
            },
            
            ParseRule {
                name: "quasiquote",
                priority: 30, 
                lookahead: TokenMatcher::Exact(TokenType::Quasiquote),
                action: RuleAction::HandleQuote(QuoteType::Quasiquote),
            },
            
            // ... 其他引用规则
        ],
        
        list_rules: vec![
            // 列表元素解析
            ParseRule {
                name: "list_element",
                priority: 10,
                lookahead: TokenMatcher::Custom(|t| !matches!(t.token_type, 
                    TokenType::RightParen | TokenType::Dot)),
                action: RuleAction::ParseSubExpression,
            },
            
            // 点对列表处理
            ParseRule {
                name: "dotted_pair",
                priority: 20,
                lookahead: TokenMatcher::Exact(TokenType::Dot),
                action: RuleAction::HandleDot,
            },
            
            // 列表结束
            ParseRule {
                name: "list_end",
                priority: 30,
                lookahead: TokenMatcher::Exact(TokenType::RightParen),
                action: RuleAction::EndList,
            },
        ],
        
        // ... 其他规则集合
    }
}
```

## 状态机执行模型

### 核心执行函数

```rust
/// Engine的核心实现 - 管理Token流和状态迭代
pub struct ParseEngine<I> 
where 
    I: Iterator<Item = Result<Token, LexError>>
{
    /// Token输入流
    tokens: std::iter::Peekable<I>,
    /// 当前位置（用于错误报告）
    position: usize,
}

impl<I> ParseEngine<I> 
where 
    I: Iterator<Item = Result<Token, LexError>>
{
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            position: 0,
        }
    }
    
    /// 状态机的核心迭代函数
    /// 纯函数设计：输入状态 + 当前Token -> 输出状态 + 结果
    pub fn parse_step(
        &mut self,
        state: ParserState, 
        rules: &RuleSet
    ) -> ParseStep {
        // 1. 获取前瞻Token
        let lookahead = match self.tokens.peek() {
            Some(Ok(token)) => token.clone(),
            Some(Err(lex_error)) => return ParseStep::error(state, lex_error.clone().into()),
            None => return ParseStep::end_of_input(state),
        };
        
        // 2. 根据当前状态和前瞻Token选择规则
        let rule = match rules.select_rule(&state, &lookahead) {
            Some(rule) => rule,
            None => return ParseStep::error(state, 
                ParseError::unexpected_token(lookahead, "No matching rule")),
        };
        
        // 3. 应用规则执行相应动作
        self.apply_rule(state, rule)
    }
    
    /// 应用规则并返回新状态
    fn apply_rule(&mut self, state: ParserState, rule: &ParseRule) -> ParseStep {
        match rule.action {
            RuleAction::ParseAtom => {
                // 消费一个Token并解析为原子
                if let Some(Ok(token)) = self.tokens.next() {
                    self.position += 1;
                    // 解析原子逻辑...
                    ParseStep::expression(state, parse_atom_from_token(token))
                } else {
                    ParseStep::error(state, ParseError::unexpected_eof())
                }
            },
            RuleAction::StartList => {
                // 消费左括号并进入列表上下文
                if let Some(Ok(token)) = self.tokens.next() {
                    self.position += 1;
                    let new_context = ParseContext::List {
                        start_token: token,
                        elements: vec![],
                        state: ListState::Normal,
                    };
                    ParseStep::continue_with_context(state, new_context)
                } else {
                    ParseStep::error(state, ParseError::unexpected_eof())
                }
            },
            // ... 其他规则动作的实现
            _ => todo!("实现其他规则动作"),
        }
    }
    
    /// 完整的解析流程
    pub fn parse_all(mut self, rules: &RuleSet) -> ParseOutput {
        let mut state = ParserState::new();
        
        loop {
            let step = self.parse_step(state, rules);
            
            match step.result {
                StepResult::Expression(expr) => {
                    state = step.new_state.add_expression(expr);
                },
                StepResult::Continue => {
                    state = step.new_state;
                },
                StepResult::EndOfInput => {
                    return ParseOutput {
                        result: Ok(state.parsed_exprs),
                        source_text: state.consumed_text,
                    };
                },
                StepResult::Error(error) => {
                    return ParseOutput {
                        result: Err(error),
                        source_text: state.consumed_text,
                    };
                },
                StepResult::NeedMoreInput => {
                    // 在MVP阶段简化处理
                    return ParseOutput {
                        result: Err(ParseError::unexpected_eof()),
                        source_text: state.consumed_text,
                    };
                },
            }
        }
    }
}

/// 辅助函数实现
impl ParseStep {
    pub fn expression(state: ParserState, expr: SExpr) -> Self {
        Self {
            new_state: state,
            result: StepResult::Expression(expr),
            tokens_consumed: 1,
        }
    }
    
    pub fn continue_with_context(mut state: ParserState, context: ParseContext) -> Self {
        state.context_stack.push(context);
        state.depth += 1;
        Self {
            new_state: state,
            result: StepResult::Continue,
            tokens_consumed: 1,
        }
    }
    
    pub fn error(state: ParserState, error: ParseError) -> Self {
        Self {
            new_state: state,
            result: StepResult::Error(error),
            tokens_consumed: 0,
        }
    }
    
    pub fn end_of_input(state: ParserState) -> Self {
        Self {
            new_state: state,
            result: StepResult::EndOfInput,
            tokens_consumed: 0,
        }
    }
}
```

**设计优势**：
1. **纯函数式**：所有状态转换都是纯函数，便于测试和推理
2. **可组合**：规则可以自由组合和扩展
3. **类型安全**：编译时就能捕获大部分状态转换错误
4. **可调试**：每个解析步骤都有明确的状态快照
5. **可扩展**：新增语法特性只需要添加新规则，不需要修改核心引擎

