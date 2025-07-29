use std::rc::Rc;
use std::cell::RefCell;
use crate::types::{Value, SchemeError, Result};
use crate::env::{Environment, EnvironmentManager, EnvironmentId};
use crate::eval::special_forms::SpecialFormsEvaluator;

/// 核心求值器
pub struct CoreEvaluator {
    /// 环境管理器
    env_manager: Rc<RefCell<EnvironmentManager>>,
    /// 全局环境 ID
    global_env_id: EnvironmentId,
}

impl CoreEvaluator {
    /// 创建新的核心求值器
    pub fn new() -> Self {
        let env_manager = Rc::new(RefCell::new(EnvironmentManager::new()));
        let global_env = Environment::new(Rc::clone(&env_manager));
        let global_env_id = global_env.id();
        
        CoreEvaluator {
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

    /// 核心求值方法
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
                            "quote" => SpecialFormsEvaluator::eval_quote(&list[1..], env),
                            "if" => SpecialFormsEvaluator::eval_if(&list[1..], env, &|e, env| self.eval(e, env)),
                            "define" => SpecialFormsEvaluator::eval_define(&list[1..], env, &|e, env| self.eval(e, env)),
                            "set!" => SpecialFormsEvaluator::eval_set(&list[1..], env, &|e, env| self.eval(e, env)),
                            "lambda" => SpecialFormsEvaluator::eval_lambda(&list[1..], env),
                            "let" => SpecialFormsEvaluator::eval_let(&list[1..], env, &|e, env| self.eval(e, env)),
                            "begin" => SpecialFormsEvaluator::eval_begin(&list[1..], env, &|e, env| self.eval(e, env)),
                            "and" => SpecialFormsEvaluator::eval_and(&list[1..], env, &|e, env| self.eval(e, env)),
                            "or" => SpecialFormsEvaluator::eval_or(&list[1..], env, &|e, env| self.eval(e, env)),
                            "cond" => SpecialFormsEvaluator::eval_cond(&list[1..], env, &|e, env| self.eval(e, env)),
                            _ => self.eval_application(&list, env),
                        }
                    } else {
                        self.eval_application(&list, env)
                    }
                } else {
                    Err(SchemeError::RuntimeError("Invalid list structure".to_string(), None))
                }
            },
            
            _ => Err(SchemeError::RuntimeError(format!("Cannot evaluate {expr}"), None)),
        }
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
                            format!("Expected {} arguments, got {}", expected_arity, args.len()), None
                        ));
                    }
                }
                func(&args)
            },
            
            Value::Lambda { params, body, env_id } => {
                if args.len() != params.len() {
                    return Err(SchemeError::ArityError(
                        format!("Expected {} arguments, got {}", params.len(), args.len()), None
                    ));
                }
                
                // 从环境ID创建新环境绑定参数
                let closure_env = Environment::from_id(env_id, self.env_manager.clone());
                let new_env = closure_env.extend(params, args)?;
                self.eval(&body, &new_env)
            },
            
            _ => Err(SchemeError::TypeError(format!("Cannot apply non-function: {func}"), None)),
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

impl Default for CoreEvaluator {
    fn default() -> Self {
        Self::new()
    }
} 