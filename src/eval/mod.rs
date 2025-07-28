use std::rc::Rc;
use crate::types::{Value, SchemeError, Result};
use crate::env::Environment;
use crate::builtins;

/// 求值器
pub struct Evaluator {
    /// 全局环境
    pub global_env: Rc<Environment>,
}

impl Evaluator {
    /// 创建新的求值器
    pub fn new() -> Self {
        let mut global_env = Environment::new();
        
        // 注册内置函数
        Self::register_builtins(&mut global_env);
        
        Evaluator {
            global_env: Rc::new(global_env),
        }
    }

    /// 注册内置函数
    fn register_builtins(env: &mut Environment) {
        // 算术运算
        env.define("+".to_string(), Value::BuiltinFunction {
            name: "+".to_string(),
            func: builtins::add,
            arity: None, // 可变参数
        });
        
        env.define("-".to_string(), Value::BuiltinFunction {
            name: "-".to_string(),
            func: builtins::subtract,
            arity: None,
        });
        
        env.define("*".to_string(), Value::BuiltinFunction {
            name: "*".to_string(),
            func: builtins::multiply,
            arity: None,
        });
        
        env.define("/".to_string(), Value::BuiltinFunction {
            name: "/".to_string(),
            func: builtins::divide,
            arity: None,
        });

        // 比较运算
        env.define("=".to_string(), Value::BuiltinFunction {
            name: "=".to_string(),
            func: builtins::equal,
            arity: Some(2),
        });
        
        env.define("<".to_string(), Value::BuiltinFunction {
            name: "<".to_string(),
            func: builtins::less_than,
            arity: Some(2),
        });

        // 列表操作
        env.define("cons".to_string(), Value::BuiltinFunction {
            name: "cons".to_string(),
            func: builtins::cons,
            arity: Some(2),
        });
        
        env.define("car".to_string(), Value::BuiltinFunction {
            name: "car".to_string(),
            func: builtins::car,
            arity: Some(1),
        });
        
        env.define("cdr".to_string(), Value::BuiltinFunction {
            name: "cdr".to_string(),
            func: builtins::cdr,
            arity: Some(1),
        });
        
        env.define("list".to_string(), Value::BuiltinFunction {
            name: "list".to_string(),
            func: builtins::list,
            arity: None,
        });

        // 类型谓词
        env.define("null?".to_string(), Value::BuiltinFunction {
            name: "null?".to_string(),
            func: builtins::is_null,
            arity: Some(1),
        });
        
        env.define("pair?".to_string(), Value::BuiltinFunction {
            name: "pair?".to_string(),
            func: builtins::is_pair,
            arity: Some(1),
        });
        
        env.define("number?".to_string(), Value::BuiltinFunction {
            name: "number?".to_string(),
            func: builtins::is_number,
            arity: Some(1),
        });
        
        env.define("symbol?".to_string(), Value::BuiltinFunction {
            name: "symbol?".to_string(),
            func: builtins::is_symbol,
            arity: Some(1),
        });
        
        env.define("string?".to_string(), Value::BuiltinFunction {
            name: "string?".to_string(),
            func: builtins::is_string,
            arity: Some(1),
        });
    }

    /// 求值表达式
    pub fn eval(&self, expr: &Value, env: Rc<Environment>) -> Result<Value> {
        match expr {
            // 自求值表达式
            Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Bool(_) => {
                Ok(expr.clone())
            },
            
            // 空列表
            Value::Nil => Ok(Value::Nil),
            
            // 符号（变量查找）
            Value::Symbol(name) => env.lookup(name),
            
            // 列表（函数调用或特殊形式）
            Value::Cons(_, _) => {
                if let Some(list) = expr.to_vec() {
                    if list.is_empty() {
                        return Ok(Value::Nil);
                    }
                    
                    // 检查是否为特殊形式
                    if let Value::Symbol(op) = &list[0] {
                        match op.as_str() {
                            "quote" => self.eval_quote(&list[1..], env),
                            "if" => self.eval_if(&list[1..], env),
                            "define" => self.eval_define(&list[1..], env),
                            "lambda" => self.eval_lambda(&list[1..], env),
                            "let" => self.eval_let(&list[1..], env),
                            "begin" => self.eval_begin(&list[1..], env),
                            _ => self.eval_application(&list, env),
                        }
                    } else {
                        self.eval_application(&list, env)
                    }
                } else {
                    Err(SchemeError::RuntimeError("Invalid list structure".to_string()))
                }
            },
            
            _ => Err(SchemeError::RuntimeError(format!("Cannot evaluate {}", expr))),
        }
    }

    /// 求值 quote 特殊形式
    fn eval_quote(&self, args: &[Value], _env: Rc<Environment>) -> Result<Value> {
        if args.len() != 1 {
            return Err(SchemeError::ArityError("quote requires exactly 1 argument".to_string()));
        }
        Ok(args[0].clone())
    }

    /// 求值 if 特殊形式
    fn eval_if(&self, args: &[Value], env: Rc<Environment>) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            return Err(SchemeError::ArityError("if requires 2 or 3 arguments".to_string()));
        }

        let condition = self.eval(&args[0], env.clone())?;
        
        if condition.is_truthy() {
            self.eval(&args[1], env)
        } else if args.len() == 3 {
            self.eval(&args[2], env)
        } else {
            Ok(Value::Nil)
        }
    }

    /// 求值 define 特殊形式
    fn eval_define(&self, args: &[Value], env: Rc<Environment>) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("define requires exactly 2 arguments".to_string()));
        }

        match &args[0] {
            Value::Symbol(_name) => {
                let _value = self.eval(&args[1], env.clone())?;
                // 注意：这里需要修改环境，但 Rc 是不可变的
                // 在实际实现中需要使用 RefCell 或其他方案
                Err(SchemeError::RuntimeError(
                    "Cannot modify environment through Rc - need RefCell".to_string()
                ))
            },
            _ => Err(SchemeError::TypeError("define expects a symbol".to_string())),
        }
    }

    /// 求值 lambda 特殊形式
    fn eval_lambda(&self, args: &[Value], env: Rc<Environment>) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("lambda requires exactly 2 arguments".to_string()));
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
                            return Err(SchemeError::TypeError(
                                "lambda parameters must be symbols".to_string()
                            ));
                        }
                    }
                    params
                } else {
                    return Err(SchemeError::TypeError(
                        "lambda parameters must be a list".to_string()
                    ));
                }
            }
        };

        Ok(Value::Lambda {
            params,
            body: Rc::new(args[1].clone()),
            env: env.clone(),
        })
    }

    /// 求值 let 特殊形式
    fn eval_let(&self, args: &[Value], env: Rc<Environment>) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("let requires exactly 2 arguments".to_string()));
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
                                    let value = self.eval(&pair[1], env.clone())?;
                                    bindings.push((name.clone(), value));
                                } else {
                                    return Err(SchemeError::TypeError(
                                        "let binding name must be a symbol".to_string()
                                    ));
                                }
                            } else {
                                return Err(SchemeError::TypeError(
                                    "let binding must have exactly 2 elements".to_string()
                                ));
                            }
                        } else {
                            return Err(SchemeError::TypeError(
                                "let binding must be a list".to_string()
                            ));
                        }
                    }
                    bindings
                } else {
                    return Err(SchemeError::TypeError(
                        "let bindings must be a list".to_string()
                    ));
                }
            }
        };

        // 创建新环境
        let names: Vec<String> = bindings.iter().map(|(name, _)| name.clone()).collect();
        let values: Vec<Value> = bindings.into_iter().map(|(_, value)| value).collect();
        let new_env = env.extend(names, values)?;

        // 在新环境中求值 body
        self.eval(&args[1], Rc::new(new_env))
    }

    /// 求值 begin 特殊形式
    fn eval_begin(&self, args: &[Value], env: Rc<Environment>) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Nil);
        }

        let mut result = Value::Nil;
        for expr in args {
            result = self.eval(expr, env.clone())?;
        }
        Ok(result)
    }

    /// 求值函数应用
    fn eval_application(&self, exprs: &[Value], env: Rc<Environment>) -> Result<Value> {
        if exprs.is_empty() {
            return Ok(Value::Nil);
        }

        // 求值函数
        let func = self.eval(&exprs[0], env.clone())?;
        
        // 求值参数
        let mut args = Vec::new();
        for arg_expr in &exprs[1..] {
            args.push(self.eval(arg_expr, env.clone())?);
        }

        // 应用函数
        match func {
            Value::BuiltinFunction { func, arity, .. } => {
                // 检查参数个数
                if let Some(expected_arity) = arity {
                    if args.len() != expected_arity {
                        return Err(SchemeError::ArityError(
                            format!("Expected {} arguments, got {}", expected_arity, args.len())
                        ));
                    }
                }
                func(&args)
            },
            
            Value::Lambda { params, body, env: closure_env } => {
                if args.len() != params.len() {
                    return Err(SchemeError::ArityError(
                        format!("Expected {} arguments, got {}", params.len(), args.len())
                    ));
                }
                
                // 创建新环境绑定参数
                let new_env = closure_env.extend(params, args)?;
                self.eval(&body, Rc::new(new_env))
            },
            
            _ => Err(SchemeError::TypeError(
                format!("Cannot apply non-function: {}", func)
            )),
        }
    }

    /// 便利方法：求值字符串
    pub fn eval_string(&self, input: &str) -> Result<Value> {
        let expr = crate::parser::Parser::parse(input)?;
        self.eval(&expr, self.global_env.clone())
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_atoms() {
        let evaluator = Evaluator::new();
        
        assert_eq!(evaluator.eval_string("42").unwrap(), Value::Integer(42));
        assert_eq!(evaluator.eval_string("3.14").unwrap(), Value::Float(3.14));
        assert_eq!(evaluator.eval_string("\"hello\"").unwrap(), Value::String("hello".to_string()));
        assert_eq!(evaluator.eval_string("#t").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_arithmetic() {
        let evaluator = Evaluator::new();
        
        assert_eq!(evaluator.eval_string("(+ 1 2 3)").unwrap(), Value::Integer(6));
        assert_eq!(evaluator.eval_string("(* 2 3 4)").unwrap(), Value::Integer(24));
        assert_eq!(evaluator.eval_string("(- 10 3)").unwrap(), Value::Integer(7));
    }

    #[test]
    fn test_eval_quote() {
        let evaluator = Evaluator::new();
        
        let result = evaluator.eval_string("'foo").unwrap();
        assert_eq!(result, Value::Symbol("foo".to_string()));
        
        let result = evaluator.eval_string("'(1 2 3)").unwrap();
        let expected = Value::from_vec(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_eval_if() {
        let evaluator = Evaluator::new();
        
        assert_eq!(evaluator.eval_string("(if #t 1 2)").unwrap(), Value::Integer(1));
        assert_eq!(evaluator.eval_string("(if #f 1 2)").unwrap(), Value::Integer(2));
        assert_eq!(evaluator.eval_string("(if #f 1)").unwrap(), Value::Nil);
    }
}
