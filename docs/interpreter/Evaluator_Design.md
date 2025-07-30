# 执行器设计

## 文档状态

**当前版本**: Draft-2  
**最后更新**: 2024年  
**状态**: 实现阶段

## 设计目标

- **高效求值**：优化的表达式求值算法，支持各种表达式类型的快速分派
- **尾递归优化**：避免栈溢出的尾调用优化，支持深度递归计算
- **错误处理**：完整的运行时错误管理，提供详细的错误位置和上下文信息
- **调试支持**：详细的执行轨迹和调用栈，支持逐步执行和变量监视

## 核心数据结构

### 求值器配置 (EvalOptions)

```rust
#[derive(Debug, Clone)]
pub struct EvalOptions {
    pub tail_call_optimization: bool,     // 是否启用尾调用优化
    pub max_call_depth: usize,           // 最大调用深度限制
    pub debug_mode: bool,                // 是否启用调试模式
    pub trace_execution: bool,           // 是否记录执行轨迹
}
```

### 调用帧 (CallFrame)

```rust
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: Option<String>,   // 函数名（如果有）
    pub environment: EnvironmentId,      // 调用环境
    pub location: SourceLocation,        // 调用位置
    pub tail_position: bool,             // 是否在尾位置
}
```

### 求值上下文 (EvalContext)

```rust
#[derive(Debug)]
pub struct EvalContext {
    pub env_manager: EnvironmentManager, // 环境管理器
    pub call_stack: Vec<CallFrame>,      // 调用栈
    pub options: EvalOptions,            // 求值选项
    pub trace: Vec<TraceEntry>,          // 执行轨迹
}

#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub expression: SExpr,
    pub environment: EnvironmentId,
    pub result: Option<Value>,
    pub timestamp: u64,
}
```

### 运行时错误类型 (EvalError)

```rust
#[derive(Debug, Clone)]
pub enum EvalError {
    UndefinedVariable {
        name: String,
        location: SourceLocation,
        available_vars: Vec<String>,
    },
    TypeError {
        expected: ValueType,
        actual: ValueType,
        location: SourceLocation,
    },
    ArityMismatch {
        function_name: String,
        expected: Arity,
        actual: usize,
        location: SourceLocation,
    },
    StackOverflow {
        current_depth: usize,
        max_depth: usize,
        location: SourceLocation,
    },
    TailCallOptimizationFailed {
        reason: String,
        location: SourceLocation,
    },
    RuntimeError {
        message: String,
        location: SourceLocation,
    },
}

#[derive(Debug, Clone)]
pub enum Arity {
    Exact(usize),                        // 精确参数数量
    AtLeast(usize),                      // 至少参数数量
    Range(usize, usize),                 // 参数数量范围
}
```

### 内置函数特征 (BuiltinFunction)

```rust
pub trait BuiltinFunction {
    fn name(&self) -> &str;
    fn arity(&self) -> Arity;
    fn call(&self, args: &[Value], context: &mut EvalContext) -> Result<Value, EvalError>;
    fn is_special_form(&self) -> bool { false }
    fn description(&self) -> &str;
}
```

## 核心函数接口

### 主要求值函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `eval_expression` | `expr: &SExpr`, `env: EnvironmentId`, `context: &mut EvalContext` | `Result<Value, EvalError>` | 主要求值函数，对任意表达式进行求值 |
| `eval_program` | `program: &[SExpr]`, `env: EnvironmentId`, `context: &mut EvalContext` | `Result<Value, EvalError>` | 求值程序（表达式序列），返回最后一个表达式的值 |

### 特殊形式求值函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `eval_if` | `condition: &SExpr`, `then_expr: &SExpr`, `else_expr: Option<&SExpr>`, `env: EnvironmentId`, `context: &mut EvalContext` | `Result<Value, EvalError>` | 求值条件表达式 |
| `eval_lambda` | `params: &[String]`, `body: &SExpr`, `env: EnvironmentId`, `context: &EvalContext` | `Result<Value, EvalError>` | 创建闭包函数 |
| `eval_define` | `name: &str`, `value: &SExpr`, `env: EnvironmentId`, `context: &mut EvalContext` | `Result<Value, EvalError>` | 定义变量或函数 |

### 函数调用函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `apply_function` | `function: &Value`, `args: &[SExpr]`, `env: EnvironmentId`, `context: &mut EvalContext` | `Result<Value, EvalError>` | 应用函数到参数列表 |
| `call_builtin` | `builtin: &dyn BuiltinFunction`, `args: &[Value]`, `context: &mut EvalContext` | `Result<Value, EvalError>` | 调用内置函数 |

### 环境和变量管理函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `lookup_variable` | `name: &str`, `env: EnvironmentId`, `context: &EvalContext` | `Result<Value, EvalError>` | 查找变量值 |
| `define_variable` | `name: String`, `value: Value`, `env: EnvironmentId`, `context: &mut EvalContext` | `Result<(), EvalError>` | 在环境中定义变量 |

### 错误处理和调试函数 (对外接口)

| 函数名 | 参数 | 返回值 | 描述 |
|--------|------|--------|------|
| `format_eval_error` | `error: &EvalError`, `context: &EvalContext` | `String` | 格式化求值错误为用户友好的消息 |
| `get_call_stack_trace` | `context: &EvalContext` | `Vec<String>` | 获取当前调用栈的字符串表示 |

## 设计考虑

### 尾递归优化实现

1. **尾位置检测** - 识别函数调用是否在尾位置
2. **栈帧复用** - 复用当前栈帧而非创建新栈帧
3. **迭代实现** - 将递归调用转换为迭代循环
4. **环境管理** - 正确处理尾调用中的环境切换

### 错误处理策略

1. **位置保持** - 在错误中保持原始表达式位置信息
2. **上下文收集** - 收集错误发生时的环境和调用栈信息
3. **用户友好** - 提供清晰的错误消息和修复建议
4. **错误恢复** - 支持从某些运行时错误中恢复

### 性能优化考虑

1. **表达式缓存** - 缓存常量表达式的求值结果
2. **符号表优化** - 优化变量查找的性能
3. **内存管理** - 有效管理值的生命周期
4. **惰性求值** - 在适当情况下使用惰性求值

## 待解决问题

### TODO-1: 复杂尾递归优化

**问题**: 相互递归函数和复杂控制流的尾调用优化
**影响**: 某些递归模式无法优化，可能导致栈溢出
**解决方向**: 实现蹦床技术和连续传递风格转换

### TODO-2: 调试器集成

**问题**: 如何与外部调试器集成，提供断点和单步执行
**影响**: 调试体验不够完善
**解决方向**: 设计调试器接口，支持断点设置和状态查询

### TODO-3: 错误恢复机制

**问题**: 运行时错误后的状态恢复和继续执行
**影响**: 错误后无法继续交互式会话
**解决方向**: 实现错误隔离和状态回滚机制

### TODO-4: 值类型优化

**问题**: 值的拷贝和移动语义优化
**影响**: 大量值拷贝可能影响性能
**解决方向**: 引入值的引用语义和写时拷贝机制

### TODO-5: 并发求值支持

**问题**: 多线程环境下的安全求值
**影响**: 无法利用多核性能进行并行计算
**解决方向**: 设计线程安全的环境管理和值共享机制
