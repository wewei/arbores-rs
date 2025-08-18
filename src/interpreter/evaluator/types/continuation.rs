//! 续延结构定义
//! 
//! 支持 call/cc 的续延结构

use gc::{Finalize, Gc, Trace};
use super::{RuntimeObject, EvaluateResult, EvaluateError};

/// 续延类型枚举 - 支持不同的续延实现
#[derive(Clone, Trace, Finalize)]
pub enum Continuation {
    /// 根续延
    Root,
    /// 函数求值续延
    FunctionEval {
        frame: Gc<super::Frame>,
        operands: Gc<RuntimeObject>,
    },
    /// 参数求值续延
    ArgumentEval {
        frame: Gc<super::Frame>,
        function_value: Gc<RuntimeObject>,
        remaining_args: Gc<RuntimeObject>,
        evaluated_args: Vec<Gc<RuntimeObject>>,
    },
    /// 特殊形式续延
    SpecialForm {
        frame: Gc<super::Frame>,
        form_type: String,
        args: Gc<RuntimeObject>,
    },
}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::fmt::Debug for Continuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Continuation::Root => write!(f, "Continuation::Root"),
            Continuation::FunctionEval { .. } => write!(f, "Continuation::FunctionEval"),
            Continuation::ArgumentEval { .. } => write!(f, "Continuation::ArgumentEval"),
            Continuation::SpecialForm { form_type, .. } => write!(f, "Continuation::SpecialForm({})", form_type),
        }
    }
}

impl Continuation {
    /// 调用续延
    pub fn call(&self, value: Gc<RuntimeObject>) -> EvaluateResult {
        match self {
            Continuation::Root => {
                EvaluateResult::Completed(std::rc::Rc::new(value.as_ref().clone()))
            },
            Continuation::FunctionEval { frame, operands } => {
                // 当函数求值完成后，开始求值参数
                evaluate_arguments(frame.clone(), value, operands, Vec::new())
            },
            Continuation::ArgumentEval { frame, function_value, remaining_args, evaluated_args } => {
                // 参数求值完成，继续求值下一个参数
                let mut new_evaluated_args = evaluated_args.clone();
                new_evaluated_args.push(value);
                evaluate_arguments(frame.clone(), function_value.clone(), remaining_args, new_evaluated_args)
            },
            Continuation::SpecialForm { frame, form_type, args } => {
                // 特殊形式求值
                evaluate_special_form(frame.clone(), form_type, args, value)
            },
        }
    }
}

/// 求值函数参数
fn evaluate_arguments(
    frame: Gc<super::Frame>,
    function_value: Gc<RuntimeObject>,
    remaining_args: &Gc<RuntimeObject>,
    evaluated_args: Vec<Gc<RuntimeObject>>,
) -> EvaluateResult {
    // 暂时不支持参数求值，返回错误
    EvaluateResult::Error(EvaluateError::NotImplemented {
        expr: function_value.source.clone().unwrap_or_else(|| {
            std::rc::Rc::new(crate::interpreter::SExpr::without_span(
                crate::interpreter::SExprContent::Atom(crate::interpreter::Value::Symbol("function".to_string()))
            ))
        }),
        feature: "argument evaluation".to_string(),
    })
}

/// 求值特殊形式
fn evaluate_special_form(
    frame: Gc<super::Frame>,
    form_type: &str,
    args: &Gc<RuntimeObject>,
    value: Gc<RuntimeObject>,
) -> EvaluateResult {
    // 暂时不支持特殊形式求值，返回错误
    EvaluateResult::Error(EvaluateError::NotImplemented {
        expr: value.source.clone().unwrap_or_else(|| {
            std::rc::Rc::new(crate::interpreter::SExpr::without_span(
                crate::interpreter::SExprContent::Atom(crate::interpreter::Value::Symbol(form_type.to_string()))
            ))
        }),
        feature: format!("special form: {}", form_type),
    })
}
