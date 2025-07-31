use crate::legacy::types::{Value, SchemeError, Result};

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
            _ => return Err(SchemeError::TypeError(format!("+ expects numbers, got {arg}"), None)),
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
        return Err(SchemeError::ArityError("- requires at least 1 argument".to_string(), None));
    }

    if args.len() == 1 {
        // 一元减法（取负数）
        match &args[0] {
            Value::Integer(n) => return Ok(Value::Integer(-n)),
            Value::Float(f) => return Ok(Value::Float(-f)),
            _ => return Err(SchemeError::TypeError(format!("- expects numbers, got {}", args[0]), None)),
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
        _ => return Err(SchemeError::TypeError(format!("- expects numbers, got {}", args[0]), None)),
    };

    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => result -= *n as f64,
            Value::Float(f) => {
                is_float = true;
                result -= f;
            },
            _ => return Err(SchemeError::TypeError(format!("- expects numbers, got {arg}"), None)),
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
            _ => return Err(SchemeError::TypeError(format!("* expects numbers, got {arg}"), None)),
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
        return Err(SchemeError::ArityError("/ requires at least 1 argument".to_string(), None));
    }

    if args.len() == 1 {
        // 一元除法（求倒数）
        match &args[0] {
            Value::Integer(n) => {
                if *n == 0 {
                    return Err(SchemeError::DivisionByZero(None));
                }
                return Ok(Value::Float(1.0 / (*n as f64)));
            },
            Value::Float(f) => {
                if *f == 0.0 {
                    return Err(SchemeError::DivisionByZero(None));
                }
                return Ok(Value::Float(1.0 / f));
            },
            _ => return Err(SchemeError::TypeError(format!("/ expects numbers, got {}", args[0]), None)),
        }
    }

    // 二元及多元除法
    let mut result = match &args[0] {
        Value::Integer(n) => *n as f64,
        Value::Float(f) => *f,
        _ => return Err(SchemeError::TypeError(format!("/ expects numbers, got {}", args[0]), None)),
    };

    for arg in &args[1..] {
        match arg {
            Value::Integer(n) => {
                if *n == 0 {
                    return Err(SchemeError::DivisionByZero(None));
                }
                result /= *n as f64;
            },
            Value::Float(f) => {
                if *f == 0.0 {
                    return Err(SchemeError::DivisionByZero(None));
                }
                result /= f;
            },
            _ => return Err(SchemeError::TypeError(format!("/ expects numbers, got {arg}"), None)),
        }
    }

    Ok(Value::Float(result))
}

/// 比较运算函数
pub fn equal(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("= requires exactly 2 arguments".to_string(), None));
    }

    let result = match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Integer(a), Value::Float(b)) => (*a as f64) == *b,
        (Value::Float(a), Value::Integer(b)) => *a == (*b as f64),
        _ => return Err(SchemeError::TypeError("= expects numbers".to_string(), None)),
    };

    Ok(Value::Bool(result))
}

pub fn less_than(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("< requires exactly 2 arguments".to_string(), None));
    }

    let result = match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => a < b,
        (Value::Float(a), Value::Float(b)) => a < b,
        (Value::Integer(a), Value::Float(b)) => (*a as f64) < *b,
        (Value::Float(a), Value::Integer(b)) => *a < (*b as f64),
        _ => return Err(SchemeError::TypeError("< expects numbers".to_string(), None)),
    };

    Ok(Value::Bool(result))
}

pub fn greater_than(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("> requires exactly 2 arguments".to_string(), None));
    }

    let result = match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => a > b,
        (Value::Float(a), Value::Float(b)) => a > b,
        (Value::Integer(a), Value::Float(b)) => (*a as f64) > *b,
        (Value::Float(a), Value::Integer(b)) => *a > (*b as f64),
        _ => return Err(SchemeError::TypeError("> expects numbers".to_string(), None)),
    };

    Ok(Value::Bool(result))
}

pub fn less_equal(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("<= requires exactly 2 arguments".to_string(), None));
    }

    let result = match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => a <= b,
        (Value::Float(a), Value::Float(b)) => a <= b,
        (Value::Integer(a), Value::Float(b)) => (*a as f64) <= *b,
        (Value::Float(a), Value::Integer(b)) => *a <= (*b as f64),
        _ => return Err(SchemeError::TypeError("<= expects numbers".to_string(), None)),
    };

    Ok(Value::Bool(result))
}

pub fn greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError(">= requires exactly 2 arguments".to_string(), None));
    }

    let result = match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => a >= b,
        (Value::Float(a), Value::Float(b)) => a >= b,
        (Value::Integer(a), Value::Float(b)) => (*a as f64) >= *b,
        (Value::Float(a), Value::Integer(b)) => *a >= (*b as f64),
        _ => return Err(SchemeError::TypeError(">= expects numbers".to_string(), None)),
    };

    Ok(Value::Bool(result))
}

/// 数学函数
pub fn abs_func(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("abs requires exactly 1 argument".to_string(), None));
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(SchemeError::TypeError(format!("abs expects a number, got {}", args[0]), None)),
    }
}

pub fn max_func(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(SchemeError::ArityError("max requires at least 1 argument".to_string(), None));
    }

    let mut max_val = &args[0];
    let mut is_float = false;

    // 检查是否有浮点数
    for arg in args {
        match arg {
            Value::Float(_) => is_float = true,
            Value::Integer(_) => {},
            _ => return Err(SchemeError::TypeError(format!("max expects numbers, got {}", arg), None)),
        }
    }

    for arg in &args[1..] {
        let greater = match (max_val, arg) {
            (Value::Integer(a), Value::Integer(b)) => b > a,
            (Value::Float(a), Value::Float(b)) => b > a,
            (Value::Integer(a), Value::Float(b)) => b > &(*a as f64),
            (Value::Float(a), Value::Integer(b)) => (*b as f64) > *a,
            _ => return Err(SchemeError::TypeError("max expects numbers".to_string(), None)),
        };
        
        if greater {
            max_val = arg;
        }
    }

    if is_float {
        match max_val {
            Value::Integer(n) => Ok(Value::Float(*n as f64)),
            Value::Float(f) => Ok(Value::Float(*f)),
            _ => unreachable!(),
        }
    } else {
        Ok(max_val.clone())
    }
}

pub fn min_func(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(SchemeError::ArityError("min requires at least 1 argument".to_string(), None));
    }

    let mut min_val = &args[0];
    let mut is_float = false;

    // 检查是否有浮点数
    for arg in args {
        match arg {
            Value::Float(_) => is_float = true,
            Value::Integer(_) => {},
            _ => return Err(SchemeError::TypeError(format!("min expects numbers, got {}", arg), None)),
        }
    }

    for arg in &args[1..] {
        let less = match (min_val, arg) {
            (Value::Integer(a), Value::Integer(b)) => b < a,
            (Value::Float(a), Value::Float(b)) => b < a,
            (Value::Integer(a), Value::Float(b)) => b < &(*a as f64),
            (Value::Float(a), Value::Integer(b)) => (*b as f64) < *a,
            _ => return Err(SchemeError::TypeError("min expects numbers".to_string(), None)),
        };
        
        if less {
            min_val = arg;
        }
    }

    if is_float {
        match min_val {
            Value::Integer(n) => Ok(Value::Float(*n as f64)),
            Value::Float(f) => Ok(Value::Float(*f)),
            _ => unreachable!(),
        }
    } else {
        Ok(min_val.clone())
    }
}

/// 列表操作函数
pub fn cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(SchemeError::ArityError("cons requires exactly 2 arguments".to_string(), None));
    }

    Ok(Value::Cons(
        std::rc::Rc::new(args[0].clone()),
        std::rc::Rc::new(args[1].clone())
    ))
}

pub fn car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("car requires exactly 1 argument".to_string(), None));
    }

    match &args[0] {
        Value::Cons(car_val, _) => Ok((**car_val).clone()),
        Value::Nil => Err(SchemeError::RuntimeError("car of empty list".to_string(), None)),
        _ => Err(SchemeError::TypeError(format!("car expects a pair, got {}", args[0]), None)),
    }
}

pub fn cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("cdr requires exactly 1 argument".to_string(), None));
    }

    match &args[0] {
        Value::Cons(_, cdr_val) => Ok((**cdr_val).clone()),
        Value::Nil => Err(SchemeError::RuntimeError("cdr of empty list".to_string(), None)),
        _ => Err(SchemeError::TypeError(format!("cdr expects a pair, got {}", args[0]), None)),
    }
}

pub fn list(args: &[Value]) -> Result<Value> {
    Ok(Value::from_vec(args.to_vec()))
}

/// 类型谓词函数
pub fn is_null(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("null? requires exactly 1 argument".to_string(), None));
    }

    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

pub fn is_pair(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("pair? requires exactly 1 argument".to_string(), None));
    }

    Ok(Value::Bool(matches!(args[0], Value::Cons(_, _))))
}

pub fn is_number(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("number? requires exactly 1 argument".to_string(), None));
    }

    Ok(Value::Bool(matches!(args[0], Value::Integer(_) | Value::Float(_))))
}

pub fn is_symbol(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("symbol? requires exactly 1 argument".to_string(), None));
    }

    Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
}

pub fn is_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(SchemeError::ArityError("string? requires exactly 1 argument".to_string(), None));
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

    #[test]
    fn test_comparison_functions() {
        // 测试 >
        assert_eq!(greater_than(&[Value::Integer(5), Value::Integer(3)]).unwrap(), Value::Bool(true));
        assert_eq!(greater_than(&[Value::Integer(3), Value::Integer(5)]).unwrap(), Value::Bool(false));
        assert_eq!(greater_than(&[Value::Float(5.5), Value::Integer(5)]).unwrap(), Value::Bool(true));
        
        // 测试 <=
        assert_eq!(less_equal(&[Value::Integer(3), Value::Integer(5)]).unwrap(), Value::Bool(true));
        assert_eq!(less_equal(&[Value::Integer(5), Value::Integer(5)]).unwrap(), Value::Bool(true));
        assert_eq!(less_equal(&[Value::Integer(7), Value::Integer(5)]).unwrap(), Value::Bool(false));
        
        // 测试 >=
        assert_eq!(greater_equal(&[Value::Integer(5), Value::Integer(3)]).unwrap(), Value::Bool(true));
        assert_eq!(greater_equal(&[Value::Integer(5), Value::Integer(5)]).unwrap(), Value::Bool(true));
        assert_eq!(greater_equal(&[Value::Integer(3), Value::Integer(5)]).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_math_builtin_functions() {
        // 测试 abs
        assert_eq!(abs_func(&[Value::Integer(-5)]).unwrap(), Value::Integer(5));
        assert_eq!(abs_func(&[Value::Integer(3)]).unwrap(), Value::Integer(3));
        assert_eq!(abs_func(&[Value::Float(-3.14)]).unwrap(), Value::Float(3.14));
        
        // 测试 max
        assert_eq!(max_func(&[Value::Integer(1), Value::Integer(2), Value::Integer(3)]).unwrap(), Value::Integer(3));
        assert_eq!(max_func(&[Value::Float(1.5), Value::Integer(2)]).unwrap(), Value::Float(2.0));
        
        // 测试 min
        assert_eq!(min_func(&[Value::Integer(3), Value::Integer(1), Value::Integer(2)]).unwrap(), Value::Integer(1));
        assert_eq!(min_func(&[Value::Float(1.5), Value::Integer(2)]).unwrap(), Value::Float(1.5));
    }
}
