//! 算术运算内置函数
//! 
//! 本模块实现基础的算术运算函数：+, -, *, /, =, <, > 等

use super::super::types::*;
use std::rc::Rc;

/// 注册所有算术运算函数到环境中
pub fn register_arithmetic_functions(env: &mut Environment) {
    // 加法
    env.define(
        "+".to_string(),
        RuntimeValue::BuiltinFunction {
            name: "+".to_string(),
            arity: FunctionArity::AtLeast(0),
            implementation: BuiltinImpl {
                func: add_function,
            },
        },
    );
    
    // 减法
    env.define(
        "-".to_string(),
        RuntimeValue::BuiltinFunction {
            name: "-".to_string(),
            arity: FunctionArity::AtLeast(1),
            implementation: BuiltinImpl {
                func: subtract_function,
            },
        },
    );
    
    // 乘法
    env.define(
        "*".to_string(),
        RuntimeValue::BuiltinFunction {
            name: "*".to_string(),
            arity: FunctionArity::AtLeast(0),
            implementation: BuiltinImpl {
                func: multiply_function,
            },
        },
    );
    
    // 除法
    env.define(
        "/".to_string(),
        RuntimeValue::BuiltinFunction {
            name: "/".to_string(),
            arity: FunctionArity::AtLeast(1),
            implementation: BuiltinImpl {
                func: divide_function,
            },
        },
    );
}

/// 加法函数实现
fn add_function(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    let mut sum = 0.0;
    
    for arg in args {
        match arg {
            RuntimeValue::Number(n) => {
                sum += n;
            },
            _ => {
                return Err(EvaluateError::TypeError {
                    span: create_empty_span(),
                    expected: "number".to_string(),
                    actual: format!("{:?}", arg),
                });
            }
        }
    }
    
    Ok(RuntimeValue::Number(sum))
}

/// 减法函数实现
fn subtract_function(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.is_empty() {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: create_empty_span(),
            expected: "at least 1".to_string(),
            actual: 0,
        });
    }
    
    // 获取第一个参数作为初始值
    let first = match &args[0] {
        RuntimeValue::Number(n) => *n,
        _ => {
            return Err(EvaluateError::TypeError {
                span: create_empty_span(),
                expected: "number".to_string(),
                actual: format!("{:?}", args[0]),
            });
        }
    };
    
    if args.len() == 1 {
        // 单参数情况：返回负数
        Ok(RuntimeValue::Number(-first))
    } else {
        // 多参数情况：从第一个数减去后面所有数
        let mut result = first;
        for arg in &args[1..] {
            match arg {
                RuntimeValue::Number(n) => {
                    result -= n;
                },
                _ => {
                    return Err(EvaluateError::TypeError {
                        span: create_empty_span(),
                        expected: "number".to_string(),
                        actual: format!("{:?}", arg),
                    });
                }
            }
        }
        Ok(RuntimeValue::Number(result))
    }
}

/// 乘法函数实现
fn multiply_function(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    let mut product = 1.0;
    
    for arg in args {
        match arg {
            RuntimeValue::Number(n) => {
                product *= n;
            },
            _ => {
                return Err(EvaluateError::TypeError {
                    span: create_empty_span(),
                    expected: "number".to_string(),
                    actual: format!("{:?}", arg),
                });
            }
        }
    }
    
    Ok(RuntimeValue::Number(product))
}

/// 除法函数实现
fn divide_function(args: &[RuntimeValue]) -> Result<RuntimeValue, EvaluateError> {
    if args.is_empty() {
        return Err(EvaluateError::ArgumentCountMismatch {
            span: create_empty_span(),
            expected: "at least 1".to_string(),
            actual: 0,
        });
    }
    
    // 获取第一个参数作为初始值
    let first = match &args[0] {
        RuntimeValue::Number(n) => *n,
        _ => {
            return Err(EvaluateError::TypeError {
                span: create_empty_span(),
                expected: "number".to_string(),
                actual: format!("{:?}", args[0]),
            });
        }
    };
    
    if args.len() == 1 {
        // 单参数情况：返回倒数
        if first == 0.0 {
            return Err(EvaluateError::DivisionByZero {
                span: create_empty_span(),
            });
        }
        Ok(RuntimeValue::Number(1.0 / first))
    } else {
        // 多参数情况：第一个数除以后面所有数
        let mut result = first;
        for arg in &args[1..] {
            match arg {
                RuntimeValue::Number(n) => {
                    if *n == 0.0 {
                        return Err(EvaluateError::DivisionByZero {
                            span: create_empty_span(),
                        });
                    }
                    result /= n;
                },
                _ => {
                    return Err(EvaluateError::TypeError {
                        span: create_empty_span(),
                        expected: "number".to_string(),
                        actual: format!("{:?}", arg),
                    });
                }
            }
        }
        Ok(RuntimeValue::Number(result))
    }
}

/// 创建空的 span（用于内置函数的错误报告）
fn create_empty_span() -> Rc<crate::interpreter::lexer::types::Span> {
    Rc::new(crate::interpreter::lexer::types::Span::empty(0))
}
