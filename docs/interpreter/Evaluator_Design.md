# Evaluator 设计

状态：Draft-2

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

## 关键数据类型

### EvalState
```rust
/// 求值状态 - 表示求值过程中的当前状态
/// 采用不可变设计，每次状态转移都产生新的状态
pub struct EvalState {
    /// 当前调用栈 Frame
    frame: Frame,
    /// 待求值表达式
    expr: SExpr,
}
```

### Frame
```rust
/// 调用栈帧 - 链式栈结构，表示当前的执行上下文
pub struct Frame {
    /// 当前栈的环境
    env: Environment,
    /// 返回的 Lambda 回调，输入返回的 SExpr，返回 EvaluateResult
    continuation: Box<dyn Fn(SExpr) -> EvaluateResult>,
    /// 父栈帧（链式结构）
    parent: Option<Box<Frame>>,
}
```

### Environment
```rust
/// 环境 - 变量绑定和作用域管理
/// 链式结构，每个节点包含局部绑定并引用上级环境
pub struct Environment {
    /// 当前环境的变量绑定表 (变量名 -> 值)
    bindings: HashMap<String, SExpr>,
    /// 上级环境（链式结构）
    parent: Option<Box<Environment>>,
}
```

### EvaluateError
```rust
/// 求值错误类型 - 表示求值过程中可能出现的各种错误
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluateError {
    // TODO: 待定义具体的错误类型
    // 可能包含：未定义变量、类型错误、运行时错误等
}
```

### EvaluateResult
```rust
/// 求值步骤结果 - 表示单步求值的三种可能结果
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluateResult {
    /// 求值完成，返回最终结果
    Completed(SExpr),
    /// 需要继续求值，返回下一个状态
    Continue(EvalState),
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
| expr | SExpr | 要求值的 S 表达式 |
| env | Environment | 全局环境 |

#### 返回值
| 类型 | 描述 |
|------|------|
| Result<SExpr, EvaluateError> | 求值结果的 S 表达式或错误信息 |

### evaluate_step() - 单步状态转移函数 (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| state | EvalState | 当前求值状态 |

#### 返回值
| 类型 | 描述 |
|------|------|
| EvaluateResult | 三分枝结果：Completed(结果)、Continue(下一状态)、Error(错误) |

## 关键设计问题

### 问题：如何初始化 EvalState？

EvalState 的初始化需要创建一个根栈帧和待求值的表达式：

1. **创建根栈帧**：
   - `env`: 使用传入的全局环境
   - `continuation`: 创建一个终止回调函数，当求值完成时返回 `Completed` 结果
   - `parent`: 设为 `None`，表示这是最顶层的栈帧

2. **设置待求值表达式**：
   - `expr`: 直接使用传入的 SExpr

```rust
fn init_eval_state(expr: SExpr, env: Environment) -> EvalState {
    let root_frame = Frame {
        env,
        continuation: Box::new(|result| {
            // 根栈帧的 continuation，表示求值完成
            EvaluateResult::Completed(result)
        }),
        parent: None,
    };
    
    EvalState {
        frame: root_frame,
        expr,
    }
}
```

这种设计的优势：
- **环境管理**：全局环境保存在根栈帧中
- **统一接口**：continuation 返回 EvaluateResult，支持所有三种状态
- **简洁实现**：根栈帧的 continuation 直接返回完成状态

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
fn evaluate(expr: SExpr, env: Environment) -> Result<SExpr, EvaluateError> {
    let mut current_state = init_eval_state(expr, env);
    
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

### 问题：单步迭代时，如何判定函数调用和特殊形式？

在 `evaluate_step` 中，需要根据当前表达式的类型进行分发处理：

1. **表达式类型分析**：
   - **自求值表达式**：数字、字符串、布尔值等，直接调用 continuation
   - **符号**：变量引用，在环境中查找值
   - **列表**：可能是函数调用或特殊形式，需要进一步判断

2. **列表表达式的判定逻辑**：
   ```rust
   fn evaluate_step(state: EvalState) -> EvaluateResult {
       match &state.expr.content {
           // 自求值表达式
           SExprContent::Atom(Value::Number(_)) | 
           SExprContent::Atom(Value::String(_)) | 
           SExprContent::Atom(Value::Bool(_)) => {
               // 直接调用当前 frame 的 continuation
               (state.frame.continuation)(state.expr)
           },
           
           // 符号（变量引用）
           SExprContent::Atom(Value::Symbol(name)) => {
               match lookup_variable(name, &state.frame.env) {
                   Some(value) => (state.frame.continuation)(value),
                   None => EvaluateResult::Error(EvaluateError::UndefinedVariable(name.clone())),
               }
           },
           
           // 列表表达式
           SExprContent::Cons { car, cdr } => {
               evaluate_list_expression(state, car, cdr)
           },
           
           // 其他情况
           _ => EvaluateResult::Error(EvaluateError::InvalidExpression),
       }
   }
   ```

3. **特殊形式判定**：
   ```rust
   fn evaluate_list_expression(state: EvalState, car: &SExpr, cdr: &SExpr) -> EvaluateResult {
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
   - **错误处理**：未知表达式类型有明确的错误处理
   - **统一接口**：所有处理函数都返回 EvaluateResult

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
   fn evaluate_function_call(state: EvalState, operator: &SExpr, operands: &SExpr) -> EvaluateResult {
       // 阶段1：求值函数表达式
       let new_continuation = Box::new(move |function_value| {
           // 当函数求值完成后，开始求值参数
           evaluate_arguments(state.frame, function_value, operands, Vec::new())
       });
       
       let new_frame = Frame {
           env: state.frame.env.clone(),
           continuation: new_continuation,
           parent: Some(Box::new(state.frame)),
       };
       
       EvaluateResult::Continue(EvalState {
           frame: new_frame,
           expr: operator.clone(),
       })
   }
   ```

3. **参数求值的递归处理**：
   ```rust
   fn evaluate_arguments(
       parent_frame: Frame, 
       function_value: SExpr, 
       remaining_args: &SExpr, 
       evaluated_args: Vec<SExpr>
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
       function_value: SExpr, 
       arguments: Vec<SExpr>
   ) -> EvaluateResult {
       match &function_value.content {
           // 内置函数
           SExprContent::Atom(Value::BuiltinFunction(func)) => {
               match func.call(&arguments) {
                   Ok(result) => (parent_frame.continuation)(result),
                   Err(error) => EvaluateResult::Error(error),
               }
           },
           
           // 用户定义的 lambda 函数
           SExprContent::Lambda { params, body, closure_env } => {
               // 创建新环境，绑定参数
               let new_env = bind_parameters(params, &arguments, closure_env)?;
               
               let new_frame = Frame {
                   env: new_env,
                   continuation: parent_frame.continuation,
                   parent: parent_frame.parent,
               };
               
               EvaluateResult::Continue(EvalState {
                   frame: new_frame,
                   expr: body.clone(),
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

6. **尾递归优化关键**：
   - lambda 函数的函数体求值时，直接使用父栈帧的 continuation
   - 避免创建新的栈帧链，实现真正的尾调用消除

### 问题：如何单步迭代 quote 特殊形式？
TODO

### 问题：如何单步迭代 if 特殊形式？
TODO

### 问题：如何单步迭代 lambda 特殊形式？
TODO

### 问题：如何单步迭代 define 特殊形式？
TODO

### 问题：如何单步迭代 let 特殊形式？
TODO

### 问题：如何实现尾递归优化？
TODO

## 参考文献

- [R7RS Scheme 标准](https://small.r7rs.org/)
- [SICP - Structure and Interpretation of Computer Programs](https://mitpress.mit.edu/sites/default/files/sicp/index.html)
- [Crafting Interpreters](https://craftinginterpreters.com/)
- [Environment_Management.md](./Environment_Management.md) - 环境管理设计
- [Parser_Design.md](./Parser_Design.md) - 语法分析器设计