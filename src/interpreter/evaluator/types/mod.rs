//! 求值器核心数据类型定义
//! 
//! 本模块定义了求值器的核心数据结构，遵循 RuntimeObject 设计：
//! - 四种分类：原子值、Rc引用值、Weak引用值、GC引用值
//! - 支持可变操作和垃圾回收
//! - 明确的内存管理策略

pub mod runtime_object;
pub mod lambda;
pub mod builtin_function;
pub mod mutable_cons;
pub mod mutable_vector;
pub mod environment;
pub mod frame;
pub mod continuation;
pub mod evaluation_error;
pub mod evaluation_result;
pub mod eval_state;

pub use runtime_object::*;
pub use lambda::*;
pub use builtin_function::*;
pub use mutable_cons::*;
pub use mutable_vector::*;
pub use environment::*;
pub use frame::*;
pub use continuation::*;
pub use evaluation_error::*;
pub use evaluation_result::*;
pub use eval_state::*;

// ============================================================================
// 兼容性类型别名（用于过渡）
// ============================================================================

/// 兼容性类型别名，用于过渡期间
pub type RuntimeValue = RuntimeObject;
