//! 求值状态类型定义
//! 
//! 表示求值过程中的当前状态

use std::rc::Rc;
use crate::interpreter::SExpr;
use super::Frame;

/// 尾调用上下文 - 标记当前表达式是否在尾位置
#[derive(Clone, Debug, PartialEq)]
pub enum TailContext {
    /// 在尾位置，可以进行尾调用优化
    TailPosition,
    /// 不在尾位置，需要保留调用上下文
    NonTailPosition,
}

/// 求值状态 - 表示求值过程中的当前状态
#[derive(Debug)]
pub struct EvalState {
    /// 当前调用栈 Frame
    pub frame: Rc<Frame>,
    /// 待求值表达式
    pub expr: Rc<SExpr>,
    /// 尾调用上下文信息（用于尾调用优化）
    pub tail_context: TailContext,
    /// 当前表达式的绑定名称（如果有的话）
    pub binding_name: Option<String>,
}

impl EvalState {
    /// 创建新的求值状态
    pub fn new(
        frame: Frame, 
        expr: Rc<SExpr>, 
        tail_context: TailContext,
        binding_name: Option<String>
    ) -> Self {
        Self {
            frame: Rc::new(frame),
            expr,
            tail_context,
            binding_name,
        }
    }
}
