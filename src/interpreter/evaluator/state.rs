//! 求值状态管理
//! 
//! 本模块提供求值状态的初始化、转换和管理功能。

use crate::interpreter::SExpr;
use crate::interpreter::evaluator::types::{
    Environment, EvalState, Frame, Continuation, TailContext, 
    EvaluateResult, RuntimeValue, FunctionArity
};

/// 初始化求值状态
/// 
/// 创建一个新的求值状态，包含根栈帧和待求值的表达式。
/// 
/// # 参数
/// - `expr`: 要求值的 S 表达式
/// - `env`: 全局环境
/// 
/// # 返回
/// 初始化好的 `EvalState`
pub fn init_eval_state(expr: Rc<SExpr>, env: Environment) -> EvalState {
    // 创建根栈帧的 continuation，表示求值完成
    let root_continuation = Continuation::new(|result| {
        EvaluateResult::Completed(result)
    });
    
    // 创建根栈帧
    let root_frame = Frame::new_root(env, root_continuation);
    
    // 创建初始求值状态
    EvalState::new(
        root_frame,
        expr,
        TailContext::TailPosition, // 顶层表达式在尾位置
        None, // 顶层表达式没有绑定名称
    )
}

/// 创建全局环境
/// 
/// 创建包含所有内置函数的全局环境。
/// 
/// # 返回
/// 包含内置函数的 `Environment`
pub fn create_global_environment() -> Environment {
    let mut env = Environment::new();
    
    // 添加基础算术运算
    env.define(
        "+".to_string(),
        RuntimeValue::builtin_function(
            "+".to_string(),
            FunctionArity::AtLeast(0),
            builtin_add,
        ),
    );
    
    env.define(
        "-".to_string(),
        RuntimeValue::builtin_function(
            "-".to_string(),
            FunctionArity::AtLeast(1),
            builtin_subtract,
        ),
    );
    
    env.define(
        "*".to_string(),
        RuntimeValue::builtin_function(
            "*".to_string(),
            FunctionArity::AtLeast(0),
            builtin_multiply,
        ),
    );
    
    env.define(
        "/".to_string(),
        RuntimeValue::builtin_function(
            "/".to_string(),
            FunctionArity::AtLeast(1),
            builtin_divide,
        ),
    );
    
    // 添加比较运算
    env.define(
        "=".to_string(),
        RuntimeValue::builtin_function(
            "=".to_string(),
            FunctionArity::AtLeast(2),
            builtin_equal,
        ),
    );
    
    env.define(
        "<".to_string(),
        RuntimeValue::builtin_function(
            "<".to_string(),
            FunctionArity::AtLeast(2),
            builtin_less_than,
        ),
    );
    
    env.define(
        ">".to_string(),
        RuntimeValue::builtin_function(
            ">".to_string(),
            FunctionArity::AtLeast(2),
            builtin_greater_than,
        ),
    );
    
    // 添加列表操作
    env.define(
        "car".to_string(),
        RuntimeValue::builtin_function(
            "car".to_string(),
            FunctionArity::Exact(1),
            builtin_car,
        ),
    );
    
    env.define(
        "cdr".to_string(),
        RuntimeValue::builtin_function(
            "cdr".to_string(),
            FunctionArity::Exact(1),
            builtin_cdr,
        ),
    );
    
    env.define(
        "cons".to_string(),
        RuntimeValue::builtin_function(
            "cons".to_string(),
            FunctionArity::Exact(2),
            builtin_cons,
        ),
    );
    
    env.define(
        "list".to_string(),
        RuntimeValue::builtin_function(
            "list".to_string(),
            FunctionArity::AtLeast(0),
            builtin_list,
        ),
    );
    
    // 添加类型判断
    env.define(
        "number?".to_string(),
        RuntimeValue::builtin_function(
            "number?".to_string(),
            FunctionArity::Exact(1),
            builtin_is_number,
        ),
    );
    
    env.define(
        "string?".to_string(),
        RuntimeValue::builtin_function(
            "string?".to_string(),
            FunctionArity::Exact(1),
            builtin_is_string,
        ),
    );
    
    env.define(
        "boolean?".to_string(),
        RuntimeValue::builtin_function(
            "boolean?".to_string(),
            FunctionArity::Exact(1),
            builtin_is_boolean,
        ),
    );
    
    env.define(
        "null?".to_string(),
        RuntimeValue::builtin_function(
            "null?".to_string(),
            FunctionArity::Exact(1),
            builtin_is_null,
        ),
    );
    
    env.define(
        "pair?".to_string(),
        RuntimeValue::builtin_function(
            "pair?".to_string(),
            FunctionArity::Exact(1),
            builtin_is_pair,
        ),
    );
    
    env
}

// ============================================================================
// 内置函数实现
// ============================================================================

use crate::interpreter::evaluator::types::EvaluateError;
use crate::interpreter::lexer::types::Span;
use std::rc::Rc;

// 算术运算

fn builtin_add(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    let mut result = 0.0;
    for arg in args {
        match arg {
            RuntimeValue::Number(n) => result += n,
            _ => return Err(EvaluateError::TypeMismatch {
                span: Rc::new(Span::empty(0)),
                expected: "number".to_string(),
                actual: arg.type_name().to_string(),
            }),
        }
    }
    Ok(RuntimeValue::Number(result))
}

fn builtin_subtract(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.is_empty() {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "at least 1".to_string(),
            actual: 0,
        });
    }
    
    match &args[0] {
        RuntimeValue::Number(first) => {
            if args.len() == 1 {
                // 一元减号
                Ok(RuntimeValue::Number(-first))
            } else {
                // 多元减法
                let mut result = *first;
                for arg in &args[1..] {
                    match arg {
                        RuntimeValue::Number(n) => result -= n,
                        _ => return Err(EvaluateError::TypeMismatch {
                            span: Rc::new(Span::empty(0)),
                            expected: "number".to_string(),
                            actual: arg.type_name().to_string(),
                        }),
                    }
                }
                Ok(RuntimeValue::Number(result))
            }
        },
        _ => Err(EvaluateError::TypeMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "number".to_string(),
            actual: args[0].type_name().to_string(),
        }),
    }
}

fn builtin_multiply(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    let mut result = 1.0;
    for arg in args {
        match arg {
            RuntimeValue::Number(n) => result *= n,
            _ => return Err(EvaluateError::TypeMismatch {
                span: Rc::new(Span::empty(0)),
                expected: "number".to_string(),
                actual: arg.type_name().to_string(),
            }),
        }
    }
    Ok(RuntimeValue::Number(result))
}

fn builtin_divide(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.is_empty() {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "at least 1".to_string(),
            actual: 0,
        });
    }
    
    match &args[0] {
        RuntimeValue::Number(first) => {
            if args.len() == 1 {
                // 一元除法：1/x
                if *first == 0.0 {
                    return Err(EvaluateError::DivisionByZero {
                        span: Rc::new(Span::empty(0)),
                    });
                }
                Ok(RuntimeValue::Number(1.0 / first))
            } else {
                // 多元除法
                let mut result = *first;
                for arg in &args[1..] {
                    match arg {
                        RuntimeValue::Number(n) => {
                            if *n == 0.0 {
                                return Err(EvaluateError::DivisionByZero {
                                    span: Rc::new(Span::empty(0)),
                                });
                            }
                            result /= n;
                        },
                        _ => return Err(EvaluateError::TypeMismatch {
                            span: Rc::new(Span::empty(0)),
                            expected: "number".to_string(),
                            actual: arg.type_name().to_string(),
                        }),
                    }
                }
                Ok(RuntimeValue::Number(result))
            }
        },
        _ => Err(EvaluateError::TypeMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "number".to_string(),
            actual: args[0].type_name().to_string(),
        }),
    }
}

// 比较运算

fn builtin_equal(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() < 2 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "at least 2".to_string(),
            actual: args.len(),
        });
    }
    
    // 检查所有参数是否都是数字且相等
    let first = match &args[0] {
        RuntimeValue::Number(n) => n,
        _ => return Err(EvaluateError::TypeMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "number".to_string(),
            actual: args[0].type_name().to_string(),
        }),
    };
    
    for arg in &args[1..] {
        match arg {
            RuntimeValue::Number(n) => {
                if (first - n).abs() > f64::EPSILON {
                    return Ok(RuntimeValue::Boolean(false));
                }
            },
            _ => return Err(EvaluateError::TypeMismatch {
                span: Rc::new(Span::empty(0)),
                expected: "number".to_string(),
                actual: arg.type_name().to_string(),
            }),
        }
    }
    
    Ok(RuntimeValue::Boolean(true))
}

fn builtin_less_than(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() < 2 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "at least 2".to_string(),
            actual: args.len(),
        });
    }
    
    for i in 0..args.len() - 1 {
        let current = match &args[i] {
            RuntimeValue::Number(n) => n,
            _ => return Err(EvaluateError::TypeMismatch {
                span: Rc::new(Span::empty(0)),
                expected: "number".to_string(),
                actual: args[i].type_name().to_string(),
            }),
        };
        
        let next = match &args[i + 1] {
            RuntimeValue::Number(n) => n,
            _ => return Err(EvaluateError::TypeMismatch {
                span: Rc::new(Span::empty(0)),
                expected: "number".to_string(),
                actual: args[i + 1].type_name().to_string(),
            }),
        };
        
        if current >= next {
            return Ok(RuntimeValue::Boolean(false));
        }
    }
    
    Ok(RuntimeValue::Boolean(true))
}

fn builtin_greater_than(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() < 2 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "at least 2".to_string(),
            actual: args.len(),
        });
    }
    
    for i in 0..args.len() - 1 {
        let current = match &args[i] {
            RuntimeValue::Number(n) => n,
            _ => return Err(EvaluateError::TypeMismatch {
                span: Rc::new(Span::empty(0)),
                expected: "number".to_string(),
                actual: args[i].type_name().to_string(),
            }),
        };
        
        let next = match &args[i + 1] {
            RuntimeValue::Number(n) => n,
            _ => return Err(EvaluateError::TypeMismatch {
                span: Rc::new(Span::empty(0)),
                expected: "number".to_string(),
                actual: args[i + 1].type_name().to_string(),
            }),
        };
        
        if current <= next {
            return Ok(RuntimeValue::Boolean(false));
        }
    }
    
    Ok(RuntimeValue::Boolean(true))
}

// 列表操作

fn builtin_car(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 1 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "1".to_string(),
            actual: args.len(),
        });
    }
    
    match &args[0] {
        RuntimeValue::Cons { car, .. } => Ok((**car).clone()),
        _ => Err(EvaluateError::TypeMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "pair".to_string(),
            actual: args[0].type_name().to_string(),
        }),
    }
}

fn builtin_cdr(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 1 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "1".to_string(),
            actual: args.len(),
        });
    }
    
    match &args[0] {
        RuntimeValue::Cons { cdr, .. } => Ok((**cdr).clone()),
        _ => Err(EvaluateError::TypeMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "pair".to_string(),
            actual: args[0].type_name().to_string(),
        }),
    }
}

fn builtin_cons(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 2 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "2".to_string(),
            actual: args.len(),
        });
    }
    
    Ok(RuntimeValue::Cons {
        car: Rc::new(args[0].clone()),
        cdr: Rc::new(args[1].clone()),
    })
}

fn builtin_list(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    let mut result = RuntimeValue::Nil;
    
    // 从后往前构建列表
    for arg in args.iter().rev() {
        result = RuntimeValue::Cons {
            car: Rc::new(arg.clone()),
            cdr: Rc::new(result),
        };
    }
    
    Ok(result)
}

// 类型判断

fn builtin_is_number(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 1 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "1".to_string(),
            actual: args.len(),
        });
    }
    
    Ok(RuntimeValue::Boolean(matches!(args[0], RuntimeValue::Number(_))))
}

fn builtin_is_string(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 1 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "1".to_string(),
            actual: args.len(),
        });
    }
    
    Ok(RuntimeValue::Boolean(matches!(args[0], RuntimeValue::String(_))))
}

fn builtin_is_boolean(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 1 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "1".to_string(),
            actual: args.len(),
        });
    }
    
    Ok(RuntimeValue::Boolean(matches!(args[0], RuntimeValue::Boolean(_))))
}

fn builtin_is_null(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 1 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "1".to_string(),
            actual: args.len(),
        });
    }
    
    Ok(RuntimeValue::Boolean(matches!(args[0], RuntimeValue::Nil)))
}

fn builtin_is_pair(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.len() != 1 {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: Rc::new(Span::empty(0)),
            expected: "1".to_string(),
            actual: args.len(),
        });
    }
    
    Ok(RuntimeValue::Boolean(matches!(args[0], RuntimeValue::Cons { .. })))
}
