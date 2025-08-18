//! 算术函数模块
//! 
//! 实现基本的算术运算函数，支持四则运算

use std::rc::Rc;
use gc::Gc;

use crate::interpreter::{SExpr, SExprContent, Value};
use super::super::types::*;

// ============================================================================
// 算术函数实现
// ============================================================================

/// 加法函数 (+)
pub fn add(args: &[RuntimeObject]) -> Result<RuntimeObject, EvaluateError> {
    if args.is_empty() {
        return Ok(RuntimeObject {
            core: RuntimeObjectCore::Integer(0),
            source: None,
        });
    }
    
    let mut result = 0.0;
    for arg in args {
        match &arg.core {
            RuntimeObjectCore::Integer(n) => result += *n as f64,
            RuntimeObjectCore::Float(n) => result += *n,
            _ => {
                return Err(EvaluateError::TypeMismatch {
                    expr: arg.source.clone().unwrap_or_else(|| {
                        Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                    }),
                    expected: "number".to_string(),
                    actual: format!("{:?}", arg.core),
                });
            }
        }
    }
    
    if result.fract() == 0.0 {
        Ok(RuntimeObject {
            core: RuntimeObjectCore::Integer(result as i64),
            source: None,
        })
    } else {
        Ok(RuntimeObject {
            core: RuntimeObjectCore::Float(result),
            source: None,
        })
    }
}

/// 减法函数 (-)
pub fn subtract(args: &[RuntimeObject]) -> Result<RuntimeObject, EvaluateError> {
    if args.is_empty() {
        return Err(EvaluateError::ArgumentCountMismatch {
            expr: Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("-".to_string())))),
            expected: "at least 1".to_string(),
            actual: 0,
        });
    }
    
    if args.len() == 1 {
        match &args[0].core {
            RuntimeObjectCore::Integer(n) => Ok(RuntimeObject {
                core: RuntimeObjectCore::Integer(-*n),
                source: None,
            }),
            RuntimeObjectCore::Float(n) => Ok(RuntimeObject {
                core: RuntimeObjectCore::Float(-*n),
                source: None,
            }),
            _ => Err(EvaluateError::TypeMismatch {
                expr: args[0].source.clone().unwrap_or_else(|| {
                    Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                }),
                expected: "number".to_string(),
                actual: format!("{:?}", args[0].core),
            }),
        }
    } else {
        let mut result = match &args[0].core {
            RuntimeObjectCore::Integer(n) => *n as f64,
            RuntimeObjectCore::Float(n) => *n,
            _ => {
                return Err(EvaluateError::TypeMismatch {
                    expr: args[0].source.clone().unwrap_or_else(|| {
                        Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                    }),
                    expected: "number".to_string(),
                    actual: format!("{:?}", args[0].core),
                });
            }
        };
        
        for arg in &args[1..] {
            match &arg.core {
                RuntimeObjectCore::Integer(n) => result -= *n as f64,
                RuntimeObjectCore::Float(n) => result -= *n,
                _ => {
                    return Err(EvaluateError::TypeMismatch {
                        expr: arg.source.clone().unwrap_or_else(|| {
                            Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                        }),
                        expected: "number".to_string(),
                        actual: format!("{:?}", arg.core),
                    });
                }
            }
        }
        
        if result.fract() == 0.0 {
            Ok(RuntimeObject {
                core: RuntimeObjectCore::Integer(result as i64),
                source: None,
            })
        } else {
            Ok(RuntimeObject {
                core: RuntimeObjectCore::Float(result),
                source: None,
            })
        }
    }
}

/// 乘法函数 (*)
pub fn multiply(args: &[RuntimeObject]) -> Result<RuntimeObject, EvaluateError> {
    if args.is_empty() {
        return Ok(RuntimeObject {
            core: RuntimeObjectCore::Integer(1),
            source: None,
        });
    }
    
    let mut result = 1.0;
    for arg in args {
        match &arg.core {
            RuntimeObjectCore::Integer(n) => result *= *n as f64,
            RuntimeObjectCore::Float(n) => result *= *n,
            _ => {
                return Err(EvaluateError::TypeMismatch {
                    expr: arg.source.clone().unwrap_or_else(|| {
                        Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                    }),
                    expected: "number".to_string(),
                    actual: format!("{:?}", arg.core),
                });
            }
        }
    }
    
    if result.fract() == 0.0 {
        Ok(RuntimeObject {
            core: RuntimeObjectCore::Integer(result as i64),
            source: None,
        })
    } else {
        Ok(RuntimeObject {
            core: RuntimeObjectCore::Float(result),
            source: None,
        })
    }
}

/// 除法函数 (/)
pub fn divide(args: &[RuntimeObject]) -> Result<RuntimeObject, EvaluateError> {
    if args.is_empty() {
        return Err(EvaluateError::ArgumentCountMismatch {
            expr: Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("/".to_string())))),
            expected: "at least 1".to_string(),
            actual: 0,
        });
    }
    
    if args.len() == 1 {
        match &args[0].core {
            RuntimeObjectCore::Integer(n) => {
                if *n == 0 {
                    return Err(EvaluateError::DivisionByZero {
                        expr: args[0].source.clone().unwrap_or_else(|| {
                            Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("0".to_string()))))
                        }),
                    });
                }
                Ok(RuntimeObject {
                    core: RuntimeObjectCore::Float(1.0 / (*n as f64)),
                    source: None,
                })
            },
            RuntimeObjectCore::Float(n) => {
                if *n == 0.0 {
                    return Err(EvaluateError::DivisionByZero {
                        expr: args[0].source.clone().unwrap_or_else(|| {
                            Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("0.0".to_string()))))
                        }),
                    });
                }
                Ok(RuntimeObject {
                    core: RuntimeObjectCore::Float(1.0 / *n),
                    source: None,
                })
            },
            _ => Err(EvaluateError::TypeMismatch {
                expr: args[0].source.clone().unwrap_or_else(|| {
                    Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                }),
                expected: "number".to_string(),
                actual: format!("{:?}", args[0].core),
            }),
        }
    } else {
        let mut result = match &args[0].core {
            RuntimeObjectCore::Integer(n) => *n as f64,
            RuntimeObjectCore::Float(n) => *n,
            _ => {
                return Err(EvaluateError::TypeMismatch {
                    expr: args[0].source.clone().unwrap_or_else(|| {
                        Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                    }),
                    expected: "number".to_string(),
                    actual: format!("{:?}", args[0].core),
                });
            }
        };
        
        for arg in &args[1..] {
            match &arg.core {
                RuntimeObjectCore::Integer(n) => {
                    if *n == 0 {
                        return Err(EvaluateError::DivisionByZero {
                            expr: arg.source.clone().unwrap_or_else(|| {
                                Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("0".to_string()))))
                            }),
                        });
                    }
                    result /= *n as f64;
                },
                RuntimeObjectCore::Float(n) => {
                    if *n == 0.0 {
                        return Err(EvaluateError::DivisionByZero {
                            expr: arg.source.clone().unwrap_or_else(|| {
                                Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("0.0".to_string()))))
                            }),
                        });
                    }
                    result /= *n;
                },
                _ => {
                    return Err(EvaluateError::TypeMismatch {
                        expr: arg.source.clone().unwrap_or_else(|| {
                            Rc::new(SExpr::without_span(SExprContent::Atom(Value::Symbol("number".to_string()))))
                        }),
                        expected: "number".to_string(),
                        actual: format!("{:?}", arg.core),
                    });
                }
            }
        }
        
        if result.fract() == 0.0 {
            Ok(RuntimeObject {
                core: RuntimeObjectCore::Integer(result as i64),
                source: None,
            })
        } else {
            Ok(RuntimeObject {
                core: RuntimeObjectCore::Float(result),
                source: None,
            })
        }
    }
}

// ============================================================================
// 内置函数创建
// ============================================================================

/// 创建加法内置函数
pub fn create_add_function() -> BuiltinFunction {
    BuiltinFunction::new("+".to_string(), FunctionArity::AtLeast(0), add)
}

/// 创建减法内置函数
pub fn create_subtract_function() -> BuiltinFunction {
    BuiltinFunction::new("-".to_string(), FunctionArity::AtLeast(1), subtract)
}

/// 创建乘法内置函数
pub fn create_multiply_function() -> BuiltinFunction {
    BuiltinFunction::new("*".to_string(), FunctionArity::AtLeast(0), multiply)
}

/// 创建除法内置函数
pub fn create_divide_function() -> BuiltinFunction {
    BuiltinFunction::new("/".to_string(), FunctionArity::AtLeast(1), divide)
}
