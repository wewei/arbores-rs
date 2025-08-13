//! 内置函数模块
//! 
//! 本模块组织所有的内置函数实现

pub mod arithmetic;

use super::types::*;

/// 创建包含所有内置函数的全局环境
pub fn create_global_environment() -> Environment {
    let mut env = Environment::new();
    
    // 添加算术运算函数
    arithmetic::register_arithmetic_functions(&mut env);
    
    env
}
