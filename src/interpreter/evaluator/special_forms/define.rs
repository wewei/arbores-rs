//! define 特殊形式实现
//! 
//! define 用于在当前环境中定义变量或函数，支持简单变量定义和函数定义的语法糖。

use crate::interpreter::{SExpr, SExprContent, Value};
use super::super::types::*;

/// 求值 define 特殊形式
/// 
/// 语法：(define name value) - 变量定义
/// 语法：(define (name param1 param2 ...) body) - 函数定义语法糖
pub fn evaluate_define(state: EvalState, args: &SExpr) -> EvaluateResult {
    // TODO: 实现 define 特殊形式
    EvaluateResult::Error(EvaluateError::InvalidDefineSyntax {
        span: state.expr.span.clone(),
        message: "define special form not yet implemented".to_string(),
    })
}
