use crate::types::{Value, SchemeError, Result};

/// 算术运算函数
pub fn add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Integer(0));
    }

    let mut result = 0i64;
    let mut is_float = false;
    let mut float_result = 0.0f64;

    for arg in args {
        match arg {
            Value::Integer(n) => {
                if is_float {
                    float_result += *n as f64;
                } else {
                    result += n;
                }
            },
            Value::Float(f) => {
                if !is_float {
                    is_float = true;
                    float_result = result as f64 + f;
                } else {
                    float_result += f;
                }
            },
            _ => return Err(SchemeError::TypeError(
                format!("+ expects numbers, got {}", arg)
            )),
        }
    }

    if is_float {
        Ok(Value::Float(float_result))
    } else {
        Ok(Value::Integer(result))
    }
}

pub fn subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(SchemeError::ArityError("- requires at least 1 argument".to_string()));
    }

    if args.len() == 1 {
        // 一元减法（取负数）
        match &args[0] {
            Value::Integer(n) => return Ok(Value::Integer(-n)),
            Value::Float(f) => return Ok(Value::Float(-f)),
            _ => return Err(SchemeError::TypeError(
                format!("- expects numbers, got {}", args[0])
            )),
        }
    }

    // 二元及多元减法
    let mut is_float = false;
    let mut result = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => {
            is_float = true;
            *f
        },
        _ => return Err(SchemeError::TypeError(
            format!("- expects numbers, got {}", args[0])
        )),
    };

    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => result -= *n as f64,
            Value::Float(f) => {
                is_float = true;
                result -= f;
            },
            _ => return Err(SchemeError::TypeError(
                format!("- expects numbers, got {}", arg)
            )),
        }
    }

    if is_float {
        Ok(Value::Float(result))
    } else {
        Ok(Value::Integer(result as i64))
    }
}

pub fn multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Integer(1));
    }

    let mut result = 1i64;
    let mut is_float = false;
    let mut float_result = 1.0f64;

    for arg in args {
        match arg {
            Value::Integer(n) => {
                if is_float {
                    float_result *= *n as f64;
                } else {
                    result *= n;
                }
            },
            Value::Float(f) => {
                if !is_float {
                    is_float = true;
                    float_result = result as f64 * f;
                } else {
                    float_result *= f;
                }
            },
            _ => return Err(SchemeError::TypeError(
                format!("* expects numbers, got {}", arg)
            )),
        }
    }

    if is_float {
        Ok(Value::Float(float_result))
    } else {
        Ok(Value::Integer(result))
    }
}

pub fn divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(SchemeError::ArityError("/ requires at least 1 argument".to_string()));
    }

    if args.len() == 1 {
        // 一元除法（求倒数）
        match &args[0] {
            Value::Integer(n) => {
                if *n == 0 {
                    return Err(SchemeError::DivisionByZero);
                }
                return Ok(Value::Float(1.0 / (*n as f64)));
            },
            Value::Float(f) => {
                if *f == 0.0 {
                    return Err(SchemeError::DivisionByZero);
                }
                return Ok(Value::Float(1.0 / f));
            },
            _ => return Err(SchemeError::TypeError(
                format!("/ expects numbers, got {}", args[0])
            )),
        }
    }

    // 二元及多元除法
    let mut result = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => return Err(SchemeError::TypeError(
            format!("/ expects numbers, got {}", args[0])
        )),
    };

    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if *n == 0 {
                    return Err(SchemeError::DivisionByZero);
                }
                result /= *n as f64;
            },
            Value::Float(f) => {
                if *f == 0.0 {
                    return Err(SchemeError::DivisionByZero);
                }
                result /= f;
            },
            _ => return Err(SchemeError::TypeError(
                format!("/ expects numbers, got {}", arg)
            )),
        }
    }

    Ok(Value::Float(result))
}

/// 比较运算函数
pub fn equal(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("= requires exactly 2 arguments".to_string()));
    }

    let result = match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Integer(a), Value::Float(b)) => (*a as f64) == *b,
        (Value::Float(a), Value::Integer(b)) => *a == (*b as f64),
        _ => return Err(SchemeError::TypeError(
            "= expects numbers".to_string()
        )),
    };

    Ok(Value::Bool(result))
}

pub fn less_than(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("< requires exactly 2 arguments".to_string()));
    }

    let result = match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => a < b,
        (Value::Float(a), Value::Float(b)) => a < b,
        (Value::Integer(a), Value::Float(b)) => (*a as f64) < *b,
        (Value::Float(a), Value::Integer(b)) => *a < (*b as f64),
        _ => return Err(SchemeError::TypeError(
            "< expects numbers".to_string()
        )),
    };

    Ok(Value::Bool(result))
}

/// 列表操作函数
pub fn cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("cons requires exactly 2 arguments".to_string()));
    }

    Ok(Value::Cons(
        std::rc::Rc::new(args[0].clone()),
        std::rc::Rc::new(args[1].clone())
    ))
}

pub fn car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("car requires exactly 1 argument".to_string()));
    }

    match &args[0] {
        Value::Cons(car_val, _) => Ok((**car_val).clone()),
        Value::Nil => Err(SchemeError::RuntimeError("car of empty list".to_string())),
        _ => Err(SchemeError::TypeError(
            format!("car expects a pair, got {}", args[0])
        )),
    }
}

pub fn cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("cdr requires exactly 1 argument".to_string()));
    }

    match &args[0] {
        Value::Cons(_, cdr_val) => Ok((**cdr_val).clone()),
        Value::Nil => Err(SchemeError::RuntimeError("cdr of empty list".to_string())),
        _ => Err(SchemeError::TypeError(
            format!("cdr expects a pair, got {}", args[0])
        )),
    }
}

pub fn list(args: &[Value]) -> Result<Value> {
    Ok(Value::from_vec(args.to_vec()))
}

/// 类型谓词函数
pub fn is_null(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("null? requires exactly 1 argument".to_string()));
    }

    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

pub fn is_pair(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("pair? requires exactly 1 argument".to_string()));
    }

    Ok(Value::Bool(matches!(args[0], Value::Cons(_, _))))
}

pub fn is_number(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("number? requires exactly 1 argument".to_string()));
    }

    Ok(Value::Bool(matches!(args[0], Value::Integer(_) | Value::Float(_))))
}

pub fn is_symbol(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("symbol? requires exactly 1 argument".to_string()));
    }

    Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
}

pub fn is_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("string? requires exactly 1 argument".to_string()));
    }

    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let args = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        assert_eq!(add(&args).unwrap(), Value::Integer(6));

        let args = vec![Value::Float(1.5), Value::Integer(2)];
        assert_eq!(add(&args).unwrap(), Value::Float(3.5));
    }

    #[test]
    fn test_cons_car_cdr() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let pair = cons(&args).unwrap();
        
        assert_eq!(car(&[pair.clone()]).unwrap(), Value::Integer(1));
        assert_eq!(cdr(&[pair]).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_list() {
        let args = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = list(&args).unwrap();
        
        assert!(result.is_list());
        assert_eq!(result.length().unwrap(), 3);
    }
}
