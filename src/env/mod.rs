use std::collections::HashMap;
use std::rc::Rc;
use crate::types::{Value, SchemeError, Result};

/// 环境（作用域）结构，用于存储变量绑定
#[derive(Debug, Clone)]
pub struct Environment {
    /// 当前环境的变量绑定
    bindings: HashMap<String, Value>,
    /// 父环境的引用
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// 创建新的全局环境
    pub fn new() -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// 创建一个新的子环境
    pub fn new_child(parent: Rc<Environment>) -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// 在当前环境中定义变量
    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// 查找变量，从当前环境开始，向上搜索父环境
    pub fn lookup(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.bindings.get(name) {
            Ok(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            Err(SchemeError::UndefinedVariable(name.to_string()))
        }
    }

    /// 设置变量的值（必须是已存在的变量）
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            Ok(())
        } else if let Some(_parent) = &self.parent {
            // 如果当前环境没有，尝试在父环境中设置
            // 注意：这里需要修改父环境，但 Rc 是不可变的
            // 在实际实现中可能需要使用 RefCell 或其他方案
            Err(SchemeError::RuntimeError(
                "Cannot modify parent environment through Rc".to_string()
            ))
        } else {
            Err(SchemeError::UndefinedVariable(name.to_string()))
        }
    }

    /// 创建一个扩展了指定绑定的新环境
    pub fn extend(&self, names: Vec<String>, values: Vec<Value>) -> Result<Environment> {
        if names.len() != values.len() {
            return Err(SchemeError::ArityError(
                format!("Expected {} arguments, got {}", names.len(), values.len())
            ));
        }

        let mut new_env = Environment::new_child(Rc::new(self.clone()));
        for (name, value) in names.into_iter().zip(values.into_iter()) {
            new_env.define(name, value);
        }

        Ok(new_env)
    }

    /// 获取当前环境中定义的所有变量名
    pub fn get_local_bindings(&self) -> Vec<&String> {
        self.bindings.keys().collect()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_basic() {
        let mut env = Environment::new();
        
        // 定义变量
        env.define("x".to_string(), Value::Integer(42));
        
        // 查找变量
        assert_eq!(env.lookup("x"), Ok(Value::Integer(42)));
        
        // 查找不存在的变量
        assert!(env.lookup("y").is_err());
    }

    #[test]
    fn test_environment_parent() {
        let mut parent = Environment::new();
        parent.define("x".to_string(), Value::Integer(42));
        
        let mut child = Environment::new_child(Rc::new(parent));
        child.define("y".to_string(), Value::String("hello".to_string()));
        
        // 子环境可以访问父环境的变量
        assert_eq!(child.lookup("x"), Ok(Value::Integer(42)));
        assert_eq!(child.lookup("y"), Ok(Value::String("hello".to_string())));
    }

    #[test]
    fn test_environment_extend() {
        let env = Environment::new();
        
        let names = vec!["a".to_string(), "b".to_string()];
        let values = vec![Value::Integer(1), Value::Integer(2)];
        
        let extended = env.extend(names, values).unwrap();
        
        assert_eq!(extended.lookup("a"), Ok(Value::Integer(1)));
        assert_eq!(extended.lookup("b"), Ok(Value::Integer(2)));
    }
}
