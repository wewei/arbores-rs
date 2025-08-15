//! 内置函数模块
//! 
//! 本模块包含所有内置函数的实现，按照功能分类组织：
//! 
//! TODO: 实现以下内置函数分类
//! 
//! ## 算术函数
//! - `+`, `-`, `*`, `/` - 基本算术运算
//! - `mod`, `quotient`, `remainder` - 整数运算
//! - `abs`, `max`, `min` - 数值函数
//! 
//! ## 比较函数
//! - `=`, `<`, `>`, `<=`, `>=` - 数值比较
//! - `eq?`, `equal?` - 相等性比较
//! 
//! ## 列表函数
//! - `car`, `cdr`, `cons` - 基本列表操作
//! - `list`, `append`, `reverse` - 列表构造
//! - `length`, `null?`, `pair?` - 列表检查
//! 
//! ## 向量函数
//! - `vector`, `vector-ref`, `vector-set!` - 向量操作
//! - `vector-length`, `vector?` - 向量检查
//! 
//! ## 字符串函数
//! - `string-append`, `string-length` - 字符串操作
//! - `string-ref`, `string-set!` - 字符串访问
//! - `string=?`, `string<?` - 字符串比较
//! 
//! ## 类型检查函数
//! - `number?`, `string?`, `symbol?` - 类型检查
//! - `procedure?`, `boolean?` - 类型检查
//! 
//! ## 输入输出函数
//! - `display`, `write`, `newline` - 输出函数
//! - `read`, `read-char` - 输入函数
//! 
//! ## 系统函数
//! - `load`, `eval`, `apply` - 系统函数
//! - `error`, `exit` - 错误处理

// TODO: 实现内置函数模块
// 
// 建议的文件结构：
// - arithmetic.rs - 算术函数
// - comparison.rs - 比较函数  
// - list.rs - 列表函数
// - vector.rs - 向量函数
// - string.rs - 字符串函数
// - type_check.rs - 类型检查函数
// - io.rs - 输入输出函数
// - system.rs - 系统函数

pub mod arithmetic;
pub mod comparison;
pub mod list;
pub mod vector;
pub mod string;
pub mod type_check;
pub mod io;
pub mod system;

// 重新导出所有内置函数
pub use arithmetic::*;
pub use comparison::*;
pub use list::*;
pub use vector::*;
pub use string::*;
pub use type_check::*;
pub use io::*;
pub use system::*;
