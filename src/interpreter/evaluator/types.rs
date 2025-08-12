//! Evaluator 核心数据类型定义
//! 
//! 本模块定义了求值器的核心数据结构，遵循函数式设计原则：
//! - 使用代数数据类型 (ADT) 表示复杂状态
//! - 不可变设计，每次状态转移都产生新的状态
//! - 纯数据结构，不包含业务逻辑方法

use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::SExpr;
use crate::interpreter::lexer::types::Span;

// ============================================================================
// 运行时值类型
// ============================================================================

/// 运行时值 - 表示求值过程中的所有可能值类型
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    /// 数字（统一使用 f64）
    Number(f64),
    /// 字符串
    String(String),
    /// 字符
    Character(char),
    /// 布尔值
    Boolean(bool),
    /// 符号（在运行时通常作为字符串处理）
    Symbol(String),
    /// 列表（cons 结构）
    Cons { 
        car: Rc<RuntimeValue>, 
        cdr: Rc<RuntimeValue> 
    },
    /// 空列表
    Nil,
    /// 向量
    Vector(Vec<RuntimeValue>),
    /// 用户定义的 Lambda 函数
    Lambda {
        parameters: Vec<String>,     // 参数名列表
        body: SExpr,                // 函数体（未求值的 S 表达式）
        closure: Environment,       // 闭包环境
    },
    /// 内置函数
    BuiltinFunction {
        name: String,
        arity: FunctionArity,       // 参数个数要求
        implementation: BuiltinImpl, // 函数实现
    },
}

/// 函数参数个数要求
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionArity {
    /// 固定参数个数
    Exact(usize),
    /// 最少参数个数（支持可变参数）
    AtLeast(usize),
    /// 参数个数范围
    Range(usize, usize),
}

/// 内置函数实现类型
/// 注意：这里暂时使用函数指针，后续可能需要调整为更复杂的类型
#[derive(Debug, Clone)]
pub struct BuiltinImpl {
    /// 函数实现（接收参数列表，返回结果或错误）
    pub func: fn(&[RuntimeValue]) -> Result<RuntimeValue, EvaluateError>,
}

// 为 BuiltinImpl 手动实现 PartialEq，因为函数指针不能直接比较
impl PartialEq for BuiltinImpl {
    fn eq(&self, other: &Self) -> bool {
        // 比较函数指针地址
        std::ptr::eq(self.func as *const (), other.func as *const ())
    }
}

// ============================================================================
// 环境类型
// ============================================================================

/// 环境 - 变量绑定和作用域管理
/// 链式结构，每个节点包含局部绑定并引用上级环境
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    /// 当前环境的变量绑定表 (变量名 -> 运行时值)
    pub bindings: HashMap<String, RuntimeValue>,
    /// 上级环境（链式结构）
    pub parent: Option<Rc<Environment>>,
}

// ============================================================================
// 求值状态类型
// ============================================================================

/// 尾调用上下文 - 标记当前表达式是否在尾位置
#[derive(Clone, Debug, PartialEq)]
pub enum TailContext {
    /// 在尾位置，可以进行尾调用优化
    TailPosition,
    /// 不在尾位置，需要保留调用上下文
    NonTailPosition,
}

/// 求值状态 - 表示求值过程中的当前状态
/// 采用不可变设计，每次状态转移都产生新的状态
#[derive(Clone, Debug)]
pub struct EvalState {
    /// 当前调用栈 Frame
    pub frame: Frame,
    /// 待求值表达式
    pub expr: SExpr,
    /// 尾调用上下文信息（用于尾调用优化）
    pub tail_context: TailContext,
    /// 当前表达式的绑定名称（如果有的话）
    /// 用于支持递归函数定义和调试信息
    pub binding_name: Option<String>,
}

/// 调用栈帧 - 链式栈结构，表示当前的执行上下文
#[derive(Clone, Debug)]
pub struct Frame {
    /// 当前栈的环境
    pub env: Environment,
    /// 返回的 Lambda 回调，输入返回的 RuntimeValue，返回 EvaluateResult
    pub continuation: Continuation,
    /// 父栈帧（链式结构）
    pub parent: Option<Rc<Frame>>,
}

/// Continuation 类型 - 表示求值完成后的后续处理
/// 使用 Rc 包装以支持克隆和共享
#[derive(Clone)]
pub struct Continuation {
    /// continuation 函数的实现
    /// 接收求值结果，返回下一步的求值结果
    pub func: Rc<dyn Fn(RuntimeValue) -> EvaluateResult>,
}

// 手动实现 Debug for Continuation
impl std::fmt::Debug for Continuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Continuation {{ func: <function> }}")
    }
}

// ============================================================================
// 求值结果类型
// ============================================================================

/// 求值步骤结果 - 表示单步求值的三种可能结果
#[derive(Debug, Clone)]
pub enum EvaluateResult {
    /// 求值完成，返回最终结果（运行时值）
    Completed(RuntimeValue),
    /// 需要继续求值，返回下一个状态
    Continue(EvalState),
    /// 求值出错，返回错误信息
    Error(EvaluateError),
}

// 手动实现 PartialEq for EvaluateResult，但跳过 Continue 分支的比较
impl PartialEq for EvaluateResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EvaluateResult::Completed(a), EvaluateResult::Completed(b)) => a == b,
            (EvaluateResult::Error(a), EvaluateResult::Error(b)) => a == b,
            // Continue 分支不比较，因为 EvalState 包含函数指针
            _ => false,
        }
    }
}

// ============================================================================
// 错误类型
// ============================================================================

/// 求值错误类型 - 表示求值过程中可能出现的各种错误
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluateError {
    // 语法错误
    InvalidQuoteSyntax {
        span: Rc<Span>,
        message: String,
    },
    InvalidIfSyntax {
        span: Rc<Span>,
        message: String,
    },
    InvalidLambdaSyntax {
        span: Rc<Span>,
        message: String,
    },
    InvalidDefineSyntax {
        span: Rc<Span>,
        message: String,
    },
    InvalidLetSyntax {
        span: Rc<Span>,
        message: String,
    },
    InvalidLetBinding {
        span: Rc<Span>,
        message: String,
    },
    InvalidParameterName {
        span: Rc<Span>,
        name: String,
    },
    InvalidParameterList {
        span: Rc<Span>,
        message: String,
    },
    InvalidArgumentList {
        span: Rc<Span>,
        message: String,
    },
    InvalidExpression {
        span: Rc<Span>,
        message: String,
    },
    
    // 运行时错误
    UndefinedVariable {
        span: Rc<Span>,
        name: String,
    },
    UndefinedFunction {
        span: Rc<Span>,
        name: String,
    },
    NotCallable {
        span: Rc<Span>,
        value: String, // 尝试调用的值的字符串表示
    },
    ArgumentCountMismatch {
        span: Rc<Span>,
        expected: String, // 期望的参数个数描述
        actual: usize,    // 实际的参数个数
    },
    DivisionByZero {
        span: Rc<Span>,
    },
    TypeMismatch {
        span: Rc<Span>,
        expected: String,
        actual: String,
    },
    TypeError {
        span: Rc<Span>,
        expected: String,
        actual: String,
    },
    
    // 系统错误
    StackOverflow {
        span: Rc<Span>,
    },
    OutOfMemory {
        span: Rc<Span>,
    },
    InternalError {
        span: Rc<Span>,
        message: String,
    },
    NotImplemented {
        span: Rc<Span>,
        feature: String,
    },
}

// ============================================================================
// 构造函数实现
// ============================================================================

impl Environment {
    /// 创建新的空环境
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    /// 创建带父环境的新环境
    pub fn with_parent(parent: Environment) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Rc::new(parent)),
        }
    }
    
    /// 在当前环境中定义变量
    pub fn define(&mut self, name: String, value: RuntimeValue) {
        self.bindings.insert(name, value);
    }
    
    /// 查找变量值（递归向上查找）
    pub fn lookup(&self, name: &str) -> Option<RuntimeValue> {
        // 先在当前环境查找
        if let Some(value) = self.bindings.get(name) {
            return Some(value.clone());
        }
        
        // 递归在父环境查找
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

impl Continuation {
    /// 创建新的 continuation
    pub fn new<F>(func: F) -> Self 
    where 
        F: Fn(RuntimeValue) -> EvaluateResult + 'static 
    {
        Self {
            func: Rc::new(func),
        }
    }
    
    /// 调用 continuation
    pub fn call(&self, value: RuntimeValue) -> EvaluateResult {
        (self.func)(value)
    }
}

impl Frame {
    /// 创建新的根栈帧
    pub fn new_root(env: Environment, continuation: Continuation) -> Self {
        Self {
            env,
            continuation,
            parent: None,
        }
    }
    
    /// 创建带父栈帧的新栈帧
    pub fn with_parent(env: Environment, continuation: Continuation, parent: Frame) -> Self {
        Self {
            env,
            continuation,
            parent: Some(Rc::new(parent)),
        }
    }
}

impl EvalState {
    /// 创建新的求值状态
    pub fn new(
        frame: Frame, 
        expr: SExpr, 
        tail_context: TailContext,
        binding_name: Option<String>
    ) -> Self {
        Self {
            frame,
            expr,
            tail_context,
            binding_name,
        }
    }
}

impl RuntimeValue {
    /// 创建内置函数值
    pub fn builtin_function(
        name: String, 
        arity: FunctionArity, 
        func: fn(&[RuntimeValue]) -> Result<RuntimeValue, EvaluateError>
    ) -> Self {
        Self::BuiltinFunction {
            name,
            arity,
            implementation: BuiltinImpl { func },
        }
    }
    
    /// 获取值的类型名称（用于错误报告）
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Number(_) => "number",
            RuntimeValue::String(_) => "string",
            RuntimeValue::Character(_) => "character",
            RuntimeValue::Boolean(_) => "boolean",
            RuntimeValue::Symbol(_) => "symbol",
            RuntimeValue::Cons { .. } => "pair",
            RuntimeValue::Nil => "null",
            RuntimeValue::Vector(_) => "vector",
            RuntimeValue::Lambda { .. } => "procedure",
            RuntimeValue::BuiltinFunction { .. } => "procedure",
        }
    }
}

impl FunctionArity {
    /// 检查参数个数是否匹配
    pub fn matches(&self, actual: usize) -> bool {
        match self {
            FunctionArity::Exact(expected) => actual == *expected,
            FunctionArity::AtLeast(min) => actual >= *min,
            FunctionArity::Range(min, max) => actual >= *min && actual <= *max,
        }
    }
    
    /// 获取期望参数个数的描述
    pub fn description(&self) -> String {
        match self {
            FunctionArity::Exact(n) => format!("{}", n),
            FunctionArity::AtLeast(n) => format!("at least {}", n),
            FunctionArity::Range(min, max) => format!("{} to {}", min, max),
        }
    }
}

// ============================================================================
// Display 实现 - 用于错误报告和调试
// ============================================================================

impl std::fmt::Display for EvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluateError::UndefinedVariable { name, .. } => {
                write!(f, "Undefined variable: {}", name)
            },
            EvaluateError::UndefinedFunction { name, .. } => {
                write!(f, "Undefined function: {}", name)
            },
            EvaluateError::NotCallable { value, .. } => {
                write!(f, "Value is not callable: {}", value)
            },
            EvaluateError::ArgumentCountMismatch { expected, actual, .. } => {
                write!(f, "Argument count mismatch: expected {}, got {}", expected, actual)
            },
            EvaluateError::DivisionByZero { .. } => {
                write!(f, "Division by zero")
            },
            EvaluateError::TypeMismatch { expected, actual, .. } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            },
            EvaluateError::TypeError { expected, actual, .. } => {
                write!(f, "Type error: expected {}, got {}", expected, actual)
            },
            EvaluateError::InvalidQuoteSyntax { message, .. } => {
                write!(f, "Invalid quote syntax: {}", message)
            },
            EvaluateError::InvalidIfSyntax { message, .. } => {
                write!(f, "Invalid if syntax: {}", message)
            },
            EvaluateError::InvalidLambdaSyntax { message, .. } => {
                write!(f, "Invalid lambda syntax: {}", message)
            },
            EvaluateError::InvalidDefineSyntax { message, .. } => {
                write!(f, "Invalid define syntax: {}", message)
            },
            EvaluateError::InvalidLetSyntax { message, .. } => {
                write!(f, "Invalid let syntax: {}", message)
            },
            EvaluateError::InvalidLetBinding { message, .. } => {
                write!(f, "Invalid let binding: {}", message)
            },
            EvaluateError::InvalidParameterName { name, .. } => {
                write!(f, "Invalid parameter name: {}", name)
            },
            EvaluateError::InvalidParameterList { message, .. } => {
                write!(f, "Invalid parameter list: {}", message)
            },
            EvaluateError::InvalidArgumentList { message, .. } => {
                write!(f, "Invalid argument list: {}", message)
            },
            EvaluateError::InvalidExpression { message, .. } => {
                write!(f, "Invalid expression: {}", message)
            },
            EvaluateError::StackOverflow { .. } => {
                write!(f, "Stack overflow")
            },
            EvaluateError::OutOfMemory { .. } => {
                write!(f, "Out of memory")
            },
            EvaluateError::InternalError { message, .. } => {
                write!(f, "Internal error: {}", message)
            },
            EvaluateError::NotImplemented { feature, .. } => {
                write!(f, "Feature not implemented: {}", feature)
            },
        }
    }
}

impl std::error::Error for EvaluateError {}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Number(n) => write!(f, "{}", n),
            RuntimeValue::String(s) => write!(f, "\"{}\"", s),
            RuntimeValue::Character(c) => write!(f, "#\\{}", c),
            RuntimeValue::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            RuntimeValue::Symbol(s) => write!(f, "{}", s),
            RuntimeValue::Nil => write!(f, "()"),
            RuntimeValue::Cons { car, cdr } => {
                write!(f, "({}", car)?;
                let mut current = cdr;
                loop {
                    match current.as_ref() {
                        RuntimeValue::Nil => break,
                        RuntimeValue::Cons { car, cdr } => {
                            write!(f, " {}", car)?;
                            current = cdr;
                        },
                        _ => {
                            write!(f, " . {}", current)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            },
            RuntimeValue::Vector(elements) => {
                write!(f, "#(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, ")")
            },
            RuntimeValue::Lambda { .. } => write!(f, "#<procedure>"),
            RuntimeValue::BuiltinFunction { name, .. } => write!(f, "#<procedure:{}>", name),
        }
    }
}
