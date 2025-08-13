//! 可变向量结构定义
//! 
//! 支持 vector-set! 操作的可变向量结构

use std::cell::RefCell;
use super::RuntimeObject;

/// 可变向量 - 支持 vector-set! 操作
#[derive(Debug)]
pub struct MutableVector {
    /// 向量元素 - 使用 RefCell 支持可变性
    pub elements: RefCell<Vec<RuntimeObject>>,
}

impl MutableVector {
    /// 创建新的可变向量
    pub fn new(elements: Vec<RuntimeObject>) -> Self {
        Self {
            elements: RefCell::new(elements),
        }
    }
    
    /// 获取向量长度
    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }
    
    /// 检查向量是否为空
    pub fn is_empty(&self) -> bool {
        self.elements.borrow().is_empty()
    }
    
    /// 获取指定索引的元素
    pub fn get(&self, index: usize) -> Option<RuntimeObject> {
        self.elements.borrow().get(index).cloned()
    }
    
    /// 设置指定索引的元素
    pub fn set(&self, index: usize, value: RuntimeObject) -> Result<(), String> {
        let mut elements = self.elements.borrow_mut();
        if index < elements.len() {
            elements[index] = value;
            Ok(())
        } else {
            Err(format!("Index {} out of bounds for vector of length {}", index, elements.len()))
        }
    }
    
    /// 向向量末尾添加元素
    pub fn push(&self, value: RuntimeObject) {
        self.elements.borrow_mut().push(value);
    }
    
    /// 获取向量的所有元素（克隆）
    pub fn to_vec(&self) -> Vec<RuntimeObject> {
        self.elements.borrow().clone()
    }
}
