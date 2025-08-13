//! 内置函数结构定义
//! 
//! 系统提供的内置函数，包含函数名、参数个数要求和实现

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

/// 内置函数实现类型
#[derive(Debug, Clone)]
pub struct BuiltinImpl {
    /// 函数实现（接收参数列表，返回结果或错误）
    pub func: fn(&[RuntimeObject]) -> Result<RuntimeObject, EvaluateError>,
}

/// 内置函数结构
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub arity: FunctionArity,
    pub implementation: BuiltinImpl,
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
