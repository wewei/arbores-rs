//! 续延结构定义
//! 
//! 支持 call/cc 的续延结构

use gc::{Finalize, Gc, Trace};
use super::{RuntimeObject, EvaluateResult};

/// 续延函数 trait - 替代 Box<dyn Fn> 以支持 Trace
pub trait ContinuationFn: Trace + Finalize {
    fn call(&self, value: Gc<RuntimeObject>) -> EvaluateResult;
}

/// 简单的续延函数实现
#[derive(Trace, Finalize)]
pub struct SimpleContinuation {
    pub func: fn(Gc<RuntimeObject>) -> EvaluateResult,
}

impl ContinuationFn for SimpleContinuation {
    fn call(&self, value: Gc<RuntimeObject>) -> EvaluateResult {
        (self.func)(value)
    }
}

/// 续延结构 - 支持 call/cc
#[derive(Clone, Trace, Finalize)]
pub struct Continuation {
    /// 续延函数 - 使用 trait object 而不是 Box<dyn Fn>
    pub func: Gc<dyn ContinuationFn>,
}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(&self.func, &other.func)
    }
}

impl std::fmt::Debug for Continuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Continuation {{ func: <closure> }}")
    }
}

impl std::fmt::Debug for SimpleContinuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleContinuation {{ func: <function> }}")
    }
}