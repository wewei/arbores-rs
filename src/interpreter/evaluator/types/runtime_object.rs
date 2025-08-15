//! 运行时对象核心定义
//! 
//! 定义了 RuntimeObject 和 RuntimeObjectCore，这是整个求值器的核心数据结构

use std::rc::Rc;
use gc::{Trace, Finalize};

use crate::interpreter::SExpr;
use super::{MutableCons, MutableVector, Continuation, BuiltinFunction, Lambda, StringRef};

// ============================================================================
// 核心数据结构定义
// ============================================================================

/// 运行时对象核心 - 表示运行时的所有可能对象类型
/// 按照引用类型分为四类：
/// 1. 原子值：integer, float, boolean, nil - 直接存储
/// 2. Rc 引用值：Rc<String> - 强引用，不可变内容
/// 3. 直接嵌入值：Lambda - 直接存储，避免间接引用
#[derive(Debug, Clone, Trace, Finalize)]
pub enum RuntimeObjectCore {
    // === 1. 原子值（Atomic Objects）- 直接存储 ===
    /// 整数 - 原子值，直接存储
    Integer(i64),
    /// 浮点数 - 原子值，直接存储
    Float(f64),
    /// 有理数 - 原子值，直接存储
    Rational(i64, i64),  // 分子, 分母
    /// 字符 - 原子值，直接存储
    Character(char),
    /// 布尔值 - 原子值，直接存储
    Boolean(bool),
    /// 空列表 - 原子值，直接存储
    Nil,
    
    // === 2. Rc 引用值（Rc Reference Objects）- 强引用 ===
    /// 字符串 - Rc 引用值，不可变内容但可共享
    String(StringRef),
    /// 符号 - Rc 引用值，不可变内容但可共享
    Symbol(StringRef),
    
    // === 3. 直接嵌入值（Direct Embedded Objects）- 直接存储 ===
    /// 内置函数 - 直接嵌入，16 bytes
    BuiltinFunction(BuiltinFunction),
    
    /// 可变列表（cons 结构）- Gc 引用值，支持可变操作
    Cons(MutableCons),
    /// 可变向量 - Gc 引用值，支持可变操作
    Vector(MutableVector),
    /// 续延 - Gc 引用值，支持 call/cc
    Continuation(Continuation),
    
    /// Lambda 函数 - 直接嵌入，16 bytes
    Lambda(Lambda),
}










/// 运行时对象 - 包含核心对象和可选的源表达式
/// RuntimeObject 本身是一个比较小的对象，可以直接 Clone
#[derive(Debug, Clone, Trace, Finalize)]
pub struct RuntimeObject {
    /// 核心运行时对象
    pub core: RuntimeObjectCore,
    /// 可选的源表达式，用于保存计算出该 RuntimeObject 的 SExpr
    #[unsafe_ignore_trace]
    pub source: Option<Rc<SExpr>>,
}

// ============================================================================
// PartialEq 实现
// ============================================================================

impl PartialEq for RuntimeObject {
    fn eq(&self, other: &Self) -> bool {
        match (&self.core, &other.core) {
            // 原子值直接比较
            (RuntimeObjectCore::Integer(a), RuntimeObjectCore::Integer(b)) => a == b,
            (RuntimeObjectCore::Float(a), RuntimeObjectCore::Float(b)) => a == b,
            (RuntimeObjectCore::Rational(a1, a2), RuntimeObjectCore::Rational(b1, b2)) => a1 == b1 && a2 == b2,
            (RuntimeObjectCore::Character(a), RuntimeObjectCore::Character(b)) => a == b,
            (RuntimeObjectCore::Boolean(a), RuntimeObjectCore::Boolean(b)) => a == b,
            (RuntimeObjectCore::Nil, RuntimeObjectCore::Nil) => true,
            
            // Rc 引用值比较引用是否相等
            (RuntimeObjectCore::String(a), RuntimeObjectCore::String(b)) => a == b,
            (RuntimeObjectCore::Symbol(a), RuntimeObjectCore::Symbol(b)) => a == b,
            
            // 直接嵌入值比较内容是否相等
            (RuntimeObjectCore::BuiltinFunction(a), RuntimeObjectCore::BuiltinFunction(b)) => a == b,
            
            // 直接嵌入值比较内容是否相等
            (RuntimeObjectCore::Cons(a), RuntimeObjectCore::Cons(b)) => a == b,
            (RuntimeObjectCore::Vector(a), RuntimeObjectCore::Vector(b)) => a == b,
            (RuntimeObjectCore::Lambda(a), RuntimeObjectCore::Lambda(b)) => a == b,
            (RuntimeObjectCore::Continuation(a), RuntimeObjectCore::Continuation(b)) => a == b,
            
            _ => false,
        }
    }
}

// ============================================================================
// Display 实现 - 用于错误报告和调试
// ============================================================================

impl std::fmt::Display for RuntimeObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.core {
            RuntimeObjectCore::Integer(n) => write!(f, "{}", n),
            RuntimeObjectCore::Float(n) => write!(f, "{}", n),
            RuntimeObjectCore::Rational(num, den) => write!(f, "{}/{}", num, den),
            RuntimeObjectCore::Character(c) => write!(f, "#\\{}", c),
            RuntimeObjectCore::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            RuntimeObjectCore::Nil => write!(f, "()"),
            RuntimeObjectCore::String(s) => write!(f, "\"{}\"", s),
            RuntimeObjectCore::Symbol(s) => write!(f, "{}", s),
            RuntimeObjectCore::Cons(cons) => {
                write!(f, "({}", cons.car())?;
                let mut current = cons.cdr();
                loop {
                    match &current.core {
                        RuntimeObjectCore::Nil => break,
                        RuntimeObjectCore::Cons(next_cons) => {
                            write!(f, " {}", next_cons.car())?;
                            current = next_cons.cdr();
                        },
                        _ => {
                            write!(f, " . {}", current)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            },
            RuntimeObjectCore::Vector(vector) => {
                write!(f, "#(")?;
                let elements = vector.to_vec();
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, ")")
            },
            RuntimeObjectCore::Lambda(_) => write!(f, "#<procedure>"),
            RuntimeObjectCore::BuiltinFunction(builtin) => {
                write!(f, "#<procedure:{}>", builtin.name())
            },
            RuntimeObjectCore::Continuation(_) => write!(f, "#<continuation>"),
        }
    }
}


