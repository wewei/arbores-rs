//! 环境结构定义
//! 
//! 可变的链式结构，支持变量绑定修改

use std::collections::HashMap;
use std::rc::Rc;
use super::RuntimeObject;

/// 环境结构 - 可变的链式结构，支持变量绑定修改
#[derive(Debug)]
pub struct Environment {
    /// 当前环境的变量绑定表
    pub bindings: HashMap<String, RuntimeObject>,
    /// 上级环境（链式结构）
    pub parent: Option<Rc<Environment>>,
}

impl Environment {
    /// 创建新的空环境
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    /// 创建带父环境的新环境
    pub fn with_parent(parent: Rc<Environment>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }
    
    /// 在当前环境中定义变量（返回新环境）
    pub fn define(&self, name: String, value: RuntimeObject) -> Self {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(name, value);
        Self {
            bindings: new_bindings,
            parent: self.parent.clone(),
        }
    }
    
    /// 设置变量值（如果变量存在，返回新环境）
    pub fn set(&self, name: &str, value: RuntimeObject) -> Result<Self, String> {
        // 先在当前环境查找
        if self.bindings.contains_key(name) {
            let mut new_bindings = self.bindings.clone();
            new_bindings.insert(name.to_string(), value);
            return Ok(Self {
                bindings: new_bindings,
                parent: self.parent.clone(),
            });
        }
        
        // 递归在父环境查找
        if let Some(parent) = &self.parent {
            let new_parent = parent.set(name, value)?;
            Ok(Self {
                bindings: self.bindings.clone(),
                parent: Some(Rc::new(new_parent)),
            })
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }
    
    /// 查找变量值（递归向上查找）
    pub fn lookup(&self, name: &str) -> Option<RuntimeObject> {
        // 先在当前环境查找
        if let Some(value) = self.bindings.get(name) {
            return Some(value.clone());
        }
        
        // 递归在父环境查找
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}
