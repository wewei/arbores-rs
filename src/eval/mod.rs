// 导出子模块
pub mod builtins;
pub mod special_forms;
pub mod core;

// 重新导出主要类型
pub use core::CoreEvaluator;
pub use builtins::register_builtins;

use crate::types::{Value, Result};
use crate::env::Environment;

/// 求值器 - 重构后的主求值器
pub struct Evaluator {
    /// 核心求值器
    core: CoreEvaluator,
}

impl Evaluator {
    /// 创建新的求值器
    pub fn new() -> Self {
        let evaluator = Evaluator {
            core: CoreEvaluator::new(),
        };
        
        // 注册内置函数
        let global_env = evaluator.core.global_env();
        register_builtins(&global_env);
        
        evaluator
    }

    /// 获取全局环境
    pub fn global_env(&self) -> Environment {
        self.core.global_env()
    }

    /// 求值表达式
    pub fn eval(&self, expr: &Value, env: &Environment) -> Result<Value> {
        self.core.eval(expr, env)
    }

    /// 便利方法：求值字符串
    pub fn eval_string(&self, input: &str) -> Result<Value> {
        self.core.eval_string(input)
    }

    /// 获取全局环境
    pub fn get_global_env(&self) -> Environment {
        self.core.get_global_env()
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
