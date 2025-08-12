//! 特殊形式处理模块
//! 
//! 本模块包含所有 Scheme 特殊形式的实现，每个特殊形式独立一个文件。

pub mod quote;
pub mod if_form;
pub mod lambda;
pub mod define;
pub mod let_form;

// 重新导出所有特殊形式的求值函数
pub use quote::evaluate_quote;
pub use if_form::evaluate_if;
pub use lambda::evaluate_lambda;
pub use define::evaluate_define;
pub use let_form::evaluate_let;
