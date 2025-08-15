//! 可变 Cons 结构定义
//! 
//! 支持 set-car! 和 set-cdr! 操作的可变列表结构

use gc::{Trace, Finalize, Gc};
use super::RuntimeObject;

/// 可变 Cons 结构 - 支持 set-car! 和 set-cdr! 操作
#[derive(Debug, Clone, Trace, Finalize)]
pub struct MutableCons {
    /// car 部分 - 使用 Gc 支持可变性
    pub car: Gc<RuntimeObject>,
    /// cdr 部分 - 使用 Gc 支持可变性
    pub cdr: Gc<RuntimeObject>,
}

impl PartialEq for MutableCons {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(&self.car, &other.car) && Gc::ptr_eq(&self.cdr, &other.cdr)
    }
}

impl MutableCons {
    /// 创建新的可变 Cons
    pub fn new(car: RuntimeObject, cdr: RuntimeObject) -> Self {
        Self {
            car: Gc::new(car),
            cdr: Gc::new(cdr),
        }
    }
    
    /// 获取 car 部分
    pub fn car(&self) -> RuntimeObject {
        (*self.car).clone()
    }
    
    /// 获取 cdr 部分
    pub fn cdr(&self) -> RuntimeObject {
        (*self.cdr).clone()
    }
    
    /// 设置 car 部分
    pub fn set_car(&mut self, value: RuntimeObject) {
        self.car = Gc::new(value);
    }
    
    /// 设置 cdr 部分
    pub fn set_cdr(&mut self, value: RuntimeObject) {
        self.cdr = Gc::new(value);
    }
}
