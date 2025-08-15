//! 可变向量结构定义
//! 
//! 支持 vector-set! 操作的可变向量结构

use super::RuntimeObject;
use gc::{Trace, Finalize, Gc, GcCell};

/// 可变向量 - 支持 vector-set! 操作
#[derive(Debug, Clone, Trace, Finalize)]
pub struct MutableVector {
    /// 向量元素 - 使用 GcCell 支持可变性
    pub elements: Gc<Vec<RuntimeObject>>,
}

impl PartialEq for MutableVector {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&self.elements, &other.elements)
    }
}

impl MutableVector {
    /// 创建新的可变向量
    pub fn new(elements: Gc<Vec<RuntimeObject>>) -> Self {
        Self { elements }
    }
    
    /// 获取向量长度
    pub fn len(&self) -> usize {
        (*self.elements).len()
    }
    
    /// 检查向量是否为空
    pub fn is_empty(&self) -> bool {
        (*self.elements).is_empty()
    }
    
    /// 获取指定索引的元素
    pub fn get(&self, index: usize) -> Option<RuntimeObject> {
        (*self.elements).get(index).cloned()
    }
    
    /// 设置指定索引的元素
    pub fn set(&self, index: usize, value: RuntimeObject) -> Result<(), String> {
        let mut elements = (*self.elements).clone();
        if index < elements.len() {
            elements[index] = value;
            Ok(())
        } else {
            Err(format!("Index {} out of bounds for vector of length {}", index, elements.len()))
        }
    }
    
    /// 获取向量的所有元素（克隆）
    pub fn to_vec(&self) -> Vec<RuntimeObject> {
        (*self.elements).clone()
    }
}
