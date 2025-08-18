# Evaluator 设计

状态：Draft-4

## 概述

Evaluator（求值器）是 Arbores 解释器的核心组件，负责执行已解析的 S 表达式并产生计算结果。它接收来自 Parser 的抽象语法树，在指定的环境中求值表达式，并支持 Scheme 语言的所有核心语义，包括特殊形式、函数调用、变量绑定等。

## 模块职责（功能性需求）

- **表达式求值**：对各种类型的 S 表达式进行求值，包括自求值表达式、变量引用、函数调用等
- **特殊形式处理**：实现 Scheme 核心特殊形式（`quote`、`if`、`lambda`、`define`、`let` 等）
- **函数调用机制**：支持内置函数和用户自定义函数的调用
- **环境管理集成**：与环境管理系统协作，处理变量绑定和作用域
- **错误处理和诊断**：提供详细的运行时错误信息和调试支持
- **位置信息传播**：在求值过程中保持和传播源码位置信息
- **调用栈管理**：维护函数调用栈以支持错误报告和调试
- **Arbores API 支持**：执行 Arbores 特有的知识库操作函数

## 设计目标（非功能性需求）

- **正确性**：严格按照 Scheme R7RS 语义执行代码
- **性能**：高效的求值性能，支持尾递归优化
- **可扩展性**：易于添加新的特殊形式和内置函数
- **调试友好**：提供丰富的调试信息和错误诊断
- **线程安全**：支持多线程环境下的安全求值（未来需求）

## 源代码结构

Evaluator 模块采用分层模块化设计，将不同的职责分离到独立的文件中，便于维护和扩展：

```
src/interpreter/evaluator/
├── mod.rs                    # 模块入口，导出公共接口
├── types/                    # 核心数据类型定义
│   ├── mod.rs               # 类型模块入口
│   ├── runtime_object.rs    # 运行时对象核心定义
│   ├── lambda.rs            # Lambda 函数定义
│   ├── builtin_function.rs  # 内置函数定义
│   ├── mutable_cons.rs      # 可变列表定义
│   ├── mutable_vector.rs    # 可变向量定义
│   ├── environment.rs       # 环境结构定义
│   ├── frame.rs             # 调用栈帧定义
│   ├── continuation.rs      # 续延定义
│   ├── evaluation_error.rs  # 求值错误类型
│   ├── evaluation_result.rs # 求值结果类型
│   ├── eval_state.rs        # 求值状态定义
│   └── string_ref.rs        # 字符串引用类型
├── engine.rs                 # 主求值引擎（evaluate, evaluate_step）
├── function_call.rs          # 函数调用机制
├── state.rs                  # 状态管理（EvalState, Frame 相关操作）
├── special_forms/            # 特殊形式处理
│   ├── mod.rs               # 特殊形式模块入口
│   ├── basic.rs             # 基本特殊形式（define, lambda, if, quote, set!）
│   ├── control.rs           # 控制流特殊形式（begin, cond, case, and, or, let）
│   ├── iteration.rs         # 迭代特殊形式（do, for-each, map, call/cc）
│   ├── macros.rs            # 宏系统（define-syntax, syntax-rules）
│   └── modules.rs           # 模块系统（define-module, import, export）
├── builtins/                 # 内置函数
│   ├── mod.rs               # 内置函数模块入口
│   ├── arithmetic.rs        # 算术运算函数 (+, -, *, /, =, <, > 等)
│   ├── comparison.rs        # 比较函数 (eq?, equal? 等)
│   ├── list.rs              # 列表操作函数 (car, cdr, cons, list 等)
│   ├── vector.rs            # 向量操作函数 (vector, vector-ref 等)
│   ├── string.rs            # 字符串操作函数 (string-append 等)
│   ├── type_check.rs        # 类型判断函数 (number?, string? 等)
│   ├── io.rs                # 输入输出函数 (display, read 等)
│   └── system.rs            # 系统函数 (load, eval, apply 等)
└── continuation.rs           # Continuation 相关工具
```

### 模块职责分工

#### 核心模块
- **`mod.rs`**: 统一导出 `evaluate()`, `evaluate_step()` 等公共接口
- **`types/mod.rs`**: 导出所有核心数据类型
- **`engine.rs`**: 实现主求值循环和单步状态转移逻辑
- **`state.rs`**: 提供状态初始化、状态转移的辅助函数

#### 类型系统模块
- **`types/runtime_object.rs`**: 定义 `RuntimeObject` 和 `RuntimeObjectCore`，支持原子值和嵌入结构
- **`types/lambda.rs`**: Lambda 函数定义，支持静态部分和闭包分离
- **`types/builtin_function.rs`**: 内置函数定义，支持签名和实现分离
- **`types/mutable_cons.rs`**: 可变列表结构，支持 `Gc` 引用
- **`types/mutable_vector.rs`**: 可变向量结构，支持 `GcCell` 内部可变性
- **`types/environment.rs`**: 环境结构，支持链式作用域和变量绑定
- **`types/frame.rs`**: 调用栈帧，支持 `Gc` 引用和续延
- **`types/continuation.rs`**: 续延定义，支持函数式编程
- **`types/evaluation_error.rs`**: 详细的错误类型，包含位置信息
- **`types/evaluation_result.rs`**: 求值结果类型，支持三种状态
- **`types/eval_state.rs`**: 求值状态，支持尾调用优化
- **`types/string_ref.rs`**: 字符串引用，支持共享字符串

#### 求值机制
- **`function_call.rs`**: 
  - 函数调用的参数求值序列
  - 函数应用逻辑 (`apply_function`)
  - 内置函数和用户函数的统一调用接口
- **`continuation.rs`**: 
  - Continuation 构造和组合工具
  - 状态转移的 continuation 封装

#### 特殊形式处理
按功能分类组织特殊形式：
- **`special_forms/basic.rs`**: 基本特殊形式（define, lambda, if, quote, set!）
- **`special_forms/control.rs`**: 控制流特殊形式（begin, cond, case, and, or, let）
- **`special_forms/iteration.rs`**: 迭代特殊形式（do, for-each, map, call/cc）
- **`special_forms/macros.rs`**: 宏系统（define-syntax, syntax-rules）
- **`special_forms/modules.rs`**: 模块系统（define-module, import, export）

#### 内置函数
按功能分类组织内置函数：
- **`builtins/arithmetic.rs`**: 算术运算函数（+, -, *, /, mod, abs, max, min等）
- **`builtins/comparison.rs`**: 比较函数（=, <, >, eq?, equal?等）
- **`builtins/list.rs`**: 列表操作函数（car, cdr, cons, list, append等）
- **`builtins/vector.rs`**: 向量操作函数（vector, vector-ref, vector-set!等）
- **`builtins/string.rs`**: 字符串操作函数（string-append, string-length等）
- **`builtins/type_check.rs`**: 类型判断函数（number?, string?, symbol?等）
- **`builtins/io.rs`**: 输入输出函数（display, write, read等）
- **`builtins/system.rs`**: 系统函数（load, eval, apply, error等）

### 模块间依赖关系

```
engine.rs
├── types/                   # 核心类型系统
│   ├── runtime_object.rs    # 运行时对象
│   ├── eval_state.rs        # 求值状态
│   ├── evaluation_result.rs # 求值结果
│   └── evaluation_error.rs  # 求值错误
├── state.rs                 # 状态管理
├── function_call.rs         # 函数调用
└── special_forms/           # 特殊形式

function_call.rs
├── types/                   # 核心类型系统
│   ├── runtime_object.rs    # 运行时对象
│   ├── lambda.rs            # Lambda 函数
│   ├── builtin_function.rs  # 内置函数
│   └── evaluation_error.rs  # 求值错误
├── builtins/                # 内置函数
└── continuation.rs          # Continuation 支持

special_forms/*.rs
├── types/                   # 核心类型系统
│   ├── runtime_object.rs    # 运行时对象
│   ├── environment.rs       # 环境管理
│   ├── eval_state.rs        # 求值状态
│   └── evaluation_error.rs  # 求值错误
└── continuation.rs          # Continuation 支持

builtins/*.rs
├── types/                   # 核心类型系统
│   ├── runtime_object.rs    # 运行时对象
│   ├── evaluation_error.rs  # 求值错误
│   └── string_ref.rs        # 字符串引用
└── continuation.rs          # Continuation 支持
```

### 设计优势

1. **职责分离**: 每个文件专注于单一职责，便于理解和维护
2. **可扩展性**: 新增特殊形式或内置函数只需添加对应文件
3. **测试友好**: 每个模块可以独立测试
4. **重用性**: 共用的工具函数和类型定义可以被多个模块使用
5. **可读性**: 文件结构清晰反映了 Evaluator 的逻辑架构
6. **类型安全**: 完整的类型系统提供编译时错误检查
7. **内存优化**: RuntimeObject 设计支持高效的内存使用和垃圾回收
8. **模块化**: 按功能分类的模块结构便于维护和扩展

## 关键数据类型

### RuntimeObject 和 RuntimeObjectCore
```rust
/// 运行时对象核心 - 表示运行时的所有可能对象类型
/// 按照存储方式分为两大类：
/// 1. 原子值（Atomic Values）- 直接存储，无需间接引用
/// 2. 嵌入结构（Embedded Structures）- 直接嵌入，减少间接访问
#[derive(Debug, Clone, Trace, Finalize)]
pub enum RuntimeObjectCore {
    // === 1. 原子值（Atomic Values）- 直接存储 ===
    Integer(i64),           // 8 bytes
    Float(f64),             // 8 bytes
    Rational(i64, i64),     // 16 bytes
    Character(char),        // 4 bytes
    Boolean(bool),          // 1 byte
    Nil,                    // 0 bytes
    
    // === 2. 嵌入结构（Embedded Structures）- 直接嵌入 ===
    String(StringRef),      // 8 bytes (Rc<String>)
    Symbol(StringRef),      // 8 bytes (Rc<String>)
    BuiltinFunction(BuiltinFunction), // 16 bytes (直接嵌入)
    Cons(MutableCons),      // 16 bytes (直接嵌入)
    Vector(MutableVector),  // 8 bytes (直接嵌入)
    Continuation(Continuation), // 16 bytes (直接嵌入)
    Lambda(Lambda),         // 16 bytes (直接嵌入)
}

/// 运行时对象 - 包含核心对象和可选的源表达式
/// RuntimeObject 本身是一个比较小的对象，可以直接 Clone
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RuntimeObject {
    /// 核心运行时对象
    pub core: RuntimeObjectCore,
    /// 可选的源表达式，用于保存计算出该 RuntimeObject 的 SExpr
    #[unsafe_ignore_trace]
    pub source: Option<Rc<SExpr>>,
}
```

### EvalState
```rust
/// 求值状态 - 表示求值过程中的当前状态
/// 采用不可变设计，每次状态转移都产生新的状态
#[derive(Debug)]
pub struct EvalState {
    /// 当前调用栈 Frame
    pub frame: Gc<Frame>,
    /// 待求值表达式
    pub expr: Gc<RuntimeObject>,
    /// 尾调用上下文信息（用于尾调用优化）
    pub tail_context: TailContext,
    /// 当前表达式的绑定名称（如果有的话）
    /// 用于支持递归函数定义和调试信息
    pub binding_name: Option<String>,
}

/// 尾调用上下文 - 标记当前表达式是否在尾位置
#[derive(Clone, Debug, PartialEq)]
pub enum TailContext {
    TailPosition,      // 在尾位置，可以进行尾调用优化
    NonTailPosition,   // 不在尾位置，需要保留调用上下文
}
```

### Frame
```rust
/// 调用栈帧 - 支持函数调用和续延
#[derive(Debug, Trace, Finalize)]
pub struct Frame {
    /// 当前环境
    pub env: Gc<Environment>,
    /// 续延
    pub continuation: Gc<Continuation>,
    /// 父栈帧
    pub parent: Option<Gc<Frame>>,
}
```

### Continuation
```rust
/// 续延函数 trait - 替代 Box<dyn Fn> 以支持 Trace
pub trait ContinuationFn: Trace + Finalize {
    fn call(&self, value: Gc<RuntimeObject>) -> EvaluateResult;
}

/// 续延结构 - 支持 call/cc
#[derive(Clone, Trace, Finalize)]
pub struct Continuation {
    /// 续延函数 - 使用 trait object 而不是 Box<dyn Fn>
    pub func: Gc<dyn ContinuationFn>,
}
```

### Environment
```rust
/// 环境结构 - 可变的链式结构，支持变量绑定修改
#[derive(Debug, Trace, Finalize)]
pub struct Environment {
    /// 当前环境的变量绑定表
    pub bindings: HashMap<String, RuntimeObject>,
    /// 上级环境（链式结构）
    parent: Option<Gc<Environment>>,
}
```

### EvaluateError
```rust
/// 求值错误类型 - 表示求值过程中可能出现的各种错误
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluateError {
    // 语法错误
    InvalidQuoteSyntax {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidIfSyntax {
        expr: Rc<SExpr>,
        message: String,
    },
    // ... 其他语法错误
    
    // 运行时错误
    UndefinedVariable {
        expr: Rc<SExpr>,
        name: String,
    },
    UndefinedFunction {
        expr: Rc<SExpr>,
        name: String,
    },
    NotCallable {
        expr: Rc<SExpr>,
        value: String,
    },
    // ... 其他运行时错误
    
    // 系统错误
    StackOverflow {
        expr: Rc<SExpr>,
    },
    OutOfMemory {
        expr: Rc<SExpr>,
    },
    // ... 其他系统错误
}
```

### EvaluateResult
```rust
/// 求值步骤结果 - 表示单步求值的三种可能结果
#[derive(Debug)]
pub enum EvaluateResult {
    /// 求值完成，返回最终结果（运行时对象）
    Completed(Rc<RuntimeObject>),
    /// 需要继续求值，返回下一个状态
    Continue(Rc<EvalState>),
    /// 求值出错，返回错误信息
    Error(EvaluateError),
}
```

## 核心函数接口（对外接口）

**重要说明**：本节只记录对外暴露的主要接口函数，不包括内部实现函数。内部辅助函数、私有方法和实现细节不在此处描述。

### evaluate() - 主求值函数 (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| expr | Rc<SExpr> | 要求值的 S 表达式 |
| env | Gc<Environment> | 全局环境 |

#### 返回值
| 类型 | 描述 |
|------|------|
| Result<Rc<RuntimeObject>, EvaluateError> | 求值结果的运行时对象或错误信息 |

#### 说明
- 函数开始时将 `SExpr` 转换为 `Gc<RuntimeObject>`
- 转换后的运行时对象包含原始的 `SExpr` 作为 `source` 字段
- 后续的求值过程在运行时对象上进行

### evaluate_step() - 单步状态转移函数 (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | Rc<EvalState> | 当前求值状态 |

#### 返回值
| 类型 | 描述 |
|------|------|
| EvaluateResult | 三分枝结果：Completed(结果)、Continue(下一状态)、Error(错误) |

#### 说明
- `state.expr` 现在是 `Gc<RuntimeObject>` 类型，直接包含运行时对象
- 求值过程直接在运行时对象上进行，无需重复转换
- 错误信息中的 `expr` 字段从 `state.expr.source` 获取原始的 `SExpr`

## 关键设计问题

### 问题：如何初始化 EvalState？

EvalState 的初始化需要创建一个根栈帧和待求值的表达式：

1. **创建根栈帧**：
   - `env`: 使用传入的全局环境，包装为 `Gc<Environment>`
   - `continuation`: 创建一个终止续延，当求值完成时返回 `Completed` 结果
   - `parent`: 设为 `None`，表示这是最顶层的栈帧

2. **设置待求值表达式**：
   - `expr`: 使用传入的 `Rc<SExpr>`

```rust
// 根栈帧的续延实现
struct RootContinuation;

impl ContinuationFn for RootContinuation {
    fn call(&self, result: Gc<RuntimeObject>) -> EvaluateResult {
        // 根栈帧的 continuation，表示求值完成
        EvaluateResult::Completed(result)
    }
}

fn init_eval_state(expr: Gc<RuntimeObject>, env: Gc<Environment>) -> EvalState {
    let root_frame = Frame::new_root(
        env,
        Continuation {
            func: Gc::new(RootContinuation),
        }
    );
    
    EvalState::new(
        Gc::new(root_frame),
        expr,
        TailContext::TailPosition, // 顶层表达式在尾位置
        None, // 顶层表达式没有绑定名称
    )
}
```

**注意**：为了支持 `Gc`，`Continuation` 不能直接使用闭包，需要定义具体的 `ContinuationFn` 实现。

**重要设计变更**：`EvalState` 中的表达式现在是 `Gc<RuntimeObject>` 而不是 `Rc<SExpr>`，因为求值过程是在运行时对象上进行的。这需要在 `evaluate` 函数开始时将 `SExpr` 转换为 `RuntimeObject`。

这种设计的优势：
- **环境管理**：全局环境保存在根栈帧中，使用 `Gc` 进行垃圾回收
- **续延系统**：使用 `ContinuationFn` trait 和具体实现，支持 `Trace` 和 `Finalize`
- **垃圾回收兼容**：所有续延函数都支持垃圾回收，避免内存泄漏
- **类型一致性**：求值状态中的表达式与运行时对象类型保持一致
- **内存管理**：使用 `Gc` 统一管理所有运行时对象，包括状态中的表达式

### 问题：如何设计 evaluate 主循环？

evaluate 主循环采用状态机模式，反复调用 `evaluate_step` 直到完成或出错：

1. **初始化状态**：调用 `init_eval_state` 创建初始的 EvalState

2. **循环执行**：
   - 调用 `evaluate_step(current_state)`
   - 根据返回的 `EvaluateResult` 进行分支处理：
     - `Completed(result)`: 返回最终结果
     - `Continue(next_state)`: 更新当前状态，继续循环
     - `Error(error)`: 返回错误

3. **实现示例**：
```rust
fn evaluate(expr: Rc<SExpr>, env: Gc<Environment>) -> Result<Rc<RuntimeObject>, EvaluateError> {
    // 将 SExpr 转换为 RuntimeObject
    let runtime_expr = convert_sexpr_to_runtime_object(expr)?;
    let mut current_state = Rc::new(init_eval_state(runtime_expr, env));
    
    loop {
        match evaluate_step(current_state) {
            EvaluateResult::Completed(result) => return Ok(result),
            EvaluateResult::Continue(next_state) => {
                current_state = next_state;
            },
            EvaluateResult::Error(error) => return Err(error),
        }
    }
}
```

这种设计的优势：
- **可控制性**：每一步都可以被观察和调试
- **可中断性**：循环可以在任意点暂停或终止
- **尾递归友好**：状态转移不会增加调用栈深度
- **引用计数**：使用 `Rc` 管理状态的生命周期，避免不必要的克隆
- **垃圾回收**：`RuntimeObject` 和 `Frame` 使用 `Gc` 进行自动内存管理

### 问题：单步迭代时，如何判定函数调用和特殊形式？

在 `evaluate_step` 中，需要根据当前表达式的类型进行分发处理：

1. **表达式类型分析**：
   - **自求值表达式**：数字、字符串、布尔值等，直接调用 continuation
   - **符号**：变量引用，在环境中查找值
   - **列表**：可能是函数调用或特殊形式，需要进一步判断

2. **列表表达式的判定逻辑**：
   ```rust
   fn evaluate_step(state: Rc<EvalState>) -> EvaluateResult {
       match &state.expr.core {
           // 自求值表达式
           RuntimeObjectCore::Integer(n) => {
               // 直接调用 continuation
               state.frame.continuation.func.call(state.expr.clone())
           },
           RuntimeObjectCore::Float(n) => {
               state.frame.continuation.func.call(state.expr.clone())
           },
           RuntimeObjectCore::Boolean(b) => {
               state.frame.continuation.func.call(state.expr.clone())
           },
           RuntimeObjectCore::String(s) => {
               state.frame.continuation.func.call(state.expr.clone())
           },
           
           // 符号（变量引用）
           RuntimeObjectCore::Symbol(name) => {
               match state.frame.env.lookup(name) {
                   Some(value) => state.frame.continuation.func.call(Gc::new(value)),
                   None => EvaluateResult::Error(EvaluateError::UndefinedVariable {
                       expr: state.expr.source.clone().unwrap_or_else(|| {
                           Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol(name.clone()))))
                       }),
                       name: name.clone(),
                   }),
               }
           },
           
           // 列表表达式
           RuntimeObjectCore::Cons(cons) => {
               evaluate_list_expression(state, cons)
           },
           
           // 其他情况
           _ => EvaluateResult::Error(EvaluateError::InvalidExpression {
               expr: state.expr.source.clone().unwrap_or_else(|| {
                   Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("unknown".to_string()))))
               }),
               message: "Unknown expression type".to_string(),
           }),
       }
   }
   ```

3. **特殊形式判定**：
   ```rust
   fn evaluate_list_expression(state: Rc<EvalState>, car: &SExpr, cdr: &SExpr) -> EvaluateResult {
       // 检查第一个元素是否为特殊形式关键字
       if let SExprContent::Atom(Value::Symbol(operator)) = &car.content {
           match operator.as_str() {
               "quote" => evaluate_quote_special_form(state, cdr),
               "if" => evaluate_if_special_form(state, cdr),
               "lambda" => evaluate_lambda_special_form(state, cdr),
               "define" => evaluate_define_special_form(state, cdr),
               "let" => evaluate_let_special_form(state, cdr),
               // 不是特殊形式，按函数调用处理（包括 arb:create、arb:search 等内置函数）
               _ => evaluate_function_call(state, car, cdr),
           }
       } else {
           // 第一个元素不是符号，按函数调用处理（可能是 lambda 表达式）
           evaluate_function_call(state, car, cdr)
       }
   }
   ```

4. **设计优势**：
   - **清晰分发**：通过模式匹配明确处理不同表达式类型
   - **可扩展性**：新增特殊形式只需在 match 分支中添加
   - **错误处理**：未知表达式类型有明确的错误处理，包含位置信息
   - **统一接口**：所有处理函数都返回 EvaluateResult
   - **类型安全**：使用 `RuntimeObject` 统一表示所有运行时值

5. **特殊形式优先级**：
   - 特殊形式的判定优先于函数调用
   - 使用字符串匹配确保精确识别
   - Arbores API（如 arb:create、arb:search）作为内置函数通过函数调用机制处理

### 问题：如何单步迭代函数调用？

函数调用的单步迭代需要处理参数求值和函数应用两个阶段：

1. **函数调用的求值顺序**：
   - 先求值函数表达式（operator）
   - 依次求值各个参数表达式（operands）
   - 最后应用函数到参数

2. **状态转移设计**：
   ```rust
   // 函数求值完成后的续延
   struct FunctionEvalContinuation {
       frame: Gc<Frame>,
       operands: Rc<SExpr>,
   }
   
   impl ContinuationFn for FunctionEvalContinuation {
       fn call(&self, function_value: Gc<RuntimeObject>) -> EvaluateResult {
           // 当函数求值完成后，开始求值参数
           evaluate_arguments(self.frame.clone(), function_value, &self.operands, Vec::new())
       }
   }
   
   fn evaluate_function_call(state: Rc<EvalState>, operator: &SExpr, operands: &SExpr) -> EvaluateResult {
       // 阶段1：求值函数表达式
       let new_continuation = Continuation {
           func: Gc::new(FunctionEvalContinuation {
               frame: state.frame.clone(),
               operands: operands.clone(),
           }),
       };
       
       let new_frame = Frame::with_parent(
           state.frame.env.clone(),
           new_continuation,
           state.frame.as_ref().clone(),
       );
       
       EvaluateResult::Continue(Rc::new(EvalState::new(
           new_frame,
           operator.clone(),
           TailContext::NonTailPosition, // 函数求值不在尾位置
           None, // 函数求值不绑定到特定名称
       )))
   }
   ```

3. **参数求值的递归处理**：
   ```rust
   fn evaluate_arguments(
       parent_frame: Frame, 
       function_value: RuntimeValue, 
       remaining_args: &SExpr, 
       evaluated_args: Vec<RuntimeValue>
   ) -> EvaluateResult {
       match remaining_args {
           // 没有更多参数，开始函数应用
           SExprContent::Nil => {
               apply_function(parent_frame, function_value, evaluated_args)
           },
           
           // 还有参数需要求值
           SExprContent::Cons { car, cdr } => {
               let new_continuation = Box::new(move |arg_value| {
                   // 参数求值完成，继续求值下一个参数
                   let mut new_evaluated_args = evaluated_args.clone();
                   new_evaluated_args.push(arg_value);
                   evaluate_arguments(parent_frame, function_value, cdr, new_evaluated_args)
               });
               
               let new_frame = Frame {
                   env: parent_frame.env.clone(),
                   continuation: new_continuation,
                   parent: Some(Box::new(parent_frame)),
               };
               
               EvaluateResult::Continue(EvalState {
                   frame: new_frame,
                   expr: car.clone(),
                   tail_context: TailContext::NonTailPosition,
                   binding_name: None, // 参数求值不绑定到特定名称
               })
           },
           
           _ => EvaluateResult::Error(EvaluateError::InvalidArgumentList),
       }
   }
   ```

4. **函数应用阶段**：
   ```rust
   fn apply_function(
       parent_frame: Frame, 
       function_value: RuntimeValue, 
       arguments: Vec<RuntimeValue>
   ) -> EvaluateResult {
       match &function_value {
           // 内置函数
           RuntimeValue::BuiltinFunction { func, .. } => {
               match func(&arguments) {
                   Ok(result) => (parent_frame.continuation)(result),
                   Err(error) => EvaluateResult::Error(error),
               }
           },
           
           // 用户定义的 lambda 函数
           RuntimeValue::Closure { parameters, body, captured_env, .. } => {
               // 创建新环境，绑定参数
               let new_env = bind_parameters(parameters, &arguments, captured_env)?;
               
               let new_frame = Frame {
                   env: new_env,
                   continuation: parent_frame.continuation,
                   parent: parent_frame.parent,
               };
               
               EvaluateResult::Continue(EvalState {
                   frame: new_frame,
                   expr: body.clone(),
                   tail_context: TailContext::TailPosition, // 函数体在尾位置
                   binding_name: None, // 函数体求值不绑定到特定名称
               })
           },
           
           _ => EvaluateResult::Error(EvaluateError::NotCallable),
       }
   }
   ```

5. **设计优势**：
   - **严格求值顺序**：函数和参数按标准 Scheme 语义求值
   - **链式 continuation**：通过嵌套的 continuation 实现状态转移
   - **尾递归支持**：lambda 函数调用重用当前栈帧的 continuation
   - **统一处理**：内置函数和用户函数使用统一的应用机制
   - **垃圾回收兼容**：所有 continuation 都支持 `Trace` 和 `Finalize`

6. **ContinuationFn 实现模式**：
   - **根续延**：`RootContinuation` - 表示求值完成
   - **函数求值续延**：`FunctionEvalContinuation` - 函数求值完成后求值参数
   - **参数求值续延**：`ArgumentEvalContinuation` - 参数求值完成后继续下一个参数
   - **特殊形式续延**：各种特殊形式的具体续延实现
   - **尾调用续延**：`TailCallContinuation` - 支持尾调用优化

7. **尾递归优化关键**：
   - lambda 函数的函数体求值时，直接使用父栈帧的 continuation
   - 避免创建新的栈帧链，实现真正的尾调用消除

### 问题：如何单步迭代 quote 特殊形式？

`quote` 是 Scheme 中最简单的特殊形式，它阻止对表达式的求值，直接返回表达式的字面值。

1. **语法规则**：
   - `(quote expr)` 或简写为 `'expr`
   - 返回 `expr` 的字面值，不进行求值

2. **实现逻辑**：
   ```rust
   fn evaluate_quote_special_form(state: EvalState, args: &SExpr) -> EvaluateResult {
       // quote 只接受一个参数
       match args {
           SExprContent::Cons { car, cdr } => {
               // 检查是否只有一个参数
               match cdr.content {
                   SExprContent::Nil => {
                       // 将被引用的表达式转换为运行时值并返回
                       let quoted_value = RuntimeValue::from(car.clone());
                       (state.frame.continuation)(quoted_value)
                   },
                   _ => {
                       // quote 接受多个参数是错误的
                       EvaluateResult::Error(EvaluateError::InvalidQuoteSyntax)
                   }
               }
           },
           SExprContent::Nil => {
               // quote 没有参数是错误的
               EvaluateResult::Error(EvaluateError::InvalidQuoteSyntax)
           },
           _ => {
               // 参数列表格式错误
               EvaluateResult::Error(EvaluateError::InvalidArgumentList)
           }
       }
   }
   ```

3. **使用示例**：
   ```scheme
   ;; 基本用法
   (quote a)        ; 返回符号 a
   (quote (+ 1 2))  ; 返回列表 (+ 1 2)，不求值
   (quote ())       ; 返回空列表
   
   ;; 简写形式
   'a               ; 等价于 (quote a)
   '(+ 1 2)         ; 等价于 (quote (+ 1 2))
   ```

4. **设计特点**：
   - **零步骤求值**：quote 不需要创建新的求值状态，直接调用 continuation
   - **参数验证**：严格检查参数数量，确保语法正确性
   - **保持原始结构**：被引用的表达式保持其原始的 S 表达式结构
   - **支持任意表达式**：可以引用原子值、符号、列表等任何 S 表达式

5. **错误处理**：
   - `InvalidQuoteSyntax`：参数数量不正确（0个或多于1个）
   - `InvalidArgumentList`：参数列表格式错误

6. **性能优势**：
   - **常数时间**：quote 的执行时间是 O(1)，不依赖于被引用表达式的复杂度
   - **零分配**：直接返回现有的 SExpr，无需额外内存分配

### 问题：如何单步迭代 if 特殊形式？

`if` 特殊形式实现条件分支逻辑，需要先求值条件表达式，然后根据结果选择不同的分支进行求值。

1. **语法规则**：
   - `(if condition then-expr else-expr)`
   - `(if condition then-expr)` - else 分支可选，默认为未定义

2. **求值步骤**：
   - **第一步**：求值条件表达式
   - **第二步**：根据条件结果，选择 then 或 else 分支求值
   - **第三步**：返回所选分支的求值结果

3. **实现设计**：
   ```rust
   fn evaluate_if_special_form(state: EvalState, args: &SExpr) -> EvaluateResult {
       match count_list_elements(args) {
           2 | 3 => {
               // 解析参数：condition, then-expr, [else-expr]
               let (condition, rest) = extract_first_arg(args)?;
               let (then_expr, rest) = extract_first_arg(rest)?;
               let else_expr = if count_list_elements(rest) == 1 {
                   Some(extract_first_arg(rest)?.0)
               } else if count_list_elements(rest) == 0 {
                   None
               } else {
                   return EvaluateResult::Error(EvaluateError::InvalidIfSyntax);
               };

               // 创建 continuation 来处理条件求值完成后的逻辑
               let if_continuation = Box::new(move |condition_result: RuntimeValue| -> EvaluateResult {
                   if is_truthy(&condition_result) {
                       // 条件为真，求值 then 分支
                       EvaluateResult::Continue(EvalState {
                           expr: then_expr.clone(),
                           frame: state.frame.clone(), // 使用原始 frame
                           tail_context: state.tail_context.clone(), // 继承尾位置状态
                           binding_name: state.binding_name.clone(), // 继承绑定名称
                       })
                   } else {
                       // 条件为假，求值 else 分支（如果存在）
                       match else_expr {
                           Some(else_expr) => EvaluateResult::Continue(EvalState {
                               expr: else_expr.clone(),
                               frame: state.frame.clone(),
                               tail_context: state.tail_context.clone(), // 继承尾位置状态
                           }),
                           None => {
                               // 没有 else 分支，返回未定义值
                               (state.frame.continuation)(RuntimeValue::Undefined)
                           }
                       }
                   }
               });

               // 创建新的 Frame 来求值条件表达式
               let condition_frame = Frame {
                   env: state.frame.env.clone(),
                   continuation: if_continuation,
                   parent: Some(Box::new(state.frame)),
               };

               // 开始求值条件表达式
               EvaluateResult::Continue(EvalState {
                   expr: condition,
                   frame: condition_frame,
                   tail_context: TailContext::NonTailPosition, // 条件不在尾位置
                   binding_name: None, // 条件求值不绑定到特定名称
               })
           },
           _ => EvaluateResult::Error(EvaluateError::InvalidIfSyntax),
       }
   }

   fn is_truthy(value: &RuntimeValue) -> bool {
       // 在 Scheme 中，除了 #f 以外的所有值都被视为真
       match value {
           RuntimeValue::Boolean(false) => false,
           _ => true,
       }
   }
   ```

4. **辅助函数**：
   ```rust
   fn extract_first_arg(args: &SExpr) -> Result<(SExpr, &SExpr), EvaluateError> {
       match &args.content {
           SExprContent::Cons { car, cdr } => Ok((car.clone(), cdr)),
           _ => Err(EvaluateError::InvalidArgumentList),
       }
   }
   ```

5. **设计特点**：
   - **延迟求值**：只求值条件和选中的分支，未选中的分支不会被求值
   - **尾调用优化友好**：选中的分支在相同的 Frame 中求值，支持尾调用优化
   - **标准语义**：遵循 Scheme 的真值判断规则（除 #f 外都是真）
   - **错误处理**：对语法错误提供明确的错误信息

6. **使用示例**：
   ```scheme
   (if #t 'yes 'no)           ; 返回 'yes
   (if #f 'yes 'no)           ; 返回 'no
   (if (> 3 2) 'greater)      ; 返回 'greater
   (if (< 3 2) 'less)         ; 返回未定义值（无 else 分支）
   ```

7. **求值流程**：
   - **状态1**：创建条件求值的 Frame，求值条件表达式
   - **状态2**：条件求值完成，根据结果选择分支，在原 Frame 中求值选中分支
   - **完成**：分支求值完成，返回最终结果

### 问题：如何单步迭代 lambda 特殊形式？

`lambda` 特殊形式创建匿名函数（闭包），捕获当前环境并创建可调用的函数对象。

1. **语法规则**：
   - `(lambda (param1 param2 ...) body)`
   - `(lambda param body)` - 单参数简写形式
   - `(lambda () body)` - 无参数形式

2. **求值特点**：
   - **立即完成**：lambda 不需要多步求值，直接创建闭包对象
   - **环境捕获**：捕获定义时的环境作为闭包环境
   - **延迟求值**：函数体在调用时才求值，定义时不求值

3. **实现设计**：
   ```rust
   fn evaluate_lambda_special_form(state: EvalState, args: &SExpr) -> EvaluateResult {
       match count_list_elements(args) {
           2 => {
               let (params, rest) = extract_first_arg(args)?;
               let (body, rest) = extract_first_arg(rest)?;
               
               // 验证参数列表格式
               let param_names = parse_parameter_list(&params)?;
               
               // 创建闭包对象（运行时值）
               let closure = RuntimeValue::Closure {
                   parameters: param_names,
                   body: body.clone(),
                   captured_env: state.frame.env.clone(), // 捕获当前环境
                   name: state.binding_name.clone(), // 使用绑定名称（如果有的话）
               };
               
               // 直接返回闭包，无需状态转移
               (state.frame.continuation)(closure)
           },
           _ => EvaluateResult::Error(EvaluateError::InvalidLambdaSyntax),
       }
   }

   fn parse_parameter_list(params: &SExpr) -> Result<Vec<String>, EvaluateError> {
       match &params.content {
           SExprContent::Nil => Ok(vec![]), // 无参数
           SExprContent::Symbol(name) => Ok(vec![name.clone()]), // 单参数简写
           SExprContent::Cons { .. } => {
               let mut param_names = Vec::new();
               let mut current = params;
               
               loop {
                   match &current.content {
                       SExprContent::Cons { car, cdr } => {
                           // 参数必须是符号
                           match &car.content {
                               SExprContent::Symbol(name) => {
                                   param_names.push(name.clone());
                                   current = cdr;
                               },
                               _ => return Err(EvaluateError::InvalidParameterName),
                           }
                       },
                       SExprContent::Nil => break, // 正常结束
                       SExprContent::Symbol(rest_param) => {
                           // 变长参数：(lambda (a b . rest) body)
                           param_names.push(format!("...{}", rest_param));
                           break;
                       },
                       _ => return Err(EvaluateError::InvalidParameterList),
                   }
               }
               
               Ok(param_names)
           },
           _ => Err(EvaluateError::InvalidParameterList),
       }
   }
   ```

4. **数据类型扩展**：
   ```rust
   // 在 SExprContent 中添加 Closure 变体
   pub enum SExprContent {
       // ...existing variants...
       Closure {
           parameters: Vec<String>,
           body: SExpr,
           captured_env: Environment, // 捕获的环境
       },
   }
   ```

5. **闭包调用实现**：
   ```rust
   fn apply_closure(closure: &SExpr, args: Vec<SExpr>, current_frame: Frame) -> EvaluateResult {
       if let SExprContent::Closure { parameters, body, captured_env } = &closure.content {
           // 验证参数数量
           if parameters.len() != args.len() {
               return EvaluateResult::Error(EvaluateError::ArgumentCountMismatch);
           }
           
           // 创建新环境：captured_env + 参数绑定
           let mut new_env = captured_env.clone();
           for (param, arg) in parameters.iter().zip(args.iter()) {
               new_env.define(param.clone(), arg.clone());
           }
           
           // 创建新 Frame 来执行函数体
           let closure_frame = Frame {
               env: new_env,
               continuation: current_frame.continuation,
               parent: current_frame.parent,
           };
           
           // 求值函数体
           EvaluateResult::Continue(EvalState {
               expr: body.clone(),
               frame: closure_frame,
               tail_context: TailContext::TailPosition, // 函数体在尾位置
               binding_name: None, // 函数体求值不绑定到特定名称
           })
       } else {
           EvaluateResult::Error(EvaluateError::NotCallable)
       }
   }
   ```

6. **设计特点**：
   - **词法作用域**：闭包捕获定义时的环境，实现词法作用域
   - **立即创建**：lambda 表达式立即创建闭包，不需要多步求值
   - **延迟执行**：函数体在调用时才求值，支持递归定义
   - **环境隔离**：每次调用创建独立的环境，支持并发调用
   - **尾调用优化**：函数体在独立 Frame 中求值，支持尾调用优化

7. **使用示例**：
   ```scheme
   ;; 基本用法
   (lambda (x) (+ x 1))           ; 创建加一函数
   ((lambda (x y) (+ x y)) 3 4)   ; 立即调用，返回 7
   
   ;; 闭包捕获
   (let ((n 10))
     (lambda (x) (+ x n)))        ; 捕获变量 n
   
   ;; 高阶函数
   (lambda (f x) (f (f x)))       ; 函数接受函数作为参数
   ```

8. **性能考虑**：
   - **环境共享**：捕获的环境通过引用计数共享，避免深拷贝
   - **参数验证**：在创建时验证参数列表格式，避免运行时错误
   - **尾调用友好**：设计支持尾调用优化的调用约定

### 问题：如何单步迭代 define 特殊形式？

`define` 特殊形式用于在当前环境中定义变量或函数，支持简单变量定义和函数定义的语法糖。

1. **语法规则**：
   - `(define name value)` - 变量定义
   - `(define (name param1 param2 ...) body)` - 函数定义语法糖

2. **求值步骤**：
   - **变量定义**：先求值 value，然后在当前环境中绑定 name
   - **函数定义**：转换为 lambda 表达式，然后进行变量定义

3. **实现设计**：
   ```rust
   fn evaluate_define_special_form(state: EvalState, args: &SExpr) -> EvaluateResult {
       match count_list_elements(args) {
           2 => {
               let (first_arg, rest) = extract_first_arg(args)?;
               let (second_arg, _) = extract_first_arg(rest)?;
               
               match &first_arg.content {
                   // 变量定义：(define name value)
                   SExprContent::Symbol(name) => {
                       evaluate_variable_define(state, name.clone(), second_arg)
                   },
                   
                   // 函数定义：(define (name param1 param2 ...) body)
                   SExprContent::Cons { car, cdr } => {
                       if let SExprContent::Symbol(func_name) = &car.content {
                           evaluate_function_define(state, func_name.clone(), cdr, &second_arg)
                       } else {
                           EvaluateResult::Error(EvaluateError::InvalidDefineSyntax)
                       }
                   },
                   
                   _ => EvaluateResult::Error(EvaluateError::InvalidDefineSyntax),
               }
           },
           _ => EvaluateResult::Error(EvaluateError::InvalidDefineSyntax),
       }
   }

   fn evaluate_variable_define(state: EvalState, name: String, value_expr: SExpr) -> EvaluateResult {
       // 创建 continuation 来处理值求值完成后的定义
       let define_continuation = Box::new(move |evaluated_value: RuntimeValue| -> EvaluateResult {
           // 在当前环境中定义变量
           state.frame.env.define(name.clone(), evaluated_value.clone());
           
           // define 返回未定义值
           (state.frame.continuation)(RuntimeValue::Unspecified)
       });
       
       // 创建新的 Frame 来求值表达式
       let eval_frame = Frame {
           env: state.frame.env.clone(),
           continuation: define_continuation,
           parent: Some(Box::new(state.frame)),
       };
       
       // 开始求值 value 表达式，传递绑定名称
       EvaluateResult::Continue(EvalState {
           expr: value_expr,
           frame: eval_frame,
           tail_context: TailContext::TailPosition, // define 的值在尾位置
           binding_name: Some(name), // 传递绑定名称
       })
   }

   fn evaluate_function_define(
       state: EvalState, 
       func_name: String, 
       params: &SExpr, 
       body: &SExpr
   ) -> EvaluateResult {
       // 将函数定义转换为 lambda 表达式
       // (define (f x y) body) => (define f (lambda (x y) body))
       let lambda_expr = SExpr::new(SExprContent::Cons {
           car: SExpr::new(SExprContent::Symbol("lambda".to_string())),
           cdr: SExpr::new(SExprContent::Cons {
               car: params.clone(),
               cdr: SExpr::new(SExprContent::Cons {
                   car: body.clone(),
                   cdr: SExpr::new(SExprContent::Nil),
               }),
           }),
       });
       
       // 调用变量定义逻辑
       evaluate_variable_define(state, func_name, lambda_expr)
   }
   ```

4. **环境修改接口**：
   ```rust
   impl Environment {
       pub fn define(&mut self, name: String, value: RuntimeValue) {
           // 在当前（最顶层）环境中定义变量
           // 如果变量已存在，则更新其值
           if let Some(ref mut bindings) = self.bindings {
               bindings.insert(name, value);
           }
       }
       
       pub fn set(&mut self, name: &str, value: RuntimeValue) -> Result<(), EvaluateError> {
           // 在环境链中查找并更新已存在的变量
           if let Some(ref mut bindings) = self.bindings {
               if bindings.contains_key(name) {
                   bindings.insert(name.to_string(), value);
                   return Ok(());
               }
           }
           
           if let Some(ref mut parent) = self.parent {
               parent.set(name, value)
           } else {
               Err(EvaluateError::UndefinedVariable(name.to_string()))
           }
       }
   }
   ```

5. **设计特点**：
   - **副作用操作**：define 修改环境状态，是有副作用的操作
   - **语法糖支持**：函数定义自动转换为 lambda 表达式
   - **顶级定义**：define 总是在当前环境的顶层定义变量
   - **重定义允许**：允许重新定义已存在的变量
   - **延迟求值**：值表达式在定义时才求值

6. **使用示例**：
   ```scheme
   ;; 变量定义
   (define x 42)                  ; 定义变量 x
   (define y (+ 1 2))             ; 定义变量 y，值为表达式结果
   
   ;; 函数定义
   (define (square x) (* x x))    ; 定义函数 square
   (define (add x y) (+ x y))     ; 定义函数 add
   
   ;; 复杂函数定义
   (define (factorial n)
     (if (= n 0) 
         1 
         (* n (factorial (- n 1)))))
   ```

7. **求值流程**：
   - **变量定义**：
     - 状态1：求值 value 表达式
     - 状态2：将求值结果绑定到 name，返回结果
   - **函数定义**：
     - 状态1：构造 lambda 表达式
     - 状态2：按变量定义流程处理 lambda 表达式

8. **错误处理**：
   - `InvalidDefineSyntax`：语法格式错误
   - `InvalidParameterName`：函数参数不是符号
   - 继承值求值过程中的所有可能错误

### 问题：如何单步迭代 let 特殊形式？

`let` 特殊形式创建局部变量绑定，在新的环境中求值表达式。这是一个需要多步骤求值的复杂特殊形式。

1. **语法规则**：
   - `(let ((var1 val1) (var2 val2) ...) body)`
   - 创建新的作用域，在其中绑定局部变量

2. **求值步骤**：
   - **第一阶段**：依次求值所有绑定的值表达式
   - **第二阶段**：创建新环境，绑定所有变量
   - **第三阶段**：在新环境中求值 body

3. **实现设计**：
   ```rust
   fn evaluate_let_special_form(state: EvalState, args: &SExpr) -> EvaluateResult {
       match count_list_elements(args) {
           2 => {
               let (bindings, rest) = extract_first_arg(args)?;
               let (body, _) = extract_first_arg(rest)?;
               
               // 解析绑定列表
               let binding_pairs = parse_let_bindings(&bindings)?;
               
               // 开始求值绑定的值表达式
               evaluate_let_bindings(state, binding_pairs, body)
           },
           _ => EvaluateResult::Error(EvaluateError::InvalidLetSyntax),
       }
   }

   #[derive(Clone)]
   struct LetBinding {
       name: String,
       value_expr: SExpr,
   }

   fn parse_let_bindings(bindings: &SExpr) -> Result<Vec<LetBinding>, EvaluateError> {
       let mut result = Vec::new();
       let mut current = bindings;
       
       loop {
           match &current.content {
               SExprContent::Nil => break,
               SExprContent::Cons { car, cdr } => {
                   // 每个绑定应该是 (name value) 的形式
                   match &car.content {
                       SExprContent::Cons { car: name_expr, cdr: value_list } => {
                           if let SExprContent::Symbol(name) = &name_expr.content {
                               if let SExprContent::Cons { car: value_expr, cdr: nil } = &value_list.content {
                                   if let SExprContent::Nil = &nil.content {
                                       result.push(LetBinding {
                                           name: name.clone(),
                                           value_expr: value_expr.clone(),
                                       });
                                       current = cdr;
                                   } else {
                                       return Err(EvaluateError::InvalidLetBinding);
                                   }
                               } else {
                                   return Err(EvaluateError::InvalidLetBinding);
                               }
                           } else {
                               return Err(EvaluateError::InvalidLetBinding);
                           }
                       },
                       _ => return Err(EvaluateError::InvalidLetBinding),
                   }
               },
               _ => return Err(EvaluateError::InvalidLetSyntax),
           }
       }
       
       Ok(result)
   }

   fn evaluate_let_bindings(
       state: EvalState, 
       bindings: Vec<LetBinding>, 
       body: SExpr
   ) -> EvaluateResult {
       if bindings.is_empty() {
           // 没有绑定，直接求值 body
           EvaluateResult::Continue(EvalState {
               expr: body,
               frame: state.frame,
           })
       } else {
           // 开始求值第一个绑定的值
           evaluate_let_binding_sequence(state, bindings, Vec::new(), body, 0)
       }
   }

   fn evaluate_let_binding_sequence(
       state: EvalState,
       bindings: Vec<LetBinding>,
       evaluated_values: Vec<SExpr>,
       body: SExpr,
       current_index: usize,
   ) -> EvaluateResult {
       if current_index >= bindings.len() {
           // 所有绑定都已求值完成，创建新环境并求值 body
           let mut new_env = Environment::new_child(state.frame.env.clone());
           
           for (binding, value) in bindings.iter().zip(evaluated_values.iter()) {
               new_env.define(binding.name.clone(), value.clone());
           }
           
           let let_frame = Frame {
               env: new_env,
               continuation: state.frame.continuation,
               parent: Some(Box::new(state.frame)),
           };
           
           EvaluateResult::Continue(EvalState {
               expr: body,
               frame: let_frame,
               tail_context: TailContext::TailPosition, // let 的 body 在尾位置
               binding_name: None, // body 求值不绑定到特定名称
           })
       } else {
           // 求值当前绑定的值表达式
           let current_binding = &bindings[current_index];
           
           let binding_continuation = Box::new(move |evaluated_value: SExpr| -> EvaluateResult {
               let mut new_evaluated_values = evaluated_values.clone();
               new_evaluated_values.push(evaluated_value);
               
               // 继续求值下一个绑定
               evaluate_let_binding_sequence(
                   state.clone(),
                   bindings.clone(),
                   new_evaluated_values,
                   body.clone(),
                   current_index + 1,
               )
           });
           
           let eval_frame = Frame {
               env: state.frame.env.clone(),
               continuation: binding_continuation,
               parent: Some(Box::new(state.frame.clone())),
           };
           
           EvaluateResult::Continue(EvalState {
               expr: current_binding.value_expr.clone(),
               frame: eval_frame,
               tail_context: TailContext::TailPosition, // let 绑定的值在尾位置
               binding_name: Some(current_binding.name.clone()), // 传递绑定名称
           })
       }
   }
   ```

4. **环境管理**：
   ```rust
   impl Environment {
       pub fn new_child(parent: Environment) -> Environment {
           Environment {
               bindings: Some(HashMap::new()),
               parent: Some(Box::new(parent)),
           }
       }
   }
   ```

5. **设计特点**：
   - **顺序求值**：绑定的值表达式按顺序求值，前面的绑定不能引用后面的变量
   - **环境隔离**：let 创建新的作用域，不影响外层环境
   - **支持递归**：多个绑定可以相互引用（如果是 let* 语义）
   - **状态追踪**：需要追踪当前求值的绑定索引和已求值的结果

6. **使用示例**：
   ```scheme
   ;; 基本用法
   (let ((x 1) (y 2))
     (+ x y))                     ; 返回 3
   
   ;; 嵌套作用域
   (let ((x 1))
     (let ((x 2) (y x))
       (+ x y)))                  ; 返回 3 (内层 x=2, y=1)
   
   ;; 复杂表达式
   (let ((square (lambda (x) (* x x)))
         (n 5))
     (square n))                  ; 返回 25
   ```

7. **求值流程**：
   - **状态1-N**：依次求值每个绑定的值表达式
   - **状态N+1**：创建新环境，绑定所有变量，求值 body
   - **完成**：返回 body 的求值结果

8. **变体支持**：
   - **let***：可以扩展为支持后面的绑定引用前面的绑定
   - **letrec**：可以扩展为支持递归绑定
   - **named let**：可以扩展为支持命名的递归 let

9. **性能优化**：
   - **批量绑定**：一次性创建所有绑定，避免多次环境操作
   - **共享环境**：父环境通过引用共享，避免深拷贝
   - **尾调用优化**：body 在独立 Frame 中求值，支持尾调用优化

### 问题：如何实现尾递归优化？

尾递归优化（Tail Call Optimization, TCO）是函数式语言的重要特性，通过复用调用栈帧避免栈溢出，支持高效的递归计算。

1. **尾调用的识别**：
   - 函数的最后一个操作是调用另一个函数
   - 调用结果直接作为当前函数的返回值
   - 不需要保留当前函数的调用上下文

2. **尾位置标记**：
   ```rust
   #[derive(Clone, Debug)]
   pub enum TailContext {
       TailPosition,      // 在尾位置
       NonTailPosition,   // 不在尾位置，需要保留调用上下文
   }
   
   impl EvalState {
       pub fn with_tail_context(mut self, context: TailContext) -> Self {
           self.tail_context = context;
           self
       }
       
       pub fn is_tail_position(&self) -> bool {
           matches!(self.tail_context, TailContext::TailPosition)
       }
   }
   ```

3. **尾调用优化的函数应用**：
   ```rust
   fn apply_closure_with_tco(
       closure: &SExpr,
       args: Vec<SExpr>,
       current_frame: Frame,
       is_tail_call: bool,
   ) -> EvaluateResult {
       if let SExprContent::Closure { parameters, body, captured_env } = &closure.content {
           // 创建参数绑定环境
           let mut new_env = Environment::new_child(captured_env.clone());
           for (param, arg) in parameters.iter().zip(args.iter()) {
               new_env.define(param.clone(), arg.clone());
           }
           
           let execution_frame = if is_tail_call {
               // 尾调用：复用 continuation，避免栈增长
               Frame {
                   env: new_env,
                   continuation: current_frame.continuation,
                   parent: current_frame.parent, // 不保留当前 frame
               }
           } else {
               // 普通调用：保留调用上下文
               Frame {
                   env: new_env,
                   continuation: current_frame.continuation,
                   parent: Some(Box::new(current_frame)),
               }
           };
           
           EvaluateResult::Continue(EvalState {
               expr: body.clone(),
               frame: execution_frame,
               tail_context: TailContext::TailPosition, // 函数体在尾位置
           })
       } else {
           EvaluateResult::Error(EvaluateError::NotCallable)
       }
   }
   ```

4. **特殊形式中的尾位置传播**：
   ```rust
   fn evaluate_if_with_tco(state: EvalState, args: &SExpr) -> EvaluateResult {
       let (condition, then_expr, else_expr) = parse_if_args(args)?;
       
       let if_continuation = Box::new(move |condition_result: SExpr| -> EvaluateResult {
           if is_truthy(&condition_result) {
               // then 分支继承尾位置状态
               EvaluateResult::Continue(EvalState {
                   expr: then_expr.clone(),
                   frame: state.frame.clone(),
                   tail_context: state.tail_context.clone(), // 继承尾位置
               })
           } else if let Some(else_expr) = else_expr {
               // else 分支继承尾位置状态
               EvaluateResult::Continue(EvalState {
                   expr: else_expr.clone(),
                   frame: state.frame.clone(),
                   tail_context: state.tail_context.clone(), // 继承尾位置
               })
           } else {
               (state.frame.continuation)(SExpr::undefined())
           }
       });
       
       // 条件求值不在尾位置
       let condition_frame = Frame {
           env: state.frame.env.clone(),
           continuation: if_continuation,
           parent: Some(Box::new(state.frame)),
       };
       
       EvaluateResult::Continue(EvalState {
           expr: condition,
           frame: condition_frame,
           tail_context: TailContext::NonTailPosition, // 条件不在尾位置
           binding_name: None, // 条件求值不绑定到特定名称
       })
   }
   ```

5. **let 表达式中的尾位置处理**：
   ```rust
   fn evaluate_let_body(
       bindings: Vec<(String, SExpr)>,
       body: SExpr,
       current_frame: Frame,
       tail_context: TailContext,
   ) -> EvaluateResult {
       // 创建新环境
       let mut new_env = Environment::new_child(current_frame.env.clone());
       for (name, value) in bindings {
           new_env.define(name, value);
       }
       
       let let_frame = Frame {
           env: new_env,
           continuation: current_frame.continuation,
           parent: current_frame.parent, // 如果是尾调用，不保留 let frame
       };
       
       EvaluateResult::Continue(EvalState {
           expr: body,
           frame: let_frame,
           tail_context, // let body 继承尾位置状态
       })
   }
   ```

6. **循环结构的尾递归优化**：
   ```rust
   fn optimize_tail_recursion(state: &EvalState, closure: &SExpr) -> bool {
       // 检测是否是对同一函数的递归调用
       // 可以通过比较函数对象的标识来实现
       if let SExprContent::Closure { .. } = &closure.content {
           // 简化实现：总是尝试尾调用优化
           state.is_tail_position()
       } else {
           false
       }
   }
   ```

7. **性能优势**：
   - **常数栈空间**：尾递归函数使用 O(1) 栈空间
   - **避免栈溢出**：支持任意深度的递归调用
   - **内存效率**：及时释放不需要的调用帧
   - **优化编译**：为进一步的编译优化提供基础

8. **使用示例**：
   ```scheme
   ;; 尾递归阶乘（优化）
   (define (factorial n acc)
     (if (= n 0)
         acc                           ; 尾位置返回
         (factorial (- n 1) (* n acc)))) ; 尾递归调用
   
   ;; 普通递归阶乘（未优化）
   (define (factorial-normal n)
     (if (= n 0)
         1
         (* n (factorial-normal (- n 1))))) ; 非尾递归
   
   ;; 尾递归求和
   (define (sum-list lst acc)
     (if (null? lst)
         acc                           ; 尾位置返回
         (sum-list (cdr lst) (+ acc (car lst))))) ; 尾递归调用
   ```

9. **调试支持**：
   ```rust
   fn should_preserve_debug_info(frame: &Frame) -> bool {
       // 在调试模式下可以选择保留更多调用栈信息
       cfg!(debug_assertions) || frame.has_debug_flag()
   }
   ```

10. **实现注意事项**：
    - **正确识别**：准确判断哪些调用位置是尾位置
    - **状态传播**：确保尾位置信息正确传播
    - **环境管理**：正确处理环境的生命周期和引用
    - **错误处理**：保证错误信息的完整性和可追踪性

## 重要设计决策

### Draft-4 设计变更总结

在 Draft-4 中，我们对 `EvalState` 进行了重要的设计变更：

1. **表达式类型变更**：
   - `EvalState.expr` 从 `Rc<SExpr>` 改为 `Gc<RuntimeObject>`
   - 求值过程直接在运行时对象上进行，避免重复转换

2. **Frame 引用变更**：
   - `EvalState.frame` 从 `Rc<Frame>` 改为 `Gc<Frame>`
   - 统一使用 `Gc` 进行垃圾回收管理

3. **构造函数变更**：
   - `EvalState::new()` 现在接受 `Gc<Frame>` 和 `Gc<RuntimeObject>`
   - `init_eval_state()` 函数相应更新

4. **求值流程变更**：
   - `evaluate()` 函数开始时将 `SExpr` 转换为 `RuntimeObject`
   - `evaluate_step()` 函数直接处理 `RuntimeObject` 而不是 `SExpr`

这些变更的优势：
- **类型一致性**：求值状态中的表达式与运行时对象类型保持一致
- **内存管理统一**：所有运行时对象都使用 `Gc` 管理
- **性能优化**：避免在求值过程中重复的 `SExpr` 到 `RuntimeObject` 转换
- **垃圾回收友好**：所有相关对象都支持 `Trace` 和 `Finalize`

### SExpr vs RuntimeObject

**注意**：在本设计文档中，我们已经实现了清晰的类型分离：

- **SExpr**：纯粹的语法结构，只用于表示解析阶段的 S 表达式
- **RuntimeObject**：运行时对象类型，包含所有可能的计算结果，包括闭包、内置函数等

这种分离有以下优势：
1. **概念清晰**：解析时和运行时的值类型明确分离
2. **类型安全**：编译时防止在错误的阶段使用错误的类型  
3. **可扩展性**：运行时概念不会污染语法结构的纯粹性
4. **内存优化**：RuntimeObject 采用原子值和嵌入结构的分类，减少间接访问
5. **垃圾回收**：支持自动内存管理，避免内存泄漏

### ContinuationFn vs 闭包

**注意**：为了支持 `Gc` 垃圾回收，我们使用 `ContinuationFn` trait 替代闭包：

- **闭包问题**：`Box<dyn Fn>` 不支持 `Trace` trait，无法被垃圾回收器追踪
- **ContinuationFn 解决方案**：定义具体的结构体实现 `ContinuationFn` trait
- **优势**：
  1. **垃圾回收兼容**：所有续延函数都支持 `Trace` 和 `Finalize`
  2. **类型安全**：编译时检查续延函数的正确性
  3. **性能优化**：避免动态分发的开销
  4. **调试友好**：每个续延类型都有明确的名称和结构

### RuntimeObject 设计特点

1. **原子值（Atomic Values）**：
   - 直接存储在 `RuntimeObjectCore` 中
   - 包括：Integer、Float、Rational、Character、Boolean、Nil
   - 无需间接引用，访问效率高

2. **嵌入结构（Embedded Structures）**：
   - 直接嵌入到 `RuntimeObjectCore` 中
   - 包括：String、Symbol、BuiltinFunction、Cons、Vector、Continuation、Lambda
   - 减少指针间接访问，提高缓存局部性

3. **内存管理**：
   - 使用 `Gc` 进行垃圾回收
   - 使用 `Rc` 管理共享数据
   - 支持 `Trace` 和 `Finalize` trait

4. **大小优化**：
   - `RuntimeObjectCore` 最大 24 字节
   - `RuntimeObject` 最大 32 字节
   - 编译时静态检查确保大小限制

详见：[Runtime_Value_Design.md](./Runtime_Value_Design.md)

---

## 参考文献

- [R7RS Scheme 标准](https://small.r7rs.org/)
- [SICP - Structure and Interpretation of Computer Programs](https://mitpress.mit.edu/sites/default/files/sicp/index.html)
- [Crafting Interpreters](https://craftinginterpreters.com/)
- [Environment_Management.md](./Environment_Management.md) - 环境管理设计
- [Parser_Design.md](./Parser_Design.md) - 语法分析器设计