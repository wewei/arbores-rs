//! if 特殊形式实现
//! 
//! if 实现条件分支逻辑，需要先求值条件表达式，然后根据结果选择不同的分支进行求值。

use std::rc::Rc;
use crate::interpreter::{SExpr, SExprContent, Value};
use super::super::types::*;

/// 求值 if 特殊形式
/// 
/// 语法：(if condition then-expr else-expr)
/// 语法：(if condition then-expr) - else 分支可选
pub fn evaluate_if(state: Rc<EvalState>, args: Rc<SExpr>) -> EvaluateResult {
    // TODO: 实现 if 特殊形式
    EvaluateResult::Error(EvaluateError::InvalidIfSyntax {
        span: state.expr.span.clone(),
        message: "if special form not yet implemented".to_string(),
    })
}
