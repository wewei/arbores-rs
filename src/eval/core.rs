use std::rc::Rc;
use std::cell::RefCell;
use crate::types::{Value, SchemeError, Result, LocatedValue};
use crate::env::{Environment, EnvironmentManager, EnvironmentId};
use crate::eval::special_forms::SpecialFormsEvaluator;
use crate::eval::context::EvaluationContext;

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
    pub fn eval(&self, expr: &Value, env: &Environment, context: Option<&EvaluationContext>) -> Result<Value> {
        // 辅助函数：为错误添加位置信息和调用栈
        let enrich_error = |error: SchemeError| -> SchemeError {
            if let Some(ctx) = context {
                // 添加位置信息
                let error_with_pos = if let Some(pos) = ctx.current_position {
                    match error {
                        SchemeError::UndefinedVariable(name, None) => 
                            SchemeError::UndefinedVariable(name, Some(pos)),
                        SchemeError::TypeError(msg, None) => 
                            SchemeError::TypeError(msg, Some(pos)),
                        SchemeError::RuntimeError(msg, None) => 
                            SchemeError::RuntimeError(msg, Some(pos)),
                        SchemeError::ArityError(msg, None) => 
                            SchemeError::ArityError(msg, Some(pos)),
                        SchemeError::DivisionByZero(None) => 
                            SchemeError::DivisionByZero(Some(pos)),
                        other => other, // 已经有位置信息的错误保持不变
                    }
                } else {
                    error
                };
                
                // 添加调用栈信息
                let call_stack = ctx.format_call_stack();
                if !call_stack.is_empty() {
                    error_with_pos.with_callstack(call_stack)
                } else {
                    error_with_pos
                }
            } else {
                error
            }
        };

        match expr {
            // 自求值表达式
            Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Bool(_) => {
                Ok(expr.clone())
            },
            
            // 空列表
            Value::Nil => Ok(Value::Nil),
            
            // 符号（变量查找）
            Value::Symbol(name) => env.lookup(name).map_err(enrich_error),
            
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
                            "if" => SpecialFormsEvaluator::eval_if(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            "define" => SpecialFormsEvaluator::eval_define(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            "set!" => SpecialFormsEvaluator::eval_set(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            "lambda" => SpecialFormsEvaluator::eval_lambda(&list[1..], env),
                            "let" => SpecialFormsEvaluator::eval_let(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            "begin" => SpecialFormsEvaluator::eval_begin(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            "and" => SpecialFormsEvaluator::eval_and(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            "or" => SpecialFormsEvaluator::eval_or(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            "cond" => SpecialFormsEvaluator::eval_cond(&list[1..], env, &|e, env| self.eval(e, env, context)),
                            _ => {
                                // 对于函数调用，创建一个包含调用位置的新上下文
                                // TODO: 改进位置信息传递
                                // 当前实现传递调用点位置，但理想情况下应该为每个
                                // 函数体内的表达式提供更精确的位置信息
                                let call_context = context.map(|ctx| 
                                    ctx.enter_call(ctx.current_position, None)
                                );
                                self.eval_application(&list, env, call_context.as_ref())
                            },
                        }
                    } else {
                        // 对于函数调用，创建一个包含调用位置的新上下文
                        let call_context = context.map(|ctx| 
                            ctx.enter_call(ctx.current_position, None)
                        );
                        self.eval_application(&list, env, call_context.as_ref())
                    }
                } else {
                    Err(enrich_error(SchemeError::RuntimeError("Invalid list structure".to_string(), None)))
                }
            },
            
            _ => Err(enrich_error(SchemeError::RuntimeError(format!("Cannot evaluate {expr}"), None))),
        }
    }

    /// 求值函数应用
    fn eval_application(&self, exprs: &[Value], env: &Environment, context: Option<&EvaluationContext>) -> Result<Value> {
        // 辅助函数：为错误添加位置信信息
        let enrich_error = |error: SchemeError| -> SchemeError {
            if let Some(ctx) = context {
                // 添加位置信息
                let error_with_pos = if let Some(pos) = ctx.current_position {
                    match error {
                        SchemeError::UndefinedVariable(name, None) => 
                            SchemeError::UndefinedVariable(name, Some(pos)),
                        SchemeError::TypeError(msg, None) => 
                            SchemeError::TypeError(msg, Some(pos)),
                        SchemeError::RuntimeError(msg, None) => 
                            SchemeError::RuntimeError(msg, Some(pos)),
                        SchemeError::ArityError(msg, None) => 
                            SchemeError::ArityError(msg, Some(pos)),
                        SchemeError::DivisionByZero(None) => 
                            SchemeError::DivisionByZero(Some(pos)),
                        other => other, // 已经有位置信息的错误保持不变
                    }
                } else {
                    error
                };
                
                // 添加调用栈信息
                let call_stack = ctx.format_call_stack();
                if !call_stack.is_empty() {
                    error_with_pos.with_callstack(call_stack)
                } else {
                    error_with_pos
                }
            } else {
                error
            }
        };

        if exprs.is_empty() {
            return Ok(Value::Nil);
        }

        // 在求值前保存函数名（如果是符号的话）
        let func_name = if let Value::Symbol(name) = &exprs[0] {
            Some(name.clone())
        } else {
            None
        };

        // 求值函数
        let func = self.eval(&exprs[0], env, context)?;
        
        // 求值参数
        let mut args = Vec::new();
        for arg_expr in &exprs[1..] {
            args.push(self.eval(arg_expr, env, context)?);
        }

        // 应用函数
        match func {
            Value::BuiltinFunction { func, arity, .. } => {
                // 检查参数个数
                if let Some(expected_arity) = arity {
                    if args.len() != expected_arity {
                        return Err(enrich_error(SchemeError::ArityError(
                            format!("Expected {} arguments, got {}", expected_arity, args.len()), None
                        )));
                    }
                }
                func(&args).map_err(enrich_error)
            },
            
            Value::Lambda { params, body, env_id } => {
                if args.len() != params.len() {
                    return Err(enrich_error(SchemeError::ArityError(
                        format!("Expected {} arguments, got {}", params.len(), args.len()), None
                    )));
                }
                
                // 从环境ID创建新环境绑定参数
                let closure_env = Environment::from_id(env_id, self.env_manager.clone());
                let new_env = closure_env.extend(params, args)?;
                
                // 创建子上下文，使用函数名更新上下文
                let child_context = context.map(|ctx| {
                    let mut new_ctx = ctx.clone();
                    // 如果上下文还没有函数名，设置函数名
                    if new_ctx.function_name.is_none() {
                        new_ctx.function_name = func_name.or(Some("<lambda>".to_string()));
                    }
                    new_ctx
                });
                
                self.eval(&body, &new_env, child_context.as_ref())
            },
            
            _ => Err(enrich_error(SchemeError::TypeError(format!("Cannot apply non-function: {func}"), None))),
        }
    }

    /// 便利方法：求值字符串
    pub fn eval_string(&self, input: &str, context: Option<&EvaluationContext>) -> Result<Value> {
        let expr = crate::parser::Parser::parse(input)?;
        let global_env = Environment::from_id(self.global_env_id, self.env_manager.clone());
        
        // 如果没有提供上下文，创建一个根上下文以支持 callstack 追踪
        if let Some(ctx) = context {
            self.eval(&expr, &global_env, Some(ctx))
        } else {
            let default_context = EvaluationContext::new();
            self.eval(&expr, &global_env, Some(&default_context))
        }
    }

    /// 获取全局环境
    pub fn get_global_env(&self) -> Environment {
        Environment::from_id(self.global_env_id, self.env_manager.clone())
    }

    /// 求值带位置信息的表达式
    pub fn eval_located(&self, located_expr: &LocatedValue, env: &Environment, context: Option<&EvaluationContext>) -> Result<Value> {
        // 如果表达式有位置信息，创建一个新的上下文使用这个位置
        let enhanced_context = if let Some(pos) = located_expr.position {
            context.map(|ctx| {
                let mut new_ctx = ctx.clone();
                new_ctx.current_position = Some(pos);
                new_ctx
            }).or_else(|| {
                // 如果没有提供上下文，创建一个新的上下文
                Some(EvaluationContext {
                    current_position: Some(pos),
                    function_name: None,
                    parent: None,
                })
            })
        } else {
            context.map(|ctx| ctx.clone())
        };
        
        self.eval(&located_expr.value, env, enhanced_context.as_ref())
    }
    
    /// 求值带位置信息的字符串
    pub fn eval_string_located(&self, input: &str, context: Option<&EvaluationContext>) -> Result<Value> {
        let located_expr = crate::parser::Parser::parse_located(input)?;
        let global_env = Environment::from_id(self.global_env_id, self.env_manager.clone());
        self.eval_located(&located_expr, &global_env, context)
    }
}

impl Default for CoreEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
