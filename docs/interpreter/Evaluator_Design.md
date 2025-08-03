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
/// 表达式栈 - 不可变的表达式栈，用于管理待求值的表达式
#[derive(Debug, Clone)]
pub enum ExpressionStack {
    /// 空栈
    Empty,
    /// 包含表达式的栈
    Cons {
        /// 栈顶表达式
        head: Rc<SExpr>,
        /// 栈尾
        tail: Rc<ExpressionStack>,
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
TODO

### 问题：如何在函数式设计中高效实现尾调用优化？
TODO

### 问题：如何设计环境链式结构以支持词法作用域和闭包？
TODO

### 问题：如何处理特殊形式（define、if、lambda 等）的求值逻辑？
TODO

### 问题：如何在不可变设计中维护调用栈信息用于错误报告？
TODO

### 问题：如何设计内置函数的注册和调用机制？
TODO

## 参考文献
- [Structure and Interpretation of Computer Programs](https://mitpress.mit.edu/sites/default/files/sicp/index.html)
- [Scheme R7RS Specification](https://small.r7rs.org/)
- [Rust Functional Programming Patterns](https://doc.rust-lang.org/book/)