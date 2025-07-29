use std::rc::Rc;
use std::cell::RefCell;
use crate::types::{Value, SchemeError, Result};
use crate::env::{Environment, EnvironmentManager, EnvironmentId};
use crate::builtins;

/// 求值器
pub struct Evaluator {
    /// 环境管理器
    env_manager: Rc<RefCell<EnvironmentManager>>,
    /// 全局环境 ID
    global_env_id: EnvironmentId,
}

impl Evaluator {
    /// 创建新的求值器
    pub fn new() -> Self {
        let env_manager = Rc::new(RefCell::new(EnvironmentManager::new()));
        let global_env = Environment::new(Rc::clone(&env_manager));
        let global_env_id = global_env.id();
        
        // 注册内置函数
        Self::register_builtins(&global_env);
        
        Evaluator {
            env_manager,
            global_env_id,
        }
    }

    /// 获取全局环境
    pub fn global_env(&self) -> Environment {
        Environment {
            id: self.global_env_id,
            manager: Rc::clone(&self.env_manager),
        }
    }

    /// 注册内置函数
    fn register_builtins(env: &Environment) {
        // 算术运算
        env.define("+".to_string(), Value::BuiltinFunction {
            name: "+".to_string(),
            func: builtins::add,
            arity: None, // 可变参数
        }).unwrap();
        
        env.define("-".to_string(), Value::BuiltinFunction {
            name: "-".to_string(),
            func: builtins::subtract,
            arity: None,
        }).unwrap();
        
        env.define("*".to_string(), Value::BuiltinFunction {
            name: "*".to_string(),
            func: builtins::multiply,
            arity: None,
        }).unwrap();
        
        env.define("/".to_string(), Value::BuiltinFunction {
            name: "/".to_string(),
            func: builtins::divide,
            arity: None,
        }).unwrap();

        // 比较运算
        env.define("=".to_string(), Value::BuiltinFunction {
            name: "=".to_string(),
            func: builtins::equal,
            arity: Some(2),
        }).unwrap();
        
        env.define("<".to_string(), Value::BuiltinFunction {
            name: "<".to_string(),
            func: builtins::less_than,
            arity: Some(2),
        }).unwrap();
        
        env.define(">".to_string(), Value::BuiltinFunction {
            name: ">".to_string(),
            func: builtins::greater_than,
            arity: Some(2),
        }).unwrap();
        
        env.define("<=".to_string(), Value::BuiltinFunction {
            name: "<=".to_string(),
            func: builtins::less_equal,
            arity: Some(2),
        }).unwrap();
        
        env.define(">=".to_string(), Value::BuiltinFunction {
            name: ">=".to_string(),
            func: builtins::greater_equal,
            arity: Some(2),
        }).unwrap();

        // 数学函数
        env.define("abs".to_string(), Value::BuiltinFunction {
            name: "abs".to_string(),
            func: builtins::abs_func,
            arity: Some(1),
        }).unwrap();
        
        env.define("max".to_string(), Value::BuiltinFunction {
            name: "max".to_string(),
            func: builtins::max_func,
            arity: None,
        }).unwrap();
        
        env.define("min".to_string(), Value::BuiltinFunction {
            name: "min".to_string(),
            func: builtins::min_func,
            arity: None,
        }).unwrap();

        // 列表操作
        env.define("cons".to_string(), Value::BuiltinFunction {
            name: "cons".to_string(),
            func: builtins::cons,
            arity: Some(2),
        }).unwrap();
        
        env.define("car".to_string(), Value::BuiltinFunction {
            name: "car".to_string(),
            func: builtins::car,
            arity: Some(1),
        }).unwrap();
        
        env.define("cdr".to_string(), Value::BuiltinFunction {
            name: "cdr".to_string(),
            func: builtins::cdr,
            arity: Some(1),
        }).unwrap();
        
        env.define("list".to_string(), Value::BuiltinFunction {
            name: "list".to_string(),
            func: builtins::list,
            arity: None,
        }).unwrap();

        // 类型谓词
        env.define("null?".to_string(), Value::BuiltinFunction {
            name: "null?".to_string(),
            func: builtins::is_null,
            arity: Some(1),
        }).unwrap();
        
        env.define("pair?".to_string(), Value::BuiltinFunction {
            name: "pair?".to_string(),
            func: builtins::is_pair,
            arity: Some(1),
        }).unwrap();
        
        env.define("number?".to_string(), Value::BuiltinFunction {
            name: "number?".to_string(),
            func: builtins::is_number,
            arity: Some(1),
        }).unwrap();
        
        env.define("symbol?".to_string(), Value::BuiltinFunction {
            name: "symbol?".to_string(),
            func: builtins::is_symbol,
            arity: Some(1),
        }).unwrap();
        
        env.define("string?".to_string(), Value::BuiltinFunction {
            name: "string?".to_string(),
            func: builtins::is_string,
            arity: Some(1),
        }).unwrap();
    }

    /// 求值表达式
    pub fn eval(&self, expr: &Value, env: &Environment) -> Result<Value> {
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
                            "set!" => self.eval_set(&list[1..], env),
                            "lambda" => self.eval_lambda(&list[1..], env),
                            "let" => self.eval_let(&list[1..], env),
                            "begin" => self.eval_begin(&list[1..], env),
                            "and" => self.eval_and(&list[1..], env),
                            "or" => self.eval_or(&list[1..], env),
                            "cond" => self.eval_cond(&list[1..], env),
                            _ => self.eval_application(&list, env),
                        }
                    } else {
                        self.eval_application(&list, env)
                    }
                } else {
                    Err(SchemeError::RuntimeError("Invalid list structure".to_string()))
                }
            },
            
            _ => Err(SchemeError::RuntimeError(format!("Cannot evaluate {expr}"))),
        }
    }

    /// 求值 quote 特殊形式
    fn eval_quote(&self, args: &[Value], _env: &Environment) -> Result<Value> {
        if args.len() != 1 {
            return Err(SchemeError::ArityError("quote requires exactly 1 argument".to_string()));
        }
        Ok(args[0].clone())
    }

    /// 求值 if 特殊形式
    fn eval_if(&self, args: &[Value], env: &Environment) -> Result<Value> {
        if args.len() < 2 || args.len() > 3 {
            return Err(SchemeError::ArityError("if requires 2 or 3 arguments".to_string()));
        }

        let condition = self.eval(&args[0], env)?;
        
        if condition.is_truthy() {
            self.eval(&args[1], env)
        } else if args.len() == 3 {
            self.eval(&args[2], env)
        } else {
            Ok(Value::Nil)
        }
    }

    /// 求值 define 特殊形式
    fn eval_define(&self, args: &[Value], env: &Environment) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("define requires exactly 2 arguments".to_string()));
        }

        match &args[0] {
            Value::Symbol(name) => {
                let value = self.eval(&args[1], env)?;
                env.define(name.clone(), value)?;
                Ok(Value::Nil)
            },
            _ => Err(SchemeError::TypeError("define expects a symbol".to_string())),
        }
    }

    /// 求值 set! 特殊形式
    fn eval_set(&self, args: &[Value], env: &Environment) -> Result<Value> {
        if args.len() != 2 {
            return Err(SchemeError::ArityError("set! requires exactly 2 arguments".to_string()));
        }

        match &args[0] {
            Value::Symbol(name) => {
                let value = self.eval(&args[1], env)?;
                env.set(name, value)?;
                Ok(Value::Nil)
            },
            _ => Err(SchemeError::TypeError("set! expects a symbol".to_string())),
        }
    }

    /// 求值 lambda 特殊形式
    fn eval_lambda(&self, args: &[Value], env: &Environment) -> Result<Value> {
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
            env_id: env.id(),
        })
    }

    /// 求值 let 特殊形式
    fn eval_let(&self, args: &[Value], env: &Environment) -> Result<Value> {
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
                                    let value = self.eval(&pair[1], env)?;
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
        self.eval(&args[1], &new_env)
    }

    /// 求值 begin 特殊形式
    fn eval_begin(&self, args: &[Value], env: &Environment) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Nil);
        }

        let mut result = Value::Nil;
        for expr in args {
            result = self.eval(expr, env)?;
        }
        Ok(result)
    }

    /// 求值 and 特殊形式
    fn eval_and(&self, args: &[Value], env: &Environment) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Bool(true));
        }

        for arg in args {
            let result = self.eval(arg, env)?;
            if !result.is_truthy() {
                return Ok(result);
            }
        }
        
        // 如果所有表达式都为真，返回最后一个表达式的值
        self.eval(&args[args.len() - 1], env)
    }

    /// 求值 or 特殊形式
    fn eval_or(&self, args: &[Value], env: &Environment) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Bool(false));
        }

        for arg in args {
            let result = self.eval(arg, env)?;
            if result.is_truthy() {
                return Ok(result);
            }
        }
        
        // 如果所有表达式都为假，返回最后一个表达式的值
        self.eval(&args[args.len() - 1], env)
    }

    /// 求值 cond 特殊形式
    fn eval_cond(&self, args: &[Value], env: &Environment) -> Result<Value> {
        for clause in args {
            if let Some(clause_list) = clause.to_vec() {
                if clause_list.len() < 1 {
                    return Err(SchemeError::SyntaxError(
                        "cond clause must have at least a condition".to_string()
                    ));
                }
                
                // 检查是否为 else 子句
                if let Value::Symbol(s) = &clause_list[0] {
                    if s == "else" {
                        if clause_list.len() == 1 {
                            return Ok(Value::Nil);
                        } else if clause_list.len() == 2 {
                            return self.eval(&clause_list[1], env);
                        } else {
                            // 多个表达式，当作 begin 处理
                            return self.eval_begin(&clause_list[1..], env);
                        }
                    }
                }
                
                // 求值条件
                let condition = self.eval(&clause_list[0], env)?;
                
                if condition.is_truthy() {
                    if clause_list.len() == 1 {
                        return Ok(condition);
                    } else if clause_list.len() == 2 {
                        return self.eval(&clause_list[1], env);
                    } else {
                        // 多个表达式，当作 begin 处理
                        return self.eval_begin(&clause_list[1..], env);
                    }
                }
            } else {
                return Err(SchemeError::SyntaxError(
                    "cond clause must be a list".to_string()
                ));
            }
        }
        
        // 没有匹配的子句
        Ok(Value::Nil)
    }

    /// 求值函数应用
    fn eval_application(&self, exprs: &[Value], env: &Environment) -> Result<Value> {
        if exprs.is_empty() {
            return Ok(Value::Nil);
        }

        // 求值函数
        let func = self.eval(&exprs[0], env)?;
        
        // 求值参数
        let mut args = Vec::new();
        for arg_expr in &exprs[1..] {
            args.push(self.eval(arg_expr, env)?);
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
            
            Value::Lambda { params, body, env_id } => {
                if args.len() != params.len() {
                    return Err(SchemeError::ArityError(
                        format!("Expected {} arguments, got {}", params.len(), args.len())
                    ));
                }
                
                // 从环境ID创建新环境绑定参数
                let closure_env = Environment::from_id(env_id, self.env_manager.clone());
                let new_env = closure_env.extend(params, args)?;
                self.eval(&body, &new_env)
            },
            
            _ => Err(SchemeError::TypeError(
                format!("Cannot apply non-function: {func}")
            )),
        }
    }

    /// 便利方法：求值字符串
    pub fn eval_string(&self, input: &str) -> Result<Value> {
        let expr = crate::parser::Parser::parse(input)?;
        let global_env = Environment::from_id(self.global_env_id, self.env_manager.clone());
        self.eval(&expr, &global_env)
    }

    /// 获取全局环境
    pub fn get_global_env(&self) -> Environment {
        Environment::from_id(self.global_env_id, self.env_manager.clone())
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

    #[test]
    fn test_define_and_lookup() {
        let evaluator = Evaluator::new();
        
        // 测试 define
        assert_eq!(evaluator.eval_string("(define x 42)").unwrap(), Value::Nil);
        assert_eq!(evaluator.eval_string("x").unwrap(), Value::Integer(42));
        
        // 测试字符串变量
        assert_eq!(evaluator.eval_string("(define name \"hello\")").unwrap(), Value::Nil);
        assert_eq!(evaluator.eval_string("name").unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_set_variable() {
        let evaluator = Evaluator::new();
        
        // 先定义一个变量
        evaluator.eval_string("(define x 10)").unwrap();
        assert_eq!(evaluator.eval_string("x").unwrap(), Value::Integer(10));
        
        // 使用 set! 修改变量
        assert_eq!(evaluator.eval_string("(set! x 20)").unwrap(), Value::Nil);
        assert_eq!(evaluator.eval_string("x").unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_lambda_with_environment() {
        let evaluator = Evaluator::new();
        
        // 测试 lambda 函数和环境
        evaluator.eval_string("(define add (lambda (x y) (+ x y)))").unwrap();
        assert_eq!(evaluator.eval_string("(add 3 4)").unwrap(), Value::Integer(7));
        
        // 测试闭包
        evaluator.eval_string("(define make-adder (lambda (n) (lambda (x) (+ x n))))").unwrap();
        evaluator.eval_string("(define add5 (make-adder 5))").unwrap();
        assert_eq!(evaluator.eval_string("(add5 10)").unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_comparison_operators() {
        let evaluator = Evaluator::new();
        
        // 测试 >
        assert_eq!(evaluator.eval_string("(> 5 3)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(> 3 5)").unwrap(), Value::Bool(false));
        
        // 测试 <=
        assert_eq!(evaluator.eval_string("(<= 3 5)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(<= 5 5)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(<= 7 5)").unwrap(), Value::Bool(false));
        
        // 测试 >=
        assert_eq!(evaluator.eval_string("(>= 5 3)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(>= 5 5)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(>= 3 5)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_math_functions() {
        let evaluator = Evaluator::new();
        
        // 测试 abs
        assert_eq!(evaluator.eval_string("(abs -5)").unwrap(), Value::Integer(5));
        assert_eq!(evaluator.eval_string("(abs 3)").unwrap(), Value::Integer(3));
        assert_eq!(evaluator.eval_string("(abs -3.14)").unwrap(), Value::Float(3.14));
        
        // 测试 max
        assert_eq!(evaluator.eval_string("(max 1 2 3)").unwrap(), Value::Integer(3));
        assert_eq!(evaluator.eval_string("(max 5 2 8 1)").unwrap(), Value::Integer(8));
        assert_eq!(evaluator.eval_string("(max 1.5 2 3.7)").unwrap(), Value::Float(3.7));
        
        // 测试 min
        assert_eq!(evaluator.eval_string("(min 3 1 2)").unwrap(), Value::Integer(1));
        assert_eq!(evaluator.eval_string("(min 5 2 8 1)").unwrap(), Value::Integer(1));
        assert_eq!(evaluator.eval_string("(min 1.5 2 0.3)").unwrap(), Value::Float(0.3));
    }

    #[test]
    fn test_logical_operators() {
        let evaluator = Evaluator::new();
        
        // 测试 and
        assert_eq!(evaluator.eval_string("(and)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(and #t)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(and #f)").unwrap(), Value::Bool(false));
        assert_eq!(evaluator.eval_string("(and #t #t)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(and #t #f)").unwrap(), Value::Bool(false));
        assert_eq!(evaluator.eval_string("(and 1 2 3)").unwrap(), Value::Integer(3));
        
        // 测试 or
        assert_eq!(evaluator.eval_string("(or)").unwrap(), Value::Bool(false));
        assert_eq!(evaluator.eval_string("(or #f)").unwrap(), Value::Bool(false));
        assert_eq!(evaluator.eval_string("(or #t)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(or #f #t)").unwrap(), Value::Bool(true));
        assert_eq!(evaluator.eval_string("(or #f #f)").unwrap(), Value::Bool(false));
        assert_eq!(evaluator.eval_string("(or #f 42)").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_cond() {
        let evaluator = Evaluator::new();
        
        // 基本 cond 测试
        assert_eq!(
            evaluator.eval_string("(cond (#t 1))").unwrap(), 
            Value::Integer(1)
        );
        assert_eq!(
            evaluator.eval_string("(cond (#f 1) (#t 2))").unwrap(), 
            Value::Integer(2)
        );
        
        // else 子句
        assert_eq!(
            evaluator.eval_string("(cond (#f 1) (else 42))").unwrap(), 
            Value::Integer(42)
        );
        
        // 没有匹配的子句
        assert_eq!(
            evaluator.eval_string("(cond (#f 1) (#f 2))").unwrap(), 
            Value::Nil
        );
    }
}
