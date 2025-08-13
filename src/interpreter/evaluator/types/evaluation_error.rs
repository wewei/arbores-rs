//! 求值错误类型定义
//! 
//! 表示求值过程中可能出现的各种错误

use std::rc::Rc;
use crate::interpreter::SExpr;

/// 求值错误类型 - 表示求值过程中可能出现的各种错误
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluateError {
    // 语法错误
    InvalidQuoteSyntax {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidIfSyntax {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidLambdaSyntax {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidDefineSyntax {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidLetSyntax {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidLetBinding {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidParameterName {
        expr: Rc<SExpr>,
        name: String,
    },
    InvalidParameterList {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidArgumentList {
        expr: Rc<SExpr>,
        message: String,
    },
    InvalidExpression {
        expr: Rc<SExpr>,
        message: String,
    },
    
    // 运行时错误
    UndefinedVariable {
        expr: Rc<SExpr>,
        name: String,
    },
    UndefinedFunction {
        expr: Rc<SExpr>,
        name: String,
    },
    NotCallable {
        expr: Rc<SExpr>,
        value: String, // 尝试调用的值的字符串表示
    },
    ArgumentCountMismatch {
        expr: Rc<SExpr>,
        expected: String, // 期望的参数个数描述
        actual: usize,    // 实际的参数个数
    },
    DivisionByZero {
        expr: Rc<SExpr>,
    },
    TypeMismatch {
        expr: Rc<SExpr>,
        expected: String,
        actual: String,
    },
    TypeError {
        expr: Rc<SExpr>,
        expected: String,
        actual: String,
    },
    
    // 系统错误
    StackOverflow {
        expr: Rc<SExpr>,
    },
    OutOfMemory {
        expr: Rc<SExpr>,
    },
    InternalError {
        expr: Rc<SExpr>,
        message: String,
    },
    NotImplemented {
        expr: Rc<SExpr>,
        feature: String,
    },
}

// ============================================================================
// Display 实现 - 用于错误报告和调试
// ============================================================================

impl std::fmt::Display for EvaluateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluateError::UndefinedVariable { name, .. } => {
                write!(f, "Undefined variable: {}", name)
            },
            EvaluateError::UndefinedFunction { name, .. } => {
                write!(f, "Undefined function: {}", name)
            },
            EvaluateError::NotCallable { value, .. } => {
                write!(f, "Value is not callable: {}", value)
            },
            EvaluateError::ArgumentCountMismatch { expected, actual, .. } => {
                write!(f, "Argument count mismatch: expected {}, got {}", expected, actual)
            },
            EvaluateError::DivisionByZero { .. } => {
                write!(f, "Division by zero")
            },
            EvaluateError::TypeMismatch { expected, actual, .. } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            },
            EvaluateError::TypeError { expected, actual, .. } => {
                write!(f, "Type error: expected {}, got {}", expected, actual)
            },
            EvaluateError::InvalidQuoteSyntax { message, .. } => {
                write!(f, "Invalid quote syntax: {}", message)
            },
            EvaluateError::InvalidIfSyntax { message, .. } => {
                write!(f, "Invalid if syntax: {}", message)
            },
            EvaluateError::InvalidLambdaSyntax { message, .. } => {
                write!(f, "Invalid lambda syntax: {}", message)
            },
            EvaluateError::InvalidDefineSyntax { message, .. } => {
                write!(f, "Invalid define syntax: {}", message)
            },
            EvaluateError::InvalidLetSyntax { message, .. } => {
                write!(f, "Invalid let syntax: {}", message)
            },
            EvaluateError::InvalidLetBinding { message, .. } => {
                write!(f, "Invalid let binding: {}", message)
            },
            EvaluateError::InvalidParameterName { name, .. } => {
                write!(f, "Invalid parameter name: {}", name)
            },
            EvaluateError::InvalidParameterList { message, .. } => {
                write!(f, "Invalid parameter list: {}", message)
            },
            EvaluateError::InvalidArgumentList { message, .. } => {
                write!(f, "Invalid argument list: {}", message)
            },
            EvaluateError::InvalidExpression { message, .. } => {
                write!(f, "Invalid expression: {}", message)
            },
            EvaluateError::StackOverflow { .. } => {
                write!(f, "Stack overflow")
            },
            EvaluateError::OutOfMemory { .. } => {
                write!(f, "Out of memory")
            },
            EvaluateError::InternalError { message, .. } => {
                write!(f, "Internal error: {}", message)
            },
            EvaluateError::NotImplemented { feature, .. } => {
                write!(f, "Feature not implemented: {}", feature)
            },
        }
    }
}

impl std::error::Error for EvaluateError {}
