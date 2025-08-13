//! quote 特殊形式实现
//! 
//! quote 阻止对表达式的求值，直接返回表达式的字面值。

use std::rc::Rc;
use crate::interpreter::{SExpr, SExprContent, Value};
use super::super::types::*;

/// 求值 quote 特殊形式
/// 
/// 语法：(quote expr) 或简写为 'expr
/// 返回 expr 的字面值，不进行求值
pub fn evaluate_quote(state: Rc<EvalState>, args: Rc<SExpr>) -> EvaluateResult {
    // quote 只接受一个参数
    match &args.content {
        SExprContent::Cons { car, cdr } => {
            // 检查是否只有一个参数
            match &cdr.content {
                SExprContent::Nil => {
                    // 将被引用的表达式转换为运行时值并返回
                    let quoted_value = s_expr_to_runtime_value(car);
                    (state.as_ref().frame.continuation.func)(quoted_value)
                },
                _ => {
                    // quote 接受多个参数是错误的
                    EvaluateResult::Error(EvaluateError::InvalidQuoteSyntax {
                        span: state.as_ref().expr.span.clone(),
                        message: "quote expects exactly one argument".to_string(),
                    })
                }
            }
        },
        SExprContent::Nil => {
            // quote 没有参数是错误的
            EvaluateResult::Error(EvaluateError::InvalidQuoteSyntax {
                span: state.as_ref().expr.span.clone(),
                message: "quote expects exactly one argument".to_string(),
            })
        },
        _ => {
            // 参数列表格式错误
            EvaluateResult::Error(EvaluateError::InvalidArgumentList {
                span: state.as_ref().expr.span.clone(),
                message: "quote argument must be a list".to_string(),
            })
        }
    }
}

/// 将 SExpr 转换为 RuntimeValue
/// 
/// 这是一个递归转换函数，保持表达式的原始结构
fn s_expr_to_runtime_value(expr: &SExpr) -> RuntimeValue {
    match &expr.content {
        SExprContent::Atom(Value::Number(n)) => RuntimeValue::Number(*n),
        SExprContent::Atom(Value::String(s)) => RuntimeValue::String(s.clone()),
        SExprContent::Atom(Value::Character(c)) => RuntimeValue::Character(*c),
        SExprContent::Atom(Value::Boolean(b)) => RuntimeValue::Boolean(*b),
        SExprContent::Atom(Value::Symbol(s)) => RuntimeValue::Symbol(s.clone()),
        SExprContent::Cons { car, cdr } => {
            RuntimeValue::Cons {
                car: Rc::new(s_expr_to_runtime_value(car)),
                cdr: Rc::new(s_expr_to_runtime_value(cdr)),
            }
        },
        SExprContent::Nil => RuntimeValue::Nil,
        SExprContent::Vector(elements) => {
            let runtime_elements: Vec<RuntimeValue> = elements
                .iter()
                .map(|e| s_expr_to_runtime_value(e))
                .collect();
            RuntimeValue::Vector(Rc::new(runtime_elements))
        },
    }
}
