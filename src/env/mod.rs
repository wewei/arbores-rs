use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::types::{Value, SchemeError, Result};

/// 环境 ID 类型
pub type EnvironmentId = usize;

/// 中央环境管理器
#[derive(Debug)]
pub struct EnvironmentManager {
    /// 所有环境的绑定数据
    environments: HashMap<EnvironmentId, EnvironmentData>,
    /// 下一个可用的环境 ID
    next_id: EnvironmentId,
}

/// 单个环境的数据
#[derive(Debug, Clone)]
pub struct EnvironmentData {
    /// 当前环境的变量绑定
    bindings: HashMap<String, Value>,
    /// 父环境的 ID
    parent_id: Option<EnvironmentId>,
}

/// 环境引用（轻量级，不包含实际数据）
#[derive(Debug, Clone)]
pub struct Environment {
    /// 环境的唯一标识
    pub id: EnvironmentId,
    /// 对环境管理器的引用（唯一的可变部分）
    pub manager: Rc<RefCell<EnvironmentManager>>,
}

impl Default for EnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentManager {
    /// 创建新的环境管理器
    pub fn new() -> Self {
        EnvironmentManager {
            environments: HashMap::new(),
            next_id: 0,
        }
    }

    /// 创建新的根环境
    pub fn create_root_env(&mut self) -> EnvironmentId {
        let id = self.next_id;
        self.next_id += 1;
        
        self.environments.insert(id, EnvironmentData {
            bindings: HashMap::new(),
            parent_id: None,
        });
        
        id
    }

    /// 创建子环境
    pub fn create_child_env(&mut self, parent_id: EnvironmentId) -> EnvironmentId {
        let id = self.next_id;
        self.next_id += 1;
        
        self.environments.insert(id, EnvironmentData {
            bindings: HashMap::new(),
            parent_id: Some(parent_id),
        });
        
        id
    }

    /// 在环境中定义变量
    pub fn define(&mut self, env_id: EnvironmentId, name: String, value: Value) -> Result<()> {
        if let Some(env_data) = self.environments.get_mut(&env_id) {
            env_data.bindings.insert(name, value);
            Ok(())
        } else {
            Err(SchemeError::RuntimeError(format!("Environment {env_id} not found"), None))
        }
    }

    /// 查找变量（递归查找父环境）
    pub fn lookup(&self, env_id: EnvironmentId, name: &str) -> Result<Value> {
        if let Some(env_data) = self.environments.get(&env_id) {
            if let Some(value) = env_data.bindings.get(name) {
                Ok(value.clone())
            } else if let Some(parent_id) = env_data.parent_id {
                self.lookup(parent_id, name)
            } else {
                Err(SchemeError::UndefinedVariable(name.to_string(), None))
            }
        } else {
            Err(SchemeError::RuntimeError(format!("Environment {env_id} not found"), None))
        }
    }

    /// 设置变量值（必须是已存在的变量）
    pub fn set(&mut self, env_id: EnvironmentId, name: &str, value: Value) -> Result<()> {
        if let Some(env_data) = self.environments.get_mut(&env_id) {
            if env_data.bindings.contains_key(name) {
                env_data.bindings.insert(name.to_string(), value);
                Ok(())
            } else if let Some(parent_id) = env_data.parent_id {
                self.set(parent_id, name, value)
            } else {
                Err(SchemeError::UndefinedVariable(name.to_string(), None))
            }
        } else {
            Err(SchemeError::RuntimeError(format!("Environment {env_id} not found"), None))
        }
    }

    /// 创建一个扩展了指定绑定的新环境
    pub fn extend(&mut self, parent_id: EnvironmentId, names: Vec<String>, values: Vec<Value>) -> Result<EnvironmentId> {
        if names.len() != values.len() {
            return Err(SchemeError::ArityError(
                format!("Expected {} arguments, got {}", names.len(), values.len()), None
            ));
        }

        let new_env_id = self.create_child_env(parent_id);
        for (name, value) in names.into_iter().zip(values.into_iter()) {
            self.define(new_env_id, name, value)?;
        }

        Ok(new_env_id)
    }

    /// 获取指定环境中定义的所有变量名
    pub fn get_local_bindings(&self, env_id: EnvironmentId) -> Vec<String> {
        if let Some(env_data) = self.environments.get(&env_id) {
            env_data.bindings.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Environment {
    /// 创建新的根环境
    pub fn new(manager: Rc<RefCell<EnvironmentManager>>) -> Self {
        let id = manager.borrow_mut().create_root_env();
        Environment { id, manager }
    }

    /// 从环境ID创建环境引用
    pub fn from_id(env_id: EnvironmentId, manager: Rc<RefCell<EnvironmentManager>>) -> Self {
        Environment { id: env_id, manager }
    }

    /// 创建子环境
    pub fn new_child(&self) -> Self {
        let id = self.manager.borrow_mut().create_child_env(self.id);
        Environment {
            id,
            manager: Rc::clone(&self.manager),
        }
    }

    /// 定义变量
    pub fn define(&self, name: String, value: Value) -> Result<()> {
        self.manager.borrow_mut().define(self.id, name, value)
    }

    /// 查找变量
    pub fn lookup(&self, name: &str) -> Result<Value> {
        self.manager.borrow().lookup(self.id, name)
    }

    /// 设置变量
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        self.manager.borrow_mut().set(self.id, name, value)
    }

    /// 获取环境 ID
    pub fn id(&self) -> EnvironmentId {
        self.id
    }

    /// 创建一个扩展了指定绑定的新环境
    pub fn extend(&self, names: Vec<String>, values: Vec<Value>) -> Result<Environment> {
        let new_env_id = self.manager.borrow_mut().extend(self.id, names, values)?;
        Ok(Environment {
            id: new_env_id,
            manager: Rc::clone(&self.manager),
        })
    }

    /// 获取当前环境中定义的所有变量名
    pub fn get_local_bindings(&self) -> Vec<String> {
        self.manager.borrow().get_local_bindings(self.id)
    }
}

impl Default for Environment {
    fn default() -> Self {
        let manager = Rc::new(RefCell::new(EnvironmentManager::new()));
        Self::new(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_environment_basic() {
        let env = Environment::default();
        
        // 定义变量
        env.define("x".to_string(), Value::Integer(42)).unwrap();
        
        // 查找变量
        assert_eq!(env.lookup("x"), Ok(Value::Integer(42)));
        
        // 查找不存在的变量
        assert!(env.lookup("y").is_err());
    }

    #[test]
    fn test_environment_parent() {
        let parent = Environment::default();
        parent.define("x".to_string(), Value::Integer(42)).unwrap();
        
        let child = parent.new_child();
        child.define("y".to_string(), Value::String("hello".to_string())).unwrap();
        
        // 子环境可以访问父环境的变量
        assert_eq!(child.lookup("x"), Ok(Value::Integer(42)));
        assert_eq!(child.lookup("y"), Ok(Value::String("hello".to_string())));
    }

    #[test]
    fn test_environment_extend() {
        let env = Environment::default();
        
        let names = vec!["a".to_string(), "b".to_string()];
        let values = vec![Value::Integer(1), Value::Integer(2)];
        
        let extended = env.extend(names, values).unwrap();
        
        assert_eq!(extended.lookup("a"), Ok(Value::Integer(1)));
        assert_eq!(extended.lookup("b"), Ok(Value::Integer(2)));
    }

    #[test]
    fn test_environment_set() {
        let env = Environment::default();
        
        // 定义变量
        env.define("x".to_string(), Value::Integer(42)).unwrap();
        
        // 设置变量值
        env.set("x", Value::Integer(100)).unwrap();
        assert_eq!(env.lookup("x"), Ok(Value::Integer(100)));
        
        // 尝试设置不存在的变量应该失败
        assert!(env.set("y", Value::Integer(1)).is_err());
    }

    #[test]
    fn test_environment_set_parent() {
        let parent = Environment::default();
        parent.define("x".to_string(), Value::Integer(42)).unwrap();
        
        let child = parent.new_child();
        
        // 在子环境中修改父环境的变量
        child.set("x", Value::Integer(100)).unwrap();
        assert_eq!(child.lookup("x"), Ok(Value::Integer(100)));
        assert_eq!(parent.lookup("x"), Ok(Value::Integer(100)));
    }
}
