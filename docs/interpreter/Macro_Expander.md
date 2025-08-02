# 宏展开器设计

状态：Draft-1

## 概述

宏展开器模块负责处理Scheme语言的宏定义和宏展开，实现从S表达式到S表达式的语法转换。核心功能是接收包含宏调用的S表达式，展开其中的宏调用，返回完全展开后的S表达式。

## 模块职责（功能性需求）

- **宏定义管理**：支持syntax-rules、define-macro等宏定义形式
- **模式匹配**：实现强大的语法模式匹配和模板展开
- **递归展开**：处理嵌套宏调用和递归宏定义
- **环境集成**：与环境管理器协作，管理宏的作用域和可见性
- **错误处理**：提供详细的宏展开错误信息和位置追踪

## 设计目标（非功能性需求）

- **卫生宏支持**：避免变量名冲突和意外捕获
- **性能优化**：高效的模式匹配和展开算法
- **调试友好**：保持宏展开的追踪信息和原始位置信息
- **递归安全**：防止无限递归和栈溢出

## 关键数据类型

### MacroValue

宏值的代数数据类型：

```rust
/// 宏值 - 表示不同类型的宏定义
#[derive(Debug, Clone)]
pub enum MacroValue {
    /// syntax-rules 风格的宏（包括用户定义和内置宏）
    SyntaxRules {
        name: String,
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        is_builtin: bool, // 标记是否为内置宏
    },
    /// 传统 define-macro 风格的宏
    DefineMacro {
        name: String,
        params: Vec<String>,
        body: SExpr,
        env_id: EnvironmentId,
    },
}
```

### SyntaxRule

语法规则的模式匹配定义：

```rust
/// 语法规则 - 包含模式和模板
#[derive(Debug, Clone)]
pub struct SyntaxRule {
    pub pattern: Pattern,
    pub template: Template,
}
```

### Pattern

模式匹配的模式定义：

```rust
/// 模式 - 用于匹配输入S表达式
#[derive(Debug, Clone)]
pub enum Pattern {
    /// 字面值模式
    Literal(Value),
    /// 标识符模式  
    Identifier(String),
    /// 列表模式
    List(Vec<Pattern>),
    /// 椭圆模式（零个或多个）
    Ellipsis(Box<Pattern>),
    /// 通配符模式
    Wildcard,
}
```

### Template

模板定义：

```rust
/// 模板 - 用于生成输出S表达式
#[derive(Debug, Clone)]
pub enum Template {
    /// 字面值模板
    Literal(Value),
    /// 标识符模板
    Identifier(String),
    /// 列表模板
    List(Vec<Template>),
    /// 椭圆展开模板
    EllipsisExpansion(Box<Template>),
    /// 模式变量替换
    PatternVariable(String),
}
```

### ExpansionContext

宏展开上下文：

```rust
/// 宏展开上下文 - 管理展开过程的状态
#[derive(Debug)]
pub struct ExpansionContext {
    /// 当前展开深度
    pub depth: usize,
    /// 最大展开深度限制
    pub max_depth: usize,
    /// 环境管理器引用
    pub env_manager: Rc<RefCell<EnvironmentManager<MacroValue>>>,
    /// 当前环境ID
    pub current_env: EnvironmentId,
    /// 展开追踪信息
    pub trace: Vec<ExpansionTrace>,
}
```

### ExpansionTrace

展开追踪信息：

```rust
/// 展开追踪 - 记录宏展开过程
#[derive(Debug, Clone)]
pub struct ExpansionTrace {
    pub macro_name: String,
    pub input_expr: SExpr,
    pub output_expr: SExpr,
    pub call_site_span: Rc<Span>,
}
```

### PatternBindings

模式变量绑定：

```rust
/// 模式绑定 - 存储模式匹配的变量绑定
#[derive(Debug, Clone)]
pub struct PatternBindings {
    /// 简单变量绑定
    pub simple: HashMap<String, SExpr>,
    /// 椭圆变量绑定
    pub ellipsis: HashMap<String, Vec<SExpr>>,
}
```

### MacroError

宏展开错误类型：

```rust
/// 宏展开错误 - 表示宏展开过程中的各种错误
#[derive(Debug, Clone)]
pub enum MacroError {
    /// 未定义的宏
    UndefinedMacro {
        name: String,
        span: Rc<Span>,
    },
    /// 模式匹配失败
    PatternMatchFailed {
        pattern: Pattern,
        input: SExpr,
        span: Rc<Span>,
    },
    /// 展开深度超限
    ExpansionDepthExceeded {
        current_depth: usize,
        max_depth: usize,
        span: Rc<Span>,
    },
    /// 无效的语法规则
    InvalidSyntaxRule {
        reason: String,
        span: Rc<Span>,
    },
    /// 模板实例化失败
    TemplateInstantiationFailed {
        reason: String,
        span: Rc<Span>,
    },
    /// 环境错误
    EnvironmentError {
        env_error: EnvironmentError,
        span: Rc<Span>,
    },
}
```
## 核心函数接口（对外接口）

**重要说明**：本节只记录对外暴露的主要接口函数，不包括内部实现函数、私有方法和辅助函数。

### expand_macro

宏展开的主要入口函数，将包含宏调用的S表达式展开为完全展开的S表达式。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| expr | `SExpr` | 待展开的S表达式 |
| context | `&mut ExpansionContext` | 宏展开上下文 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `Result<SExpr, MacroError>` | 展开后的S表达式或错误 |

### create_expansion_context

创建宏展开上下文的工厂函数。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| env_manager | `Rc<RefCell<EnvironmentManager<MacroValue>>>` | 环境管理器 |
| current_env | `EnvironmentId` | 当前环境ID |
| max_depth | `usize` | 最大展开深度 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `ExpansionContext` | 新的展开上下文 |

### define_syntax_rules_macro

定义syntax-rules风格的宏。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| context | `&mut ExpansionContext` | 宏展开上下文 |
| name | `String` | 宏名称 |
| literals | `Vec<String>` | 字面值标识符列表 |
| rules | `Vec<SyntaxRule>` | 语法规则列表 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `Result<(), MacroError>` | 成功或错误 |

### define_traditional_macro

定义传统define-macro风格的宏。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| context | `&mut ExpansionContext` | 宏展开上下文 |
| name | `String` | 宏名称 |
| params | `Vec<String>` | 参数列表 |
| body | `SExpr` | 宏体 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `Result<(), MacroError>` | 成功或错误 |

### lookup_macro

在环境中查找宏定义。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| context | `&ExpansionContext` | 宏展开上下文 |
| name | `&str` | 宏名称 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `Option<MacroValue>` | 宏定义或None |

### expand_from_string

从字符串源代码直接进行宏展开的便利函数。

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| source | `&str` | 源代码字符串 |
| context | `&mut ExpansionContext` | 宏展开上下文 |

#### 返回值
| 类型 | 描述 |
|------|------|
| `Result<Vec<SExpr>, MacroError>` | 展开后的S表达式列表或错误 |

## 关键设计问题

### 问题：模式匹配算法的效率优化和复杂模式处理

TODO

### 问题：宏展开过程中的位置信息保持和错误追踪

TODO

### 问题：椭圆模式的匹配和展开算法设计

TODO

### 问题：卫生宏的实现策略和变量名冲突避免

TODO

### 问题：宏展开的循环依赖检测和防止无限递归

TODO

### 问题：与环境管理器的集成和宏作用域管理

TODO
