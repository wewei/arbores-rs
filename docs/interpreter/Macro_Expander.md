# 宏展开器设计

## 文档状态

**当前版本**: Draft-2  
**最后更新**: 2024年  
**状态**: 实现阶段

## 设计目标

- **卫生宏支持**：避免变量名冲突和意外捕获，通过词汇作用域保持实现
- **模式匹配**：强大的语法模式匹配能力，支持 syntax-rules 模式
- **嵌套展开**：支持宏内调用其他宏，带深度限制防止无限递归  
- **调试友好**：保持宏展开的追踪信息和原始位置信息

## 核心数据结构

### 宏值类型 (MacroValue)

```rust
#[derive(Debug, Clone)]
pub enum MacroValue {
    Base(Value),                           // 普通值
    Macro { 
        name: String, 
        params: Vec<String>, 
        body: SExpr, 
        env: EnvironmentId 
    },
    SyntaxRules { 
        literals: Vec<String>, 
        rules: Vec<SyntaxRule> 
    },
    SyntaxTransformer(fn(&[SExpr]) -> Result<SExpr, MacroError>),
}

#[derive(Debug, Clone)]
pub struct SyntaxRule {
    pub pattern: Pattern,
    pub template: Template,
}
```

### 模式匹配类型 (Pattern)

```rust
#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),                    // 匹配标识符
    Literal(Value),                        // 匹配字面值
    List(Vec<Pattern>),                   // 匹配列表
    Ellipsis(Box<Pattern>),               // 匹配零个或多个
}

#[derive(Debug, Clone)]  
pub enum Template {
    Identifier(String),                    // 模板标识符
    Literal(Value),                        // 模板字面值
    List(Vec<Template>),                  // 模板列表
    Substitution(String),                 // 模式变量替换
    EllipsisExpansion(Box<Template>),     // 椭圆展开
}
```

### 展开上下文 (ExpansionContext)

```rust
#[derive(Debug)]
pub struct ExpansionContext {
    pub depth: usize,                     // 当前展开深度
    pub max_depth: usize,                 // 最大展开深度限制
    pub env_manager: EnvironmentManager,  // 环境管理器
    pub trace: Vec<MacroCallInfo>,        // 展开轨迹
}

#[derive(Debug, Clone)]
pub struct MacroCallInfo {
    pub macro_name: String,
    pub call_site: SourceLocation,
    pub expanded_to: Option<SExpr>,
}
```

### 模式绑定 (PatternBindings)

```rust
#[derive(Debug, Clone)]
pub struct PatternBindings {
    pub simple: HashMap<String, SExpr>,           // 简单绑定
    pub ellipsis: HashMap<String, Vec<SExpr>>,    // 椭圆绑定
}
```

### 宏错误类型 (MacroError)

```rust
#[derive(Debug, Clone)]
pub enum MacroError {
    PatternMatchFailed {
        pattern: Pattern,
        input: SExpr,
        location: SourceLocation,
    },
    ExpansionDepthExceeded {
        current_depth: usize,
        max_depth: usize,
        location: SourceLocation,
    },
    UndefinedMacro {
        name: String,
        location: SourceLocation,
    },
    InvalidSyntaxRules {
        reason: String,
        location: SourceLocation,
    },
    CircularMacroDefinition {
        cycle: Vec<String>,
        location: SourceLocation,
    },
}
```

## 核心函数接口

### 宏展开函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `expand_macro` | `expr: SExpr`, `context: &mut ExpansionContext` | `Result<SExpr, MacroError>` | 主要宏展开函数，递归展开表达式中的所有宏调用 |

### 宏定义函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `define_syntax_rules_macro` | `name: String`, `literals: Vec<String>`, `rules: Vec<SyntaxRule>`, `env: EnvironmentId` | `Result<(), MacroError>` | 定义 syntax-rules 类型宏 |
| `define_traditional_macro` | `name: String`, `params: Vec<String>`, `body: SExpr`, `env: EnvironmentId` | `Result<(), MacroError>` | 定义传统参数化宏 |

### 宏查找函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `lookup_macro` | `name: &str`, `env: EnvironmentId`, `env_manager: &EnvironmentManager` | `Option<MacroValue>` | 查找环境中的宏定义 |

### 错误处理函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `format_macro_error` | `error: &MacroError`, `context: &ExpansionContext` | `String` | 格式化宏错误信息为用户友好的字符串 |

## 设计考虑

### 卫生宏实现策略

1. **标识符重命名** - 自动为宏内变量生成唯一名称
2. **环境隔离** - 宏在独立环境中展开，避免变量泄露
3. **词汇作用域保持** - 保持变量的原始绑定环境
4. **捕获避免** - 检测并避免意外的变量捕获

### 模式匹配算法优化

1. **提前失败** - 在匹配过程中尽早检测失败情况
2. **绑定去重** - 合并相同的模式变量绑定
3. **椭圆优化** - 特殊处理椭圆模式的匹配和展开
4. **缓存策略** - 缓存常用模式的匹配结果

### 性能优化策略

1. **延迟展开** - 只在需要时展开宏调用
2. **展开缓存** - 缓存相同输入的展开结果
3. **增量处理** - 只重新展开变化的部分
4. **编译时优化** - 在编译期完成宏展开

## 待解决问题

### TODO-1: 椭圆模式复杂性

**问题**: 嵌套椭圆模式的匹配和展开逻辑复杂
**影响**: 可能导致错误的匹配结果或展开失败
**解决方向**: 设计递归椭圆处理算法，建立绑定验证机制

### TODO-2: 宏展开追踪

**问题**: 如何有效追踪多层宏展开的过程
**影响**: 调试困难，错误定位不准确
**解决方向**: 建立展开树结构，保持原始位置映射

### TODO-3: 过程式宏支持

**问题**: syntax-transformer 类型宏的 FFI 边界处理
**影响**: 限制宏的表达能力
**解决方向**: 设计安全的函数指针包装，建立类型安全边界

### TODO-4: 宏作用域管理

**问题**: 宏定义的可见性和生命周期管理
**影响**: 可能出现宏定义覆盖或意外可见性
**解决方向**: 建立宏命名空间机制，明确宏的作用域规则

### TODO-5: 循环依赖检测

**问题**: 宏之间的循环调用检测算法效率
**影响**: 无法及时发现循环依赖，可能导致栈溢出
**解决方向**: 实现高效的依赖图分析算法
