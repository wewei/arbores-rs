//! 可变 Cons 结构定义
//! 
//! 支持 set-car! 和 set-cdr! 操作的可变列表结构

use std::cell::RefCell;
use super::RuntimeObject;

/// 可变 Cons 结构 - 支持 set-car! 和 set-cdr! 操作
#[derive(Debug)]
pub struct MutableCons {
    /// car 部分 - 使用 RefCell 支持可变性
    pub car: RefCell<RuntimeObject>,
    /// cdr 部分 - 使用 RefCell 支持可变性
    pub cdr: RefCell<RuntimeObject>,
}

impl MutableCons {
    /// 创建新的可变 Cons
    pub fn new(car: RuntimeObject, cdr: RuntimeObject) -> Self {
        Self {
            car: RefCell::new(car),
            cdr: RefCell::new(cdr),
        }
    }
    
    /// 获取 car 部分
    pub fn car(&self) -> RuntimeObject {
        self.car.borrow().clone()
    }
    
    /// 获取 cdr 部分
    pub fn cdr(&self) -> RuntimeObject {
        self.cdr.borrow().clone()
    }
    
    /// 设置 car 部分
    pub fn set_car(&self, value: RuntimeObject) {
        *self.car.borrow_mut() = value;
    }
    
    /// 设置 cdr 部分
    pub fn set_cdr(&self, value: RuntimeObject) {
        *self.cdr.borrow_mut() = value;
    }
}
