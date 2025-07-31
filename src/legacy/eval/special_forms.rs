use std::rc::Rc;
use crate::legacy::types::{Value, SchemeError, Result};
use crate::legacy::env::Environment;

/// 特殊形式求值器
pub struct SpecialFormsEvaluator;

impl SpecialFormsEvaluator {
    /// 求值 quote 特殊形式
    pub fn eval_quote(args: &[Value], _env: &Environment) -> Result<Value> {
        if args.len() != 1 {
            return Err(SchemeError::ArityError("quote requires exactly 1 argument".to_string(), None));
        }
        Ok(args[0].clone())
    }

    /// 求值 if 特殊形式
    pub fn eval_if(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            return Err(SchemeError::ArityError("if requires 2 or 3 arguments".to_string(), None));
        }

        let condition = eval_fn(&args[0], env)?;
        
        if condition.is_truthy() {
            eval_fn(&args[1], env)
        } else if args.len() == 3 {
            eval_fn(&args[2], env)
        } else {
            Ok(Value::Nil)
        }
    }

    /// 求值 define 特殊形式
    pub fn eval_define(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("define requires exactly 2 arguments".to_string(), None));
        }

        match &args[0] {
            // 变量定义: (define var value)
            Value::Symbol(name) => {
                let value = eval_fn(&args[1], env)?;
                env.define(name.clone(), value)?;
                Ok(Value::Nil)
            },
            // 函数定义: (define (func-name param1 param2 ...) body)
            Value::Cons(_, _) => {
                if let Some(func_def) = args[0].to_vec() {
                    if func_def.is_empty() {
                        return Err(SchemeError::TypeError("Empty function definition".to_string(), None));
                    }
                    
                    // 第一个元素应该是函数名
                    if let Value::Symbol(func_name) = &func_def[0] {
                        // 剩余的元素是参数列表
                        let mut params = Vec::new();
                        for param in &func_def[1..] {
                            if let Value::Symbol(param_name) = param {
                                params.push(param_name.clone());
                            } else {
                                return Err(SchemeError::TypeError("Function parameters must be symbols".to_string(), None));
                            }
                        }
                        
                        // 创建 lambda 并绑定到函数名
                        let lambda = Value::Lambda {
                            params,
                            body: Rc::new(args[1].clone()),
                            env_id: env.id(),
                        };
                        
                        env.define(func_name.clone(), lambda)?;
                        Ok(Value::Nil)
                    } else {
                        Err(SchemeError::TypeError("Function name must be a symbol".to_string(), None))
                    }
                } else {
                    Err(SchemeError::TypeError("Invalid function definition".to_string(), None))
                }
            },
            _ => Err(SchemeError::TypeError("define expects a symbol or function definition".to_string(), None)),
        }
    }

    /// 求值 set! 特殊形式
    pub fn eval_set(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("set! requires exactly 2 arguments".to_string(), None));
        }

        match &args[0] {
            Value::Symbol(name) => {
                let value = eval_fn(&args[1], env)?;
                env.set(name, value)?;
                Ok(Value::Nil)
            },
            _ => Err(SchemeError::TypeError("set! expects a symbol".to_string(), None)),
        }
    }

    /// 求值 lambda 特殊形式
    pub fn eval_lambda(args: &[Value], env: &Environment) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("lambda requires exactly 2 arguments".to_string(), None));
        }

        // 解析参数列表
        let params = match &args[0] {
            Value::Nil => Vec::new(),
            expr => {
                if let Some(param_list) = expr.to_vec() {
                    let mut params = Vec::new();
                    for param in param_list {
                        if let Value::Symbol(name) = param {
                            params.push(name);
                        } else {
                            return Err(SchemeError::TypeError("lambda parameters must be symbols".to_string(), None));
                        }
                    }
                    params
                } else {
                    return Err(SchemeError::TypeError("lambda parameters must be a list".to_string(), None));
                }
            }
        };

        Ok(Value::Lambda {
            params,
            body: Rc::new(args[1].clone()),
            env_id: env.id(),
        })
    }

    /// 求值 let 特殊形式
    pub fn eval_let(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("let requires exactly 2 arguments".to_string(), None));
        }

        // 解析绑定列表
        let bindings = match &args[0] {
            Value::Nil => Vec::new(),
            expr => {
                if let Some(binding_list) = expr.to_vec() {
                    let mut bindings = Vec::new();
                    for binding in binding_list {
                        if let Some(pair) = binding.to_vec() {
                            if pair.len() == 2 {
                                if let Value::Symbol(name) = &pair[0] {
                                    let value = eval_fn(&pair[1], env)?;
                                    bindings.push((name.clone(), value));
                                } else {
                                    return Err(SchemeError::TypeError("let binding name must be a symbol".to_string(), None));
                                }
                            } else {
                                return Err(SchemeError::TypeError("let binding must have exactly 2 elements".to_string(), None));
                            }
                        } else {
                            return Err(SchemeError::TypeError("let binding must be a list".to_string(), None));
                        }
                    }
                    bindings
                } else {
                    return Err(SchemeError::TypeError("let bindings must be a list".to_string(), None));
                }
            }
        };

        // 创建新环境
        let names: Vec<String> = bindings.iter().map(|(name, _)| name.clone()).collect();
        let values: Vec<Value> = bindings.into_iter().map(|(_, value)| value).collect();
        let new_env = env.extend(names, values)?;

        // 在新环境中求值 body
        eval_fn(&args[1], &new_env)
    }

    /// 求值 begin 特殊形式
    pub fn eval_begin(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Nil);
        }

        let mut result = Value::Nil;
        for expr in args {
            result = eval_fn(expr, env)?;
        }
        Ok(result)
    }

    /// 求值 and 特殊形式
    pub fn eval_and(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Bool(true));
        }

        for arg in args {
            let result = eval_fn(arg, env)?;
            if !result.is_truthy() {
                return Ok(result);
            }
        }
        
        // 如果所有表达式都为真，返回最后一个表达式的值
        eval_fn(&args[args.len() - 1], env)
    }

    /// 求值 or 特殊形式
    pub fn eval_or(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Bool(false));
        }

        for arg in args {
            let result = eval_fn(arg, env)?;
            if result.is_truthy() {
                return Ok(result);
            }
        }
        
        // 如果所有表达式都为假，返回最后一个表达式的值
        eval_fn(&args[args.len() - 1], env)
    }

    /// 求值 cond 特殊形式
    pub fn eval_cond(args: &[Value], env: &Environment, eval_fn: &dyn Fn(&Value, &Environment) -> Result<Value>) -> Result<Value> {
        for clause in args {
            if let Some(clause_list) = clause.to_vec() {
                if clause_list.len() < 1 {
                    return Err(SchemeError::SyntaxError("cond clause must have at least a condition".to_string(), None));
                }
                
                // 检查是否为 else 子句
                if let Value::Symbol(s) = &clause_list[0] {
                    if s == "else" {
                        if clause_list.len() == 1 {
                            return Ok(Value::Nil);
                        } else if clause_list.len() == 2 {
                            return eval_fn(&clause_list[1], env);
                        } else {
                            // 多个表达式，当作 begin 处理
                            return Self::eval_begin(&clause_list[1..], env, eval_fn);
                        }
                    }
                }
                
                // 求值条件
                let condition = eval_fn(&clause_list[0], env)?;
                
                if condition.is_truthy() {
                    if clause_list.len() == 1 {
                        return Ok(condition);
                    } else if clause_list.len() == 2 {
                        return eval_fn(&clause_list[1], env);
                    } else {
                        // 多个表达式，当作 begin 处理
                        return Self::eval_begin(&clause_list[1..], env, eval_fn);
                    }
                }
            } else {
                return Err(SchemeError::SyntaxError("cond clause must be a list".to_string(), None));
            }
        }
        
        // 没有匹配的子句
        Ok(Value::Nil)
    }
} 