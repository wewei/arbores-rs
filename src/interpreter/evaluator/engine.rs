//! 求值引擎模块
//! 
//! 实现基于 RuntimeObject 设计的求值引擎，支持：
//! - 状态机模式的单步求值
//! - 尾调用优化
//! - 垃圾回收兼容的续延系统

use std::rc::Rc;
use gc::{Gc, Trace, Finalize};

use crate::interpreter::{SExpr, SExprContent, Value};
use super::types::*;

// ============================================================================
// 续延实现
// ============================================================================



// ============================================================================
// SExpr 到 RuntimeObject 转换
// ============================================================================

/// 将 SExpr 转换为 RuntimeObject
fn convert_sexpr_to_runtime_object(expr: Rc<SExpr>) -> Result<Gc<RuntimeObject>, EvaluateError> {
    let runtime_obj = match &expr.content {
        SExprContent::Atom(Value::Number(n)) => {
            RuntimeObject {
                core: RuntimeObjectCore::Float(*n),
                source: Some(expr.clone()),
            }
        },
        SExprContent::Atom(Value::String(s)) => {
            RuntimeObject {
                core: RuntimeObjectCore::String(StringRef::from(s.clone())),
                source: Some(expr.clone()),
            }
        },
        SExprContent::Atom(Value::Boolean(b)) => {
            RuntimeObject {
                core: RuntimeObjectCore::Boolean(*b),
                source: Some(expr.clone()),
            }
        },
        SExprContent::Atom(Value::Character(c)) => {
            RuntimeObject {
                core: RuntimeObjectCore::Character(*c),
                source: Some(expr.clone()),
            }
        },
        SExprContent::Atom(Value::Symbol(name)) => {
            RuntimeObject {
                core: RuntimeObjectCore::Symbol(StringRef::from(name.clone())),
                source: Some(expr.clone()),
            }
        },
        SExprContent::Nil => {
            RuntimeObject {
                core: RuntimeObjectCore::Nil,
                source: Some(expr.clone()),
            }
        },
        SExprContent::Cons { car, cdr } => {
            // 暂时不支持列表转换，返回错误
            return Err(EvaluateError::NotImplemented {
                expr: expr.clone(),
                feature: "list conversion".to_string(),
            });
        },
        SExprContent::Vector(elements) => {
            // 暂时不支持向量转换，返回错误
            return Err(EvaluateError::NotImplemented {
                expr: expr.clone(),
                feature: "vector conversion".to_string(),
            });
        },
    };
    
    Ok(Gc::new(runtime_obj))
}

// ============================================================================
// 主求值函数
// ============================================================================

/// 主求值函数 - 对外接口
/// 
/// # 参数
/// - `expr`: 要求值的 S 表达式
/// - `env`: 全局环境
/// 
/// # 返回值
/// - 成功时返回求值结果的运行时对象
/// - 失败时返回求值错误
pub fn evaluate(expr: Rc<SExpr>, env: Gc<Environment>) -> Result<Rc<RuntimeObject>, EvaluateError> {
    // 将 SExpr 转换为 RuntimeObject
    let runtime_expr = convert_sexpr_to_runtime_object(expr)?;
    let mut current_state = Rc::new(init_eval_state(runtime_expr, env));
    
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
/// - `state`: 当前求值状态
/// 
/// # 返回值
/// - 三分枝结果：Completed(结果)、Continue(下一状态)、Error(错误)
pub fn evaluate_step(state: Rc<EvalState>) -> EvaluateResult {
    match &state.expr.core {
        // === 自求值表达式 ===
        RuntimeObjectCore::Integer(_) | RuntimeObjectCore::Float(_) | 
        RuntimeObjectCore::Boolean(_) | RuntimeObjectCore::Character(_) | 
        RuntimeObjectCore::String(_) | RuntimeObjectCore::Nil => {
            // 自求值表达式直接调用 continuation
            state.frame.continuation.call(state.expr.clone())
        },
        
        // === 符号（变量引用）===
        RuntimeObjectCore::Symbol(name) => {
            // 符号：变量引用，在环境中查找值
            match state.frame.env.lookup(name.as_str()) {
                Some(value) => state.frame.continuation.call(Gc::new(value)),
                None => {
                    // 如果环境中没有找到，检查是否是内置函数
                    let builtin_names = ["+", "-", "*", "/"];
                    if builtin_names.contains(&name.as_str()) {
                        let builtin_obj = create_builtin_function(name.as_str());
                        state.frame.continuation.call(Gc::new(builtin_obj))
                    } else {
                        EvaluateResult::Error(EvaluateError::UndefinedVariable {
                            expr: state.expr.source.clone().unwrap_or_else(|| {
                                Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol(name.to_string()))))
                            }),
                            name: name.to_string(),
                        })
                    }
                }
            }
        },
        
        // === 列表表达式 ===
        RuntimeObjectCore::Cons(cons) => {
            evaluate_list_expression(state.clone(), cons)
        },
        
        // === 其他类型 ===
        _ => EvaluateResult::Error(EvaluateError::InvalidExpression {
            expr: state.expr.source.clone().unwrap_or_else(|| {
                Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("unknown".to_string()))))
            }),
            message: format!("Unsupported expression type: {:?}", state.expr.core),
        }),
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 初始化求值状态
fn init_eval_state(expr: Gc<RuntimeObject>, env: Gc<Environment>) -> EvalState {
    let root_frame = Frame::new_root(env, Continuation::Root);
    
    EvalState::new(
        Gc::new(root_frame),
        expr,
        TailContext::TailPosition, // 顶层表达式在尾位置
        None, // 顶层表达式没有绑定名称
    )
}

/// 求值列表表达式（函数调用或特殊形式）
fn evaluate_list_expression(state: Rc<EvalState>, cons: &MutableCons) -> EvaluateResult {
    // 暂时不支持列表求值，返回错误
    EvaluateResult::Error(EvaluateError::NotImplemented {
        expr: state.expr.source.clone().unwrap_or_else(|| {
            Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("list".to_string()))))
        }),
        feature: "list evaluation".to_string(),
    })
}

/// 创建内置函数对象
fn create_builtin_function(name: &str) -> RuntimeObject {
    use super::builtins::arithmetic::*;
    
    let builtin = match name {
        "+" => create_add_function(),
        "-" => create_subtract_function(),
        "*" => create_multiply_function(),
        "/" => create_divide_function(),
        _ => {
            // 未知函数，返回错误
            return RuntimeObject {
                core: RuntimeObjectCore::String(StringRef::from(format!("Unknown function: {}", name))),
                source: None,
            };
        }
    };
    
    RuntimeObject {
        core: RuntimeObjectCore::BuiltinFunction(builtin),
        source: None,
    }
}
