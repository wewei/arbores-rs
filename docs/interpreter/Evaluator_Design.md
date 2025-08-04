# Evaluator 设计

状态：Draft-2

## 概述

Evaluator 模块负责对语法分析器生成的 S 表达式抽象语法树进行求值计算。本模块采用 **函数式状态机** 设计，将 Evaluator 抽象为基于不可变状态的状态机，通过三个核心函数驱动计算过程：

1. **初始化函数** (`initialize_frame`)：创建初始栈帧状态
2. **状态转移函数** (`step_evaluation`)：执行单步计算，返回新的栈帧状态
3. **继续判定函数** (`should_continue`)：判断是否继续计算或已完成求值

## 模块职责（功能性需求）

- **表达式求值**：对各种 Scheme 表达式进行求值，包括原子值、函数调用、特殊形式等
- **环境管理**：维护变量绑定的词法作用域环境链
- **调用栈管理**：通过不可变链式栈结构管理函数调用和递归
- **错误处理**：提供详细的运行时错误信息，包含调用栈和位置信息
- **特殊形式处理**：实现 Scheme 的特殊语法构造（define、if、lambda、let 等）

## 设计目标（非功能性需求）

- **函数式纯净性**：所有状态变更通过创建新的不可变数据结构实现，无副作用
- **内存效率**：通过 Rc 共享不变数据，避免不必要的数据复制
- **调试友好**：保持完整的调用栈信息和源码位置，便于错误定位
- **可测试性**：状态机模型使得每个计算步骤都可以独立测试和验证
- **可扩展性**：模块化设计便于添加新的特殊形式和内置函数

## 关键数据类型

### Frame - 调用栈帧
```rust
/// 调用栈帧 - 表示单次函数调用或表达式求值的执行上下文
#[derive(Debug, Clone)]
pub struct Frame {
    /// 当前栈帧的环境（变量绑定）
    pub environment: Rc<Environment>,
    /// 待求值的表达式栈（后进先出）
    pub expression_stack: Rc<ExpressionStack>,
    /// 父调用栈帧（用于返回）
    pub parent_frame: Option<Rc<Frame>>,
    /// 当前栈帧的标识信息（用于调试）
    pub frame_info: FrameInfo,
}

/// 栈帧标识信息
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// 栈帧类型（函数调用、特殊形式等）
    pub frame_type: FrameType,
    /// 源码位置信息
    pub span: Rc<Span>,
}

/// 栈帧类型
#[derive(Debug, Clone)]
pub enum FrameType {
    /// 顶层表达式求值
    TopLevel,
    /// 函数调用
    FunctionCall { function_name: Option<String> },
    /// 特殊形式
    SpecialForm { form_name: String },
    /// Let 绑定
    LetBinding,
}
```

### Environment - 词法环境
```rust
/// 词法环境 - 不可变的变量绑定环境，形成链式结构
#[derive(Debug, Clone)]
pub struct Environment {
    /// 当前环境的变量绑定
    pub bindings: HashMap<String, Rc<SExpr>>,
    /// 父环境（外层作用域）
    pub parent: Option<Rc<Environment>>,
}
```

### ExpressionStack - 表达式栈
```rust
/// 表达式栈 - 模拟对表达式树的前后序遍历
#[derive(Debug, Clone)]
pub enum ExpressionStack {
    /// 空栈
    Empty,
    /// 包含表达式和访问状态的栈
    Cons {
        /// 当前表达式引用
        expr: Rc<SExpr>,
        /// 当前访问状态
        state: VisitState,
        /// 栈尾
        tail: Rc<ExpressionStack>,
    },
}

/// 表达式访问状态 - 表示当前处于表达式树遍历的哪个阶段
#[derive(Debug, Clone)]
pub enum VisitState {
    /// 前序访问 - 需要分析表达式类型并决定处理策略
    PreOrder,
    /// 函数后序访问 - 所有参数已计算完成，准备执行函数调用
    FunctionPostOrder {
        /// 函数值（已求值的第一个参数）
        function: Rc<SExpr>,
        /// 已求值的参数列表
        evaluated_args: Vec<Rc<SExpr>>,
    },
    /// 特殊形式后序访问 - 按特殊形式规则处理完参数后的执行阶段
    SpecialFormPostOrder {
        /// 特殊形式名称
        form_name: String,
        /// 已处理的参数（根据特殊形式规则，部分求值或未求值）
        processed_args: Vec<Rc<SExpr>>,
    },
}
```

### ContinuationResult - 继续判定结果
```rust
/// 继续判定结果（用于 should_continue 函数）
#[derive(Debug, Clone)]
pub enum ContinuationResult {
    /// 继续计算，返回当前栈帧
    Continue(Frame),
    /// 计算完成，返回最终值
    Complete(Rc<SExpr>),
}

/// 单步求值结果类型（用于 step_evaluation 函数）
pub type StepResult = Result<Frame, EvaluationError>;

/// 求值错误
#[derive(Debug, Clone)]
pub enum EvaluationError {
    /// 未定义的变量
    UndefinedVariable { 
        name: String, 
        span: Rc<Span> 
    },
    /// 类型错误
    TypeError { 
        expected: String, 
        found: String, 
        span: Rc<Span> 
    },
    /// 参数数量错误
    ArityError { 
        expected: usize, 
        found: usize, 
        span: Rc<Span> 
    },
    /// 其他运行时错误
    RuntimeError { 
        message: String, 
        span: Rc<Span> 
    },
}
```

## 核心函数接口（对外接口）

**重要说明**：本节只记录对外暴露的主要接口函数，不包括内部实现函数。内部辅助函数、私有方法和实现细节不在此处描述。

### initialize_frame (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| expression | Rc<SExpr> | 待求值的表达式 |
| parent_frame | Option<Rc<Frame>> | 可选的父栈帧 |
| environment | Option<Rc<Environment>> | 可选的初始环境，如果为 None 则使用全局环境 |

#### 返回值
| 类型 | 描述 |
|------|------|
| Frame | 初始化的新栈帧，ready for evaluation |

### step_evaluation (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| frame | Frame | 当前栈帧状态 |

#### 返回值
| 类型 | 描述 |
|------|------|
| StepResult | 单步求值后的新栈帧，如果出错则返回错误 |

### should_continue (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| frame | Frame | 要检查的栈帧 |

#### 返回值
| 类型 | 描述 |
|------|------|
| ContinuationResult | Continue(Frame) 表示继续计算，Complete(Rc<SExpr>) 表示已完成并返回最终值 |

### evaluate (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| expression | Rc<SExpr> | 待求值的表达式 |
| environment | Option<Rc<Environment>> | 可选的初始环境 |

#### 返回值
| 类型 | 描述 |
|------|------|
| Result<Rc<SExpr>, EvaluationError> | 求值结果：成功返回值或错误 |

### evaluate_with_steps (对外接口)

#### 参数列表
| 参数名 | 类型 | 描述 |
|--------|------|------|
| expression | Rc<SExpr> | 待求值的表达式 |
| environment | Option<Rc<Environment>> | 可选的初始环境 |

#### 返回值
| 类型 | 描述 |
|------|------|
| impl Iterator<Item = ContinuationResult> | 返回状态机执行过程的迭代器，最后一项为 Complete |

## 关键设计问题

### 问题：如何设计不可变的表达式栈以支持复杂的求值过程？

**设计原理**：表达式栈模拟对表达式树的前后序遍历过程，每个栈项包含表达式引用和当前访问状态。

**核心设计**：
```rust
// 栈操作的核心函数
fn push_expression(stack: Rc<ExpressionStack>, expr: Rc<SExpr>, state: VisitState) -> Rc<ExpressionStack> {
    Rc::new(ExpressionStack::Cons { expr, state, tail: stack })
}

fn pop_expression(stack: Rc<ExpressionStack>) -> Option<(Rc<SExpr>, VisitState, Rc<ExpressionStack>)> {
    match stack.as_ref() {
        ExpressionStack::Empty => None,
        ExpressionStack::Cons { expr, state, tail } => {
            Some((expr.clone(), state.clone(), tail.clone()))
        },
    }
}
```

**表达式树遍历示例**：
```
// 求值 (+ (* 2 3) 4)
// 表达式树：
//     +
//    / \
//   *   4
//  / \
// 2   3

步骤1: [(+ (* 2 3) 4), PreOrder] - 开始处理根节点
步骤2: [(+ (* 2 3) 4), FunctionPostOrder{function: +, args: []}], [(* 2 3), PreOrder] - 识别为函数调用，开始处理第一个参数
步骤3: [(+ (* 2 3) 4), FunctionPostOrder{function: +, args: []}], [(* 2 3), FunctionPostOrder{function: *, args: []}], [2, PreOrder] - 处理乘法的第一个参数
步骤4: [(+ (* 2 3) 4), FunctionPostOrder{function: +, args: []}], [(* 2 3), FunctionPostOrder{function: *, args: [2]}], [3, PreOrder] - 处理乘法的第二个参数
步骤5: [(+ (* 2 3) 4), FunctionPostOrder{function: +, args: []}], [(* 2 3), FunctionPostOrder{function: *, args: [2, 3]}] - 执行乘法
步骤6: [(+ (* 2 3) 4), FunctionPostOrder{function: +, args: [6]}], [4, PreOrder] - 处理加法的第二个参数
步骤7: [(+ (* 2 3) 4), FunctionPostOrder{function: +, args: [6, 4]}] - 执行加法
步骤8: [] - 完成，结果为 10
```

**访问状态转换**：
- **PreOrder → FunctionPostOrder**：识别为函数调用，开始收集参数
- **PreOrder → SpecialFormPostOrder**：识别为特殊形式，按规则处理参数
- **PostOrder → 弹出栈**：完成当前表达式处理，回到父节点

**内存效率**：通过 Rc 共享表达式引用，避免重复复制表达式树节点。

### 问题：如何在函数式设计中高效实现尾调用优化？

**设计原理**：尾调用优化通过识别尾位置调用并重用当前栈帧实现，避免栈增长。

**尾调用识别**：
```rust
fn is_tail_position(remaining_stack: &ExpressionStack, parent_frame: &Option<Rc<Frame>>) -> bool {
    match remaining_stack {
        ExpressionStack::Empty => {
            // 当前栈为空，检查父栈帧是否也处于尾位置
            parent_frame.as_ref()
                .map_or(true, |parent| is_tail_position(&parent.expression_stack, &parent.parent_frame))
        },
        ExpressionStack::Cons { state, tail, .. } => {
            // 如果栈中还有其他表达式等待处理，则不是尾位置
            match state {
                VisitState::FunctionPostOrder { .. } | VisitState::SpecialFormPostOrder { .. } => {
                    // 在后序处理阶段，且没有更多表达式，可能是尾位置
                    matches!(tail.as_ref(), ExpressionStack::Empty)
                },
                _ => false,
            }
        }
    }
}
```

**尾调用优化策略**：
1. **检测尾调用**：在函数调用的后序处理阶段检查是否处于尾位置
2. **栈帧重用**：如果是尾调用，更新当前栈帧而不是创建新栈帧
3. **环境替换**：用新函数的环境替换当前环境

**实现伪代码**：
```rust
fn handle_function_call(function: Rc<SExpr>, evaluated_args: Vec<Rc<SExpr>>, remaining_stack: Rc<ExpressionStack>, frame: Frame) -> StepResult {
    if is_tail_position(&remaining_stack, &frame.parent_frame) {
        // 尾调用优化：重用当前栈帧
        let new_env = create_function_environment(&function, evaluated_args, &frame.environment)?;
        let func_body = extract_function_body(&function)?;
        
        Ok(Frame {
            environment: Rc::new(new_env),
            expression_stack: push_expression(remaining_stack, func_body, VisitState::PreOrder),
            parent_frame: frame.parent_frame, // 保持相同的父栈帧
            frame_info: frame.frame_info,
        })
    } else {
        // 普通调用：创建新栈帧
        let new_env = create_function_environment(&function, evaluated_args, &frame.environment)?;
        let func_body = extract_function_body(&function)?;
        
        let new_frame = Frame {
            environment: Rc::new(new_env),
            expression_stack: push_expression(Rc::new(ExpressionStack::Empty), func_body, VisitState::PreOrder),
            parent_frame: Some(Rc::new(Frame {
                expression_stack: remaining_stack,
                ..frame
            })),
            frame_info: create_function_frame_info(&function),
        };
        
        Ok(new_frame)
    }
}
```

**优化效果**：尾递归函数可以在常数栈空间内执行，避免栈溢出问题。

### 问题：如何设计环境链式结构以支持词法作用域和闭包？

**设计原理**：环境采用不可变链式结构，每个环境保存当前作用域的绑定和父环境引用，实现词法作用域查找。

**环境操作**：
```rust
// 创建新环境
fn extend_environment(parent: Rc<Environment>, bindings: HashMap<String, Rc<SExpr>>) -> Environment {
    Environment {
        bindings,
        parent: Some(parent),
    }
}

// 变量查找（从内到外）
fn lookup_variable(env: &Environment, name: &str) -> Option<Rc<SExpr>> {
    env.bindings.get(name).cloned()
        .or_else(|| env.parent.as_ref()?.lookup_variable(name))
}
```

**闭包实现**：
```rust
// 闭包数据结构
#[derive(Debug, Clone)]
pub struct Closure {
    pub parameters: Vec<String>,
    pub body: Rc<SExpr>,
    pub captured_environment: Rc<Environment>, // 捕获创建时的环境
}

// 闭包调用
fn call_closure(closure: &Closure, args: Vec<Rc<SExpr>>) -> Result<Environment, EvaluationError> {
    let mut bindings = HashMap::new();
    for (param, arg) in closure.parameters.iter().zip(args.iter()) {
        bindings.insert(param.clone(), arg.clone());
    }
    
    // 在捕获的环境基础上扩展新绑定
    Ok(extend_environment(closure.captured_environment.clone(), bindings))
}
```

**作用域示例**：
```scheme
;; 演示词法作用域
(define x 10)
(define make-adder 
  (lambda (n) 
    (lambda (y) (+ x n y)))) ;; 捕获外层的 x 和参数 n
(define add5 (make-adder 5))
(add5 3) ;; 结果：18 (10 + 5 + 3)
```

**内存管理**：通过 Rc 实现环境共享，多个闭包可以安全地共享相同的父环境。

### 问题：如何处理特殊形式（define、if、lambda 等）的求值逻辑？

**设计原理**：特殊形式需要特殊的求值规则，不能简单地求值所有参数。通过模式匹配和专门的处理函数实现。

**特殊形式识别**：
```rust
fn handle_special_form(form_name: &str, args: Vec<Rc<SExpr>>, frame: Frame) -> StepResult {
    match form_name {
        "define" => handle_define(args, frame),
        "if" => handle_if(args, frame),
        "lambda" => handle_lambda(args, frame),
        "let" => handle_let(args, frame),
        "quote" => handle_quote(args, frame),
        _ => Err(EvaluationError::RuntimeError {
            message: format!("Unknown special form: {}", form_name),
            span: frame.frame_info.span,
        }),
    }
}
```

**具体实现示例**：

**表达式处理主逻辑**：
```rust
fn step_evaluation(frame: Frame) -> StepResult {
    match pop_expression(frame.expression_stack.clone()) {
        None => {
            // 栈为空，无更多表达式需要处理
            Ok(frame) // should_continue 会检测到这种情况并返回 Complete
        },
        Some((expr, state, remaining_stack)) => {
            match state {
                VisitState::PreOrder => handle_pre_order(expr, remaining_stack, frame),
                VisitState::FunctionPostOrder { function, evaluated_args } => {
                    handle_function_call(function, evaluated_args, remaining_stack, frame)
                },
                VisitState::SpecialFormPostOrder { form_name, processed_args } => {
                    handle_special_form_execution(form_name, processed_args, remaining_stack, frame)
                },
            }
        }
    }
}

fn handle_pre_order(expr: Rc<SExpr>, remaining_stack: Rc<ExpressionStack>, frame: Frame) -> StepResult {
    match &expr.content {
        // 原子值直接求值完成
        SExprContent::Atom(_) => {
            accumulate_result(expr, remaining_stack, frame)
        },
        // 列表需要分析第一个元素
        SExprContent::Cons { car, cdr } => {
            if let SExprContent::Atom(Value::Symbol(symbol)) = &car.content {
                if is_special_form(symbol) {
                    // 特殊形式：按规则处理参数
                    handle_special_form_args(symbol.clone(), cdr.clone(), expr, remaining_stack, frame)
                } else {
                    // 函数调用：依次求值所有参数
                    setup_function_call(car.clone(), cdr.clone(), expr, remaining_stack, frame)
                }
            } else {
                // 第一个元素不是符号，先求值它
                let new_stack = push_expression(remaining_stack, expr, VisitState::PreOrder);
                let eval_stack = push_expression(new_stack, car.clone(), VisitState::PreOrder);
                Ok(Frame { expression_stack: eval_stack, ..frame })
            }
        },
        _ => Err(EvaluationError::TypeError {
            expected: "evaluable expression".to_string(),
            found: format!("{:?}", expr.content),
            span: expr.span.clone(),
        }),
    }
}
```

**if 特殊形式**：
```rust
fn handle_special_form_args(form_name: String, args: Rc<SExpr>, original_expr: Rc<SExpr>, remaining_stack: Rc<ExpressionStack>, frame: Frame) -> StepResult {
    match form_name.as_str() {
        "if" => {
            let args_list = extract_list(args)?;
            if args_list.len() < 2 || args_list.len() > 3 {
                return Err(EvaluationError::ArityError { 
                    expected: 2, 
                    found: args_list.len(), 
                    span: original_expr.span.clone() 
                });
            }
            
            // 只求值条件表达式，then 和 else 分支暂不求值
            let post_stack = push_expression(
                remaining_stack, 
                original_expr, 
                VisitState::SpecialFormPostOrder { 
                    form_name, 
                    processed_args: args_list[1..].to_vec() // then 和 else 分支
                }
            );
            let eval_stack = push_expression(post_stack, args_list[0].clone(), VisitState::PreOrder);
            Ok(Frame { expression_stack: eval_stack, ..frame })
        },
        // 其他特殊形式...
        _ => todo!("Handle other special forms"),
    }
}
```

**函数调用设置**：
```rust
fn setup_function_call(func_expr: Rc<SExpr>, args: Rc<SExpr>, original_expr: Rc<SExpr>, remaining_stack: Rc<ExpressionStack>, frame: Frame) -> StepResult {
    let args_list = extract_list(args)?;
    
    // 设置后序处理状态
    let post_stack = push_expression(
        remaining_stack,
        original_expr,
        VisitState::FunctionPostOrder {
            function: func_expr.clone(),
            evaluated_args: Vec::new(),
        }
    );
    
    // 先求值函数表达式
    let eval_stack = push_expression(post_stack, func_expr, VisitState::PreOrder);
    Ok(Frame { expression_stack: eval_stack, ..frame })
}
```

**状态机集成**：特殊形式处理通过修改表达式栈来控制求值流程，而不是直接返回结果。

### 问题：如何在不可变设计中维护调用栈信息用于错误报告？

**设计原理**：利用 Frame 的链式结构天然保存完整调用栈，结合源码位置信息提供详细的错误报告。

**调用栈构建**：
```rust
fn build_call_stack(frame: &Frame) -> Vec<CallStackEntry> {
    let mut stack = Vec::new();
    let mut current = Some(frame);
    
    while let Some(f) = current {
        stack.push(CallStackEntry {
            function_name: extract_function_name(&f.frame_info),
            location: f.frame_info.span.clone(),
            frame_type: f.frame_info.frame_type.clone(),
        });
        current = f.parent_frame.as_ref().map(|p| p.as_ref());
    }
    
    stack.reverse(); // 从顶层到当前层
    stack
}
```

**错误增强**：
```rust
fn enrich_error_with_context(error: EvaluationError, frame: &Frame) -> EvaluationError {
    let call_stack = build_call_stack(frame);
    match error {
        EvaluationError::UndefinedVariable { name, span } => {
            EvaluationError::UndefinedVariable { 
                name, 
                span,
                // 添加调用栈信息到错误中
            }.with_call_stack(call_stack)
        },
        // 其他错误类型类似处理...
        _ => error.with_call_stack(call_stack),
    }
}
```

**错误报告格式**：
```
Error: Undefined variable 'unknown-var' at line 15, column 8
Call stack:
  1. main (program:1:1)
  2. calculate (calculate:5:10) 
  3. helper (helper:12:15)
     
Source context:
  13:     (define helper
  14:       (lambda (x)
  15:         (+ x unknown-var)))  ; <-- Error here
  16:     helper))
```

**位置信息传递**：
```rust
fn propagate_span_information(frame: &Frame, expr: &SExpr) -> Rc<Span> {
    // 优先使用表达式自身的位置信息
    if !expr.span.is_empty() {
        expr.span.clone()
    } else {
        // 回退到栈帧的位置信息
        frame.frame_info.span.clone()
    }
}
```

**调试支持**：不可变设计使得可以在任意时刻"快照"整个调用栈状态，便于调试和错误分析。

### 问题：如何设计内置函数的注册和调用机制？

**设计原理**：内置函数通过注册表管理，与用户定义函数采用统一的调用接口，支持动态扩展。

**内置函数注册**：
```rust
// 内置函数签名
type BuiltinFunction = fn(&[Rc<SExpr>], &Environment) -> Result<Rc<SExpr>, EvaluationError>;

// 注册表结构
#[derive(Debug, Clone)]
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFunction>,
}

impl BuiltinRegistry {
    fn register(&mut self, name: &str, func: BuiltinFunction) {
        self.functions.insert(name.to_string(), func);
    }
    
    fn lookup(&self, name: &str) -> Option<BuiltinFunction> {
        self.functions.get(name).copied()
    }
}
```

**内置函数实现示例**：
```rust
// 算术运算
fn builtin_add(args: &[Rc<SExpr>], _env: &Environment) -> Result<Rc<SExpr>, EvaluationError> {
    if args.is_empty() {
        return Ok(create_number_value(0.0));
    }
    
    let mut sum = 0.0;
    for arg in args {
        sum += extract_number(arg)?;
    }
    Ok(create_number_value(sum))
}

// 列表操作
fn builtin_cons(args: &[Rc<SExpr>], _env: &Environment) -> Result<Rc<SExpr>, EvaluationError> {
    if args.len() != 2 {
        return Err(EvaluationError::ArityError { 
            expected: 2, 
            found: args.len(),
            span: Rc::new(Span::empty(0)),
        });
    }
    Ok(create_cons_value(args[0].clone(), args[1].clone()))
}
```

**全局环境初始化**：
```rust
fn create_global_environment() -> Environment {
    let mut registry = BuiltinRegistry::new();
    
    // 注册标准内置函数
    registry.register("+", builtin_add);
    registry.register("-", builtin_subtract);
    registry.register("*", builtin_multiply);
    registry.register("/", builtin_divide);
    registry.register("cons", builtin_cons);
    registry.register("car", builtin_car);
    registry.register("cdr", builtin_cdr);
    // ... 更多内置函数
    
    // 将内置函数添加到全局环境
    let mut bindings = HashMap::new();
    for (name, func) in registry.functions {
        bindings.insert(name, create_builtin_function_value(func));
    }
    
    Environment {
        bindings,
        parent: None,
    }
}
```

**调用统一接口**：
```rust
fn call_function(func_value: &SExpr, args: Vec<Rc<SExpr>>, env: &Environment) -> Result<Rc<SExpr>, EvaluationError> {
    match &func_value.content {
        SExprContent::BuiltinFunction(builtin_func) => {
            // 调用内置函数
            builtin_func(args.as_slice(), env)
        },
        SExprContent::Closure(closure) => {
            // 调用用户定义函数
            call_user_function(closure, args, env)
        },
        _ => Err(EvaluationError::TypeError {
            expected: "function".to_string(),
            found: format!("{:?}", func_value.content),
            span: func_value.span.clone(),
        }),
    }
}
```

**扩展性**：新的内置函数可以通过简单注册添加，无需修改核心求值逻辑。

## 参考文献
- [Structure and Interpretation of Computer Programs](https://mitpress.mit.edu/sites/default/files/sicp/index.html)
- [Scheme R7RS Specification](https://small.r7rs.org/)
- [Rust Functional Programming Patterns](https://doc.rust-lang.org/book/)