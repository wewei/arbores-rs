//! 续延结构定义
//! 
//! 支持 call/cc 的续延结构

use gc::{Finalize, Gc, Trace};
use super::{RuntimeObject, EvaluateResult};

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
