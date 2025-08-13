//! 调用栈帧结构定义
//! 
//! 支持函数调用和续延的调用栈帧

use std::rc::Rc;
use super::{Environment, Continuation};

/// 调用栈帧 - 支持函数调用和续延
#[derive(Debug)]
pub struct Frame {
    /// 当前环境
    pub env: Rc<Environment>,
    /// 续延
    pub continuation: Rc<Continuation>,  // Rc 包装的续延函数
    /// 父栈帧
    pub parent: Option<Rc<Frame>>,
}

impl Frame {
    /// 创建新的根栈帧
    pub fn new_root(env: Environment, continuation: Continuation) -> Self {
        Self {
            env: Rc::new(env),
            continuation: Rc::new(continuation),
            parent: None,
        }
    }
    
    /// 创建带父栈帧的新栈帧
    pub fn with_parent(env: Environment, continuation: Continuation, parent: Frame) -> Self {
        Self {
            env: Rc::new(env),
            continuation: Rc::new(continuation),
            parent: Some(Rc::new(parent)),
        }
    }
}
