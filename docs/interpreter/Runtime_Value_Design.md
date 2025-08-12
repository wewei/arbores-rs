# Runtime Value 设计

## 概述

为了清晰分离解析阶段和运行时阶段的概念，我们将创建独立的 `RuntimeValue` 类型来表示运行时的值，而保持 `SExpr` 作为纯粹的语法结构。

## 设计动机

### 问题
- `SExpr` 既用于表示解析结果，又用于表示运行时值
- 运行时特有的概念（如闭包）污染了语法结构的纯粹性
- 类型系统无法明确区分解析时和运行时的值

### 解决方案
创建两个独立的类型层次：
1. `SExpr` - 纯粹的语法结构（解析阶段）
2. `RuntimeValue` - 运行时值（求值阶段）

## 类型定义

### SExpr（语法结构）
```rust
/// 纯粹的语法结构，表示从源码解析出的 S 表达式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SExpr {
    Atom(AtomValue),
    Cons { 
        car: Box<SExpr>, 
        cdr: Box<SExpr>,
        span: Option<Span>,  // 源码位置信息
    },
    Nil,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomValue {
    Number(OrderedFloat<f64>),  // 使用 OrderedFloat 支持 Hash
    String(String),
    Symbol(String),
    Boolean(bool),
}
```

### RuntimeValue（运行时值）
```rust
/// 运行时值，表示求值过程中和求值结果的所有可能值
#[derive(Debug, Clone)]
pub enum RuntimeValue {
    // === 基础数据类型 ===
    Number(f64),
    String(String),
    Symbol(String),
    Boolean(bool),
    
    // === 复合数据类型 ===
    Pair(Box<RuntimeValue>, Box<RuntimeValue>),
    List(Vec<RuntimeValue>),
    Vector(Vec<RuntimeValue>),
    
    // === 函数类型 ===
    Closure {
        parameters: Vec<Parameter>,
        body: SExpr,  // 函数体保持为语法结构
        captured_env: Environment,
        name: Option<String>,  // 用于递归函数
    },
    
    BuiltinFunction {
        name: String,
        arity: Arity,
        func: fn(&[RuntimeValue]) -> Result<RuntimeValue, EvaluateError>,
    },
    
    // === Arbores 特有类型 ===
    Knowledge {
        id: String,
        content: Box<RuntimeValue>,
        metadata: HashMap<String, RuntimeValue>,
    },
    
    // === 特殊值 ===
    Nil,
    Undefined,
    Unspecified,  // 用于语句的返回值
}

#[derive(Debug, Clone)]
pub enum Parameter {
    Required(String),
    Optional(String, RuntimeValue),  // 默认值
    Rest(String),  // 变长参数
}

#[derive(Debug, Clone)]
pub enum Arity {
    Exact(usize),
    AtLeast(usize),
    Range(usize, usize),
}
```

## 转换接口

### SExpr → RuntimeValue
```rust
impl From<SExpr> for RuntimeValue {
    fn from(expr: SExpr) -> Self {
        match expr {
            SExpr::Atom(AtomValue::Number(n)) => RuntimeValue::Number(n.into()),
            SExpr::Atom(AtomValue::String(s)) => RuntimeValue::String(s),
            SExpr::Atom(AtomValue::Symbol(s)) => RuntimeValue::Symbol(s),
            SExpr::Atom(AtomValue::Boolean(b)) => RuntimeValue::Boolean(b),
            SExpr::Cons { car, cdr, .. } => {
                RuntimeValue::Pair(
                    Box::new((*car).into()),
                    Box::new((*cdr).into())
                )
            },
            SExpr::Nil => RuntimeValue::Nil,
        }
    }
}
```

### RuntimeValue → SExpr（部分）
```rust
impl RuntimeValue {
    /// 将运行时值转换回语法结构（如果可能）
    /// 用于 quote、宏展开等场景
    pub fn to_sexpr(&self) -> Result<SExpr, ConversionError> {
        match self {
            RuntimeValue::Number(n) => Ok(SExpr::Atom(AtomValue::Number((*n).into()))),
            RuntimeValue::String(s) => Ok(SExpr::Atom(AtomValue::String(s.clone()))),
            RuntimeValue::Symbol(s) => Ok(SExpr::Atom(AtomValue::Symbol(s.clone()))),
            RuntimeValue::Boolean(b) => Ok(SExpr::Atom(AtomValue::Boolean(*b))),
            RuntimeValue::Pair(car, cdr) => Ok(SExpr::Cons {
                car: Box::new(car.to_sexpr()?),
                cdr: Box::new(cdr.to_sexpr()?),
                span: None,
            }),
            RuntimeValue::Nil => Ok(SExpr::Nil),
            
            // 运行时特有的值无法转换
            RuntimeValue::Closure { name, .. } => {
                Err(ConversionError::RuntimeOnlyValue(
                    format!("closure {}", name.as_deref().unwrap_or("<anonymous>"))
                ))
            },
            RuntimeValue::BuiltinFunction { name, .. } => {
                Err(ConversionError::RuntimeOnlyValue(format!("builtin function {}", name)))
            },
            _ => Err(ConversionError::RuntimeOnlyValue("special value".to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversionError {
    RuntimeOnlyValue(String),
}
```

## 求值器接口更新

### 核心接口
```rust
/// 主求值函数
pub fn evaluate(expr: SExpr, env: Environment) -> Result<RuntimeValue, EvaluateError>;

/// 单步求值函数
pub fn evaluate_step(state: EvalState) -> EvaluateResult;

/// 求值状态
pub struct EvalState {
    frame: Frame,
    expr: SExpr,  // 待求值的语法结构
    tail_context: TailContext,
}

/// 求值结果
pub enum EvaluateResult {
    Completed(RuntimeValue),  // 返回运行时值
    Continue(EvalState),
    Error(EvaluateError),
}

/// 调用栈帧
pub struct Frame {
    env: Environment,
    continuation: Box<dyn Fn(RuntimeValue) -> EvaluateResult>,  // 接受运行时值
    parent: Option<Box<Frame>>,
}
```

### 环境管理
```rust
/// 环境存储运行时值
pub struct Environment {
    bindings: HashMap<String, RuntimeValue>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn define(&mut self, name: String, value: RuntimeValue) { /* ... */ }
    pub fn lookup(&self, name: &str) -> Option<&RuntimeValue> { /* ... */ }
    pub fn set(&mut self, name: &str, value: RuntimeValue) -> Result<(), EvaluateError> { /* ... */ }
}
```

## 特殊形式实现

### Lambda
```rust
fn evaluate_lambda_special_form(state: EvalState, args: &SExpr) -> EvaluateResult {
    let (params, body) = parse_lambda_args(args)?;
    let param_list = parse_parameter_list(&params)?;
    
    // 创建闭包对象（运行时值）
    let closure = RuntimeValue::Closure {
        parameters: param_list,
        body: body.clone(),  // 保持为语法结构
        captured_env: state.frame.env.clone(),
        name: None,
    };
    
    // 直接返回闭包
    (state.frame.continuation)(closure)
}
```

### Define
```rust
fn evaluate_define_special_form(state: EvalState, args: &SExpr) -> EvaluateResult {
    // ... 解析语法 ...
    
    let define_continuation = Box::new(move |evaluated_value: RuntimeValue| -> EvaluateResult {
        // 在环境中定义运行时值
        state.frame.env.define(name.clone(), evaluated_value);
        (state.frame.continuation)(RuntimeValue::Unspecified)
    });
    
    // ... 继续求值 ...
}
```

## 优势分析

### 1. 概念清晰
- **SExpr**：纯粹的语法结构，不包含运行时概念
- **RuntimeValue**：完整的运行时值系统

### 2. 类型安全
- 编译时强制区分解析时和运行时的值
- 防止在错误的阶段使用错误的类型

### 3. 可扩展性
- 可以独立扩展语法结构和运行时值
- 新的运行时概念不会影响解析器

### 4. 性能优化
- SExpr 可以实现 Hash 和 Eq，用于缓存和比较
- RuntimeValue 专注于运行时性能

## 迁移策略

### 阶段1：定义新类型
1. 创建 `RuntimeValue` 类型
2. 实现转换接口
3. 添加单元测试

### 阶段2：更新核心组件
1. 更新 Environment 存储 RuntimeValue
2. 更新 Evaluator 接口
3. 更新 Frame continuation 类型

### 阶段3：更新特殊形式
1. 逐个更新特殊形式实现
2. 更新内置函数签名
3. 更新错误处理

### 阶段4：清理和优化
1. 移除 SExpr 中的运行时概念
2. 优化转换性能
3. 完善文档

## 示例代码

### 使用示例
```rust
// 解析阶段
let source = "(lambda (x) (+ x 1))";
let sexpr = Parser::parse(source)?;  // 返回 SExpr

// 求值阶段
let runtime_value = evaluate(sexpr, global_env)?;  // 返回 RuntimeValue

// 结果是一个闭包
match runtime_value {
    RuntimeValue::Closure { parameters, body, .. } => {
        println!("Created closure with {} parameters", parameters.len());
        // body 仍然是 SExpr，可以在调用时求值
    },
    _ => unreachable!(),
}
```

## 总结

这个设计清晰地分离了解析时和运行时的概念，提供了更好的类型安全性和可维护性。虽然增加了一些复杂度，但长期来看有利于系统的清晰性和可扩展性。
