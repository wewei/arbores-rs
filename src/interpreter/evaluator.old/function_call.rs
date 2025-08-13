//! 函数调用机制
//! 
//! 本模块实现函数调用的求值逻辑，包括：
//! - 函数表达式的求值
//! - 参数的依次求值  
//! - 函数的应用（内置函数和用户定义函数）

use super::types::*;
use crate::interpreter::{SExpr, SExprContent, Value};
use std::rc::Rc;

/// 求值函数调用表达式
/// 
/// 处理形如 (operator operand1 operand2 ...) 的表达式
/// 
/// # 参数
/// - `state`: 当前求值状态
/// - `operator`: 函数表达式
/// - `operands`: 参数表达式列表
/// 
/// # 返回
/// 求值结果
pub fn evaluate_function_call(state: Rc<EvalState>, operator: Rc<SExpr>, operands: Rc<SExpr>) -> EvaluateResult {
    // 第一阶段：求值函数表达式
    let function_continuation = create_function_eval_continuation(state.clone(), operands.clone());
    
    let function_frame = Frame {
        env: state.as_ref().frame.as_ref().env.clone(),
        continuation: function_continuation,
        parent: Some(state.as_ref().frame.clone()),
    };
    
    EvaluateResult::Continue(Rc::new(EvalState {
        frame: Rc::new(function_frame),
        expr: operator.clone(),
        tail_context: TailContext::NonTailPosition, // 函数求值不在尾位置
        binding_name: None,
    }))
}

/// 创建函数求值完成后的 continuation
fn create_function_eval_continuation(
    original_state: Rc<EvalState>,
    operands: Rc<SExpr>,
) -> Continuation {
    Continuation {
        func: Rc::new(move |function_value| {
            // 函数求值完成，开始求值参数
            evaluate_arguments(
                original_state.clone(),
                function_value,
                operands.clone(),
                Vec::new(),
            )
        }),
    }
}

/// 求值参数序列
/// 
/// # 参数
/// - `state`: 原始求值状态
/// - `function_value`: 已求值的函数
/// - `remaining_args`: 剩余未求值的参数
/// - `evaluated_args`: 已求值的参数
fn evaluate_arguments(
    state: Rc<EvalState>,
    function_value: RuntimeValue,
    remaining_args: Rc<SExpr>,
    evaluated_args: Vec<RuntimeValue>,
) -> EvaluateResult {
    match &remaining_args.content {
        // 没有更多参数，开始函数应用
        SExprContent::Nil => {
            apply_function(state, function_value, evaluated_args)
        },
        
        // 还有参数需要求值
        SExprContent::Cons { car, cdr } => {
            let arg_continuation = create_argument_eval_continuation(
                state.clone(),
                function_value,
                cdr.clone(),
                evaluated_args,
            );
            
            let arg_frame = Frame {
                env: state.as_ref().frame.as_ref().env.clone(),
                continuation: arg_continuation,
                parent: Some(state.as_ref().frame.clone()),
            };
            
            EvaluateResult::Continue(Rc::new(EvalState {
                frame: Rc::new(arg_frame),
                expr: car.clone(),
                tail_context: TailContext::NonTailPosition, // 参数求值不在尾位置
                binding_name: None,
            }))
        },
        
        _ => EvaluateResult::Error(EvaluateError::InvalidArgumentList {
            span: remaining_args.span.clone(),
            message: "Invalid argument list format".to_string(),
        }),
    }
}

/// 创建参数求值完成后的 continuation
fn create_argument_eval_continuation(
    state: Rc<EvalState>,
    function_value: RuntimeValue,
    remaining_args: Rc<SExpr>,
    mut evaluated_args: Vec<RuntimeValue>,
) -> Continuation {
    Continuation {
        func: Rc::new(move |arg_value| {
            // 参数求值完成，继续求值下一个参数
            let mut new_evaluated_args = evaluated_args.clone();
            new_evaluated_args.push(arg_value);
            
            evaluate_arguments(
                state.clone(),
                function_value.clone(),
                remaining_args.clone(),
                new_evaluated_args,
            )
        }),
    }
}

/// 应用函数到参数
/// 
/// # 参数
/// - `state`: 当前求值状态
/// - `function_value`: 要应用的函数
/// - `arguments`: 求值后的参数列表
fn apply_function(
    state: Rc<EvalState>,
    function_value: RuntimeValue,
    arguments: Vec<RuntimeValue>,
) -> EvaluateResult {
    match &function_value {
        // 内置函数
        RuntimeValue::BuiltinFunction { name: _, arity, implementation } => {
            // 检查参数个数
            if !arity.matches(arguments.len()) {
                return EvaluateResult::Error(EvaluateError::ArgumentCountMismatch {
                    span: state.as_ref().expr.span.clone(),
                    expected: format!("{:?}", arity),
                    actual: arguments.len(),
                });
            }
            
            // 调用内置函数
            match (implementation.func)(&arguments) {
                Ok(result) => (state.as_ref().frame.continuation.func)(result),
                Err(error) => EvaluateResult::Error(error),
            }
        },
        
        // 用户定义的 Lambda 函数（暂时不实现）
        RuntimeValue::Lambda { .. } => {
            EvaluateResult::Error(EvaluateError::NotImplemented {
                span: state.as_ref().expr.span.clone(),
                feature: "Lambda function calls".to_string(),
            })
        },
        
        _ => EvaluateResult::Error(EvaluateError::NotCallable {
            span: state.as_ref().expr.span.clone(),
            value: format!("{:?}", function_value),
        }),
    }
}
