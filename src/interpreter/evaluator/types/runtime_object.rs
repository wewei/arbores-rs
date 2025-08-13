//! 运行时对象核心定义
//! 
//! 定义了 RuntimeObject 和 RuntimeObjectCore，这是整个求值器的核心数据结构

use std::rc::Rc;
use std::sync::Weak;

use crate::interpreter::SExpr;
use super::{MutableCons, MutableVector, Lambda, Continuation, BuiltinFunction};

// ============================================================================
// 核心数据结构定义
// ============================================================================

/// 运行时对象核心 - 表示运行时的所有可能对象类型
/// 按照引用类型分为四类：
/// 1. 原子值：integer, float, boolean, nil - 直接存储
/// 2. Rc 引用值：Rc<String> - 强引用，不可变内容
/// 3. Weak 引用值：Weak<BuiltinFunction> - 弱引用，避免循环引用
/// 4. GC 引用值：Gc<Cons>, Gc<Vector>, Gc<Lambda>, Gc<Continuation> - 垃圾回收，支持可变操作
#[derive(Debug, Clone)]
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
    String(Rc<String>),
    /// 符号 - Rc 引用值，不可变内容但可共享
    Symbol(Rc<String>),
    
    // === 3. Weak 引用值（Weak Reference Objects）- 弱引用 ===
    /// 内置函数 - Weak 引用值，避免循环引用
    BuiltinFunction(Weak<BuiltinFunction>),
    
    // === 4. Rc 引用值（Rc Reference Objects）- 强引用，支持可变操作 ===
    /// 可变列表（cons 结构）- Rc 引用值，支持可变操作
    Cons(Rc<MutableCons>),
    /// 可变向量 - Rc 引用值，支持可变操作
    Vector(Rc<MutableVector>),
    /// Lambda 函数 - Rc 引用值，支持环境可变
    Lambda(Rc<Lambda>),
    /// 续延 - Rc 引用值，支持 call/cc
    Continuation(Rc<Continuation>),
}

/// 运行时对象 - 包含核心对象和可选的源表达式
/// RuntimeObject 本身是一个比较小的对象，可以直接 Clone
#[derive(Debug, Clone)]
pub struct RuntimeObject {
    /// 核心运行时对象
    pub core: RuntimeObjectCore,
    /// 可选的源表达式，用于保存计算出该 RuntimeObject 的 SExpr
    pub source: Option<Rc<SExpr>>,
}

// ============================================================================
// 构造函数实现
// ============================================================================

impl RuntimeObject {
    /// 创建新的运行时对象
    pub fn new(core: RuntimeObjectCore) -> Self {
        Self {
            core,
            source: None,
        }
    }
    
    /// 创建带源表达式的运行时对象
    pub fn with_source(core: RuntimeObjectCore, source: Rc<SExpr>) -> Self {
        Self {
            core,
            source: Some(source),
        }
    }
    
    /// 获取核心对象
    pub fn core(&self) -> &RuntimeObjectCore {
        &self.core
    }
    
    /// 获取源表达式
    pub fn source(&self) -> Option<&Rc<SExpr>> {
        self.source.as_ref()
    }
    
    /// 获取对象类型名称
    pub fn type_name(&self) -> &'static str {
        match &self.core {
            RuntimeObjectCore::Integer(_) => "integer",
            RuntimeObjectCore::Float(_) => "float",
            RuntimeObjectCore::Rational(_, _) => "rational",
            RuntimeObjectCore::Character(_) => "character",
            RuntimeObjectCore::Boolean(_) => "boolean",
            RuntimeObjectCore::Nil => "null",
            RuntimeObjectCore::String(_) => "string",
            RuntimeObjectCore::Symbol(_) => "symbol",
            RuntimeObjectCore::BuiltinFunction(_) => "procedure",
            RuntimeObjectCore::Cons(_) => "pair",
            RuntimeObjectCore::Vector(_) => "vector",
            RuntimeObjectCore::Lambda(_) => "procedure",
            RuntimeObjectCore::Continuation(_) => "continuation",
        }
    }
    
    /// 检查是否为真值
    pub fn is_truthy(&self) -> bool {
        match &self.core {
            RuntimeObjectCore::Boolean(false) => false,
            _ => true,
        }
    }
    
    /// 设置 Cons 的 car 部分
    pub fn set_car(&self, value: RuntimeObject) -> Result<(), String> {
        match &self.core {
            RuntimeObjectCore::Cons(cons) => {
                cons.set_car(value);
                Ok(())
            },
            _ => Err(format!("Expected pair, got {}", self.type_name())),
        }
    }
    
    /// 设置 Cons 的 cdr 部分
    pub fn set_cdr(&self, value: RuntimeObject) -> Result<(), String> {
        match &self.core {
            RuntimeObjectCore::Cons(cons) => {
                cons.set_cdr(value);
                Ok(())
            },
            _ => Err(format!("Expected pair, got {}", self.type_name())),
        }
    }
    
    /// 设置向量元素
    pub fn vector_set(&self, index: usize, value: RuntimeObject) -> Result<(), String> {
        match &self.core {
            RuntimeObjectCore::Vector(vector) => {
                vector.set(index, value)
            },
            _ => Err(format!("Expected vector, got {}", self.type_name())),
        }
    }
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
            (RuntimeObjectCore::String(a), RuntimeObjectCore::String(b)) => Rc::ptr_eq(a, b),
            (RuntimeObjectCore::Symbol(a), RuntimeObjectCore::Symbol(b)) => Rc::ptr_eq(a, b),
            
            // Weak 引用值需要升级为强引用后比较
            (RuntimeObjectCore::BuiltinFunction(a), RuntimeObjectCore::BuiltinFunction(b)) => {
                if let (Some(ra), Some(rb)) = (a.upgrade(), b.upgrade()) {
                    std::ptr::eq(&*ra, &*rb)
                } else {
                    false
                }
            },
            
            // Rc 引用值比较引用是否相等
            (RuntimeObjectCore::Cons(a), RuntimeObjectCore::Cons(b)) => Rc::ptr_eq(a, b),
            (RuntimeObjectCore::Vector(a), RuntimeObjectCore::Vector(b)) => Rc::ptr_eq(a, b),
            (RuntimeObjectCore::Lambda(a), RuntimeObjectCore::Lambda(b)) => Rc::ptr_eq(a, b),
            (RuntimeObjectCore::Continuation(a), RuntimeObjectCore::Continuation(b)) => Rc::ptr_eq(a, b),
            
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
                if let Some(strong_ref) = builtin.upgrade() {
                    write!(f, "#<procedure:{}>", strong_ref.name)
                } else {
                    write!(f, "#<procedure:builtin>")
                }
            },
            RuntimeObjectCore::Continuation(_) => write!(f, "#<continuation>"),
        }
    }
}
