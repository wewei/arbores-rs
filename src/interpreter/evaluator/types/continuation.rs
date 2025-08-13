//! 续延结构定义
//! 
//! 支持 call/cc 的续延结构

use std::rc::Rc;
use super::{RuntimeObject, EvaluateResult};

/// 续延结构 - 支持 call/cc
pub struct Continuation {
    /// 续延函数 - 捕获了必要的上下文（环境、调用栈等）
    pub func: Box<dyn Fn(Rc<RuntimeObject>) -> EvaluateResult>,
}

impl Continuation {
    /// 创建新的续延
    pub fn new<F>(func: F) -> Self 
    where 
        F: Fn(Rc<RuntimeObject>) -> EvaluateResult + 'static
    {
        Self {
            func: Box::new(func),
        }
    }
    
    /// 调用续延函数
    pub fn call(&self, value: Rc<RuntimeObject>) -> EvaluateResult {
        (self.func)(value)
    }
}

impl std::fmt::Debug for Continuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Continuation {{ func: <closure> }}")
    }
}
