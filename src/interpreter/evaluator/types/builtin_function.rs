//! 内置函数结构定义
//! 
//! 系统提供的内置函数，包含函数名、参数个数要求和实现

use std::rc::Rc;
use gc::{Trace, Finalize};
use super::{RuntimeObject, EvaluateError};

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

/// 内置函数签名 - 包含函数名和参数个数要求
#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    /// 函数名
    pub name: String,
    /// 参数个数要求
    pub arity: FunctionArity,
}

/// 内置函数实现类型
#[derive(Debug, Clone)]
pub struct BuiltinImpl {
    /// 函数实现（接收参数列表，返回结果或错误）
    pub func: fn(&[RuntimeObject]) -> Result<RuntimeObject, EvaluateError>,
}

/// 内置函数结构 - 优化为 16 bytes
#[derive(Debug, Clone, Trace, Finalize)]
pub struct BuiltinFunction {
    /// 函数签名 - 用 Rc 引用，支持共享
    #[unsafe_ignore_trace]
    pub signature: Rc<Signature>,
    /// 函数实现 - 直接嵌入
    #[unsafe_ignore_trace]
    pub implementation: BuiltinImpl,
}

impl PartialEq for BuiltinFunction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.signature, &other.signature) && 
        self.implementation.func as usize == other.implementation.func as usize
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

impl BuiltinFunction {
    /// 创建新的内置函数
    pub fn new(name: String, arity: FunctionArity, func: fn(&[RuntimeObject]) -> Result<RuntimeObject, EvaluateError>) -> Self {
        let signature = Signature { name, arity };
        let implementation = BuiltinImpl { func };
        Self {
            signature: Rc::new(signature),
            implementation,
        }
    }
    
    /// 获取函数名
    pub fn name(&self) -> &str {
        &self.signature.name
    }
    
    /// 获取参数个数要求
    pub fn arity(&self) -> &FunctionArity {
        &self.signature.arity
    }
    
    /// 检查参数个数是否匹配
    pub fn matches_arity(&self, actual: usize) -> bool {
        self.signature.arity.matches(actual)
    }
    
    /// 调用函数
    pub fn call(&self, args: &[RuntimeObject]) -> Result<RuntimeObject, EvaluateError> {
        (self.implementation.func)(args)
    }
}
