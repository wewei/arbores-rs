//! Lambda 函数结构定义
//! 
//! 用户定义的函数，包含参数、函数体和闭包环境

use std::rc::Rc;
use crate::interpreter::SExpr;
use super::Environment;
use gc::{Trace, Finalize, Gc};

/// Lambda 静态部分 - 不可变的参数和函数体
#[derive(Debug, Clone, Trace, Finalize, PartialEq)]
pub struct LambdaStatic {
    /// 参数名列表
    #[unsafe_ignore_trace]
    pub parameters: Vec<String>,
    /// 函数体（语法结构）
    #[unsafe_ignore_trace]
    pub body: Rc<SExpr>,
}

/// Lambda 函数 - 用户定义的函数
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Lambda {
    /// 静态部分（参数和函数体）- 用 Rc 引用，8 bytes
    #[unsafe_ignore_trace]
    pub static_part: Rc<LambdaStatic>,
    /// 闭包环境（可变，用 Gc 包装）- 8 bytes
    pub closure: Gc<Environment>,
}

impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.static_part, &other.static_part) && Gc::ptr_eq(&self.closure, &other.closure)
    }
}
