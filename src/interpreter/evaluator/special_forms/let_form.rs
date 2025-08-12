//! let 特殊形式实现
//! 
//! let 创建局部变量绑定，在新的环境中求值表达式。

use crate::interpreter::{SExpr, SExprContent, Value};
use super::super::types::*;

/// 求值 let 特殊形式
/// 
/// 语法：(let ((var1 val1) (var2 val2) ...) body)
/// 创建新的作用域，在其中绑定局部变量
pub fn evaluate_let(state: EvalState, args: &SExpr) -> EvaluateResult {
    // TODO: 实现 let 特殊形式
    EvaluateResult::Error(EvaluateError::InvalidLetSyntax {
        span: state.expr.span.clone(),
        message: "let special form not yet implemented".to_string(),
    })
}
