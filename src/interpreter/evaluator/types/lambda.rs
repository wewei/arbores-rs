//! Lambda 函数结构定义
//! 
//! 用户定义的函数，包含参数、函数体和闭包环境

use std::rc::Rc;
use crate::interpreter::SExpr;
use super::Environment;

/// Lambda 函数 - 用户定义的函数
#[derive(Debug)]
pub struct Lambda {
    /// 参数名列表
    pub parameters: Vec<String>,
    /// 函数体（语法结构）
    pub body: Rc<SExpr>,
    /// 闭包环境（可变，用 Rc 包装）
    pub closure: Rc<Environment>,
}
