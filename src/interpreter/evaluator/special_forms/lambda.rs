//! lambda 特殊形式实现
//! 
//! lambda 创建匿名函数（闭包），捕获当前环境并创建可调用的函数对象。

use crate::interpreter::{SExpr, SExprContent, Value};
use super::super::types::*;

/// 求值 lambda 特殊形式
/// 
/// 语法：(lambda (param1 param2 ...) body)
/// 语法：(lambda param body) - 单参数简写形式
/// 语法：(lambda () body) - 无参数形式
pub fn evaluate_lambda(state: EvalState, args: &SExpr) -> EvaluateResult {
    // TODO: 实现 lambda 特殊形式
    EvaluateResult::Error(EvaluateError::InvalidLambdaSyntax {
        span: state.expr.span.clone(),
        message: "lambda special form not yet implemented".to_string(),
    })
}
