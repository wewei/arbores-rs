//! 特殊形式模块
//! 
//! 本模块包含所有特殊形式的实现，这些是 Scheme 语言的核心语法结构：
//! 
//! TODO: 实现以下特殊形式
//! 
//! ## 基本特殊形式
//! - `define` - 变量和函数定义
//! - `lambda` - 匿名函数创建
//! - `if` - 条件表达式
//! - `quote` - 字面量引用
//! - `set!` - 变量赋值
//! 
//! ## 控制流特殊形式
//! - `begin` - 顺序执行
//! - `cond` - 条件分支
//! - `case` - 模式匹配
//! - `and`, `or` - 逻辑运算
//! - `let`, `let*`, `letrec` - 局部绑定
//! 
//! ## 迭代特殊形式
//! - `do` - 迭代循环
//! - `for-each`, `map` - 高阶函数
//! - `call/cc` - 续延
//! 
//! ## 宏系统
//! - `define-syntax` - 语法定义
//! - `syntax-rules` - 语法规则
//! - `macro-expand` - 宏展开
//! 
//! ## 模块系统
//! - `define-module` - 模块定义
//! - `import` - 模块导入
//! - `export` - 模块导出

// TODO: 实现特殊形式模块
// 
// 建议的文件结构：
// - basic.rs - 基本特殊形式（define, lambda, if, quote, set!）
// - control.rs - 控制流特殊形式（begin, cond, case, and, or, let）
// - iteration.rs - 迭代特殊形式（do, for-each, map, call/cc）
// - macros.rs - 宏系统（define-syntax, syntax-rules）
// - modules.rs - 模块系统（define-module, import, export）

pub mod basic;
pub mod control;
pub mod iteration;
pub mod macros;
pub mod modules;

// 重新导出所有特殊形式
pub use basic::*;
pub use control::*;
pub use iteration::*;
pub use macros::*;
pub use modules::*;
