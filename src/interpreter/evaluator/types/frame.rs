//! 调用栈帧结构定义
//! 
//! 支持函数调用和续延的调用栈帧

use super::{Environment, Continuation};
use gc::{Trace, Finalize, Gc};

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

impl Frame {
    /// 创建新的根栈帧
    pub fn new_root(env: Gc<Environment>, continuation: Continuation) -> Self {
        Self {
            env,
            continuation: Gc::new(continuation),
            parent: None,
        }
    }
    
    /// 创建带父栈帧的新栈帧
    pub fn with_parent(env: Gc<Environment>, continuation: Continuation, parent: Frame) -> Self {
        Self {
            env,
            continuation: Gc::new(continuation),
            parent: Some(Gc::new(parent)),
        }
    }
}
