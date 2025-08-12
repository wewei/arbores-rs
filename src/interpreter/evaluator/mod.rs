//! Evaluator 模块入口
//! 
//! 本模块导出求值器的主要接口，包括求值函数和相关类型。

pub mod types;
pub mod engine;
pub mod state;
pub mod function_call;
pub mod builtins;
pub mod special_forms;



// 重新导出核心类型
pub use types::{
    RuntimeValue, Environment, EvalState, Frame, Continuation, TailContext,
    EvaluateResult, EvaluateError, FunctionArity, BuiltinImpl,
};

// 重新导出主要接口
pub use engine::{evaluate, evaluate_step, evaluate_with_global_env};
