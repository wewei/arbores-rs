//! 求值结果类型定义
//! 
//! 表示单步求值的三种可能结果

use std::rc::Rc;
use super::{RuntimeObject, EvalState, EvaluateError};

/// 求值步骤结果 - 表示单步求值的三种可能结果
#[derive(Debug)]
pub enum EvaluateResult {
    /// 求值完成，返回最终结果（运行时对象）
    Completed(Rc<RuntimeObject>),
    /// 需要继续求值，返回下一个状态
    Continue(Rc<EvalState>),
    /// 求值出错，返回错误信息
    Error(EvaluateError),
}
