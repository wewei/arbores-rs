//! 主求值引擎
//! 
//! 本模块实现求值器的核心逻辑，包括主求值循环和单步状态转移。
//! 特殊形式和函数调用的具体实现委托给专门的模块。

use crate::interpreter::{SExpr, SExprContent, Value};
use super::types::*;
use super::state::*;

/// 主求值函数 - 对外接口
/// 
/// # 参数
/// 
/// * `expr` - 要求值的 S 表达式
/// * `env` - 全局环境
/// 
/// # 返回值
/// 
/// 求值结果的运行时值或错误信息
pub fn evaluate(expr: SExpr, env: Environment) -> Result<RuntimeValue, EvaluateError> {
    let mut current_state = init_eval_state(expr, env);
    
    loop {
        match evaluate_step(current_state) {
            EvaluateResult::Completed(result) => return Ok(result),
            EvaluateResult::Continue(next_state) => {
                current_state = next_state;
            },
            EvaluateResult::Error(error) => return Err(error),
        }
    }
}

/// 单步状态转移函数 - 对外接口
/// 
/// # 参数
/// 
/// * `state` - 当前求值状态
/// 
/// # 返回值
/// 
/// 三分枝结果：Completed(结果)、Continue(下一状态)、Error(错误)
pub fn evaluate_step(state: EvalState) -> EvaluateResult {
    match &state.expr.content {
        // 自求值表达式（原子值）
        SExprContent::Atom(Value::Number(n)) => {
            (state.frame.continuation.func)(RuntimeValue::Number(*n))
        },
        SExprContent::Atom(Value::String(s)) => {
            (state.frame.continuation.func)(RuntimeValue::String(s.clone()))
        },
        SExprContent::Atom(Value::Boolean(b)) => {
            (state.frame.continuation.func)(RuntimeValue::Boolean(*b))
        },
        SExprContent::Atom(Value::Character(c)) => {
            (state.frame.continuation.func)(RuntimeValue::Character(*c))
        },
        
        // 符号（变量引用）
        SExprContent::Atom(Value::Symbol(name)) => {
            match lookup_variable(&name, &state.frame.env) {
                Some(value) => (state.frame.continuation.func)(value.clone()),
                None => EvaluateResult::Error(EvaluateError::UndefinedVariable {
                    name: name.clone(),
                    span: state.expr.span.clone(),
                }),
            }
        },
        
        // 列表表达式
        SExprContent::Cons { car, cdr } => {
            evaluate_list_expression(state.clone(), car.as_ref(), cdr.as_ref())
        },
        
        // 空列表
        SExprContent::Nil => {
            (state.frame.continuation.func)(RuntimeValue::Nil)
        },
        
        // 其他情况
        _ => EvaluateResult::Error(EvaluateError::InvalidExpression {
            span: state.expr.span.clone(),
            message: format!("Unsupported expression type: {:?}", state.expr.content),
        }),
    }
}

/// 求值列表表达式
/// 
/// 根据第一个元素判断是特殊形式还是函数调用
fn evaluate_list_expression(state: EvalState, car: &SExpr, cdr: &SExpr) -> EvaluateResult {
    // 检查第一个元素是否为特殊形式关键字
    if let SExprContent::Atom(Value::Symbol(operator)) = &car.content {
        match operator.as_str() {
            "quote" => {
                // 委托给特殊形式模块
                crate::interpreter::evaluator::special_forms::quote::evaluate_quote(state, cdr)
            },
            "if" => {
                // 委托给特殊形式模块
                crate::interpreter::evaluator::special_forms::if_form::evaluate_if(state, cdr)
            },
            "lambda" => {
                // 委托给特殊形式模块
                crate::interpreter::evaluator::special_forms::lambda::evaluate_lambda(state, cdr)
            },
            "define" => {
                // 委托给特殊形式模块
                crate::interpreter::evaluator::special_forms::define::evaluate_define(state, cdr)
            },
            "let" => {
                // 委托给特殊形式模块
                crate::interpreter::evaluator::special_forms::let_form::evaluate_let(state, cdr)
            },
            // 不是特殊形式，按函数调用处理
            _ => {
                // 委托给函数调用模块
                crate::interpreter::evaluator::function_call::evaluate_function_call(state, car, cdr)
            }
        }
    } else {
        // 第一个元素不是符号，按函数调用处理（可能是 lambda 表达式）
        crate::interpreter::evaluator::function_call::evaluate_function_call(state, car, cdr)
    }
}

/// 在环境中查找变量
fn lookup_variable(name: &str, env: &Environment) -> Option<RuntimeValue> {
    // 在当前环境中查找
    if let Some(value) = env.bindings.get(name) {
        return Some(value.clone());
    }
    
    // 在父环境中查找
    if let Some(ref parent) = env.parent {
        lookup_variable(name, parent)
    } else {
        None
    }
}

/// 使用全局环境进行求值 - 便捷函数
pub fn evaluate_with_global_env(expr: SExpr) -> Result<RuntimeValue, EvaluateError> {
    let global_env = create_global_environment();
    evaluate(expr, global_env)
}
