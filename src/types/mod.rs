use std::fmt;
use std::rc::Rc;

/// Scheme 值的核心类型定义
#[derive(Debug, Clone)]
pub enum Value {
    /// 空值 (nil)
    Nil,
    /// 布尔值
    Bool(bool),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 符号
    Symbol(String),
    /// Cons 对 (列表的基本构造块)
    Cons(Rc<Value>, Rc<Value>),
    /// 内置函数
    BuiltinFunction {
        name: String,
        func: fn(&[Value]) -> Result<Value>,
        arity: Option<usize>, // None 表示可变参数
    },
    /// 用户定义的函数 (lambda)
    Lambda {
        params: Vec<String>,
        body: Rc<Value>,
        env: Rc<crate::env::Environment>, // 闭包环境
    },
}

impl Value {
    /// 检查值是否为真值（Scheme 中除了 #f 外都是真值）
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Bool(false))
    }

    /// 检查是否为空列表
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// 检查是否为列表（包括空列表）
    pub fn is_list(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Cons(_, cdr) => cdr.is_list(),
            _ => false,
        }
    }

    /// 将列表转换为 Vec（如果可能）
    pub fn to_vec(&self) -> Option<Vec<Value>> {
        let mut result = Vec::new();
        let mut current = self;
        
        loop {
            match current {
                Value::Nil => return Some(result),
                Value::Cons(car, cdr) => {
                    result.push((**car).clone());
                    current = cdr;
                },
                _ => return None, // 不是有效的列表
            }
        }
    }

    /// 从 Vec 创建列表
    pub fn from_vec(values: Vec<Value>) -> Value {
        values.into_iter().rev().fold(Value::Nil, |acc, val| {
            Value::Cons(Rc::new(val), Rc::new(acc))
        })
    }

    /// 获取列表长度
    pub fn length(&self) -> Option<usize> {
        self.to_vec().map(|v| v.len())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "()"),
            Value::Bool(true) => write!(f, "#t"),
            Value::Bool(false) => write!(f, "#f"),
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Cons(_, _) => {
                // 打印列表形式
                if let Some(vec) = self.to_vec() {
                    write!(f, "(")?;
                    for (i, val) in vec.iter().enumerate() {
                        if i > 0 { write!(f, " ")?; }
                        write!(f, "{}", val)?;
                    }
                    write!(f, ")")
                } else {
                    // 非正常列表（dotted pair）
                    write!(f, "({} . {})", self.car().unwrap(), self.cdr().unwrap())
                }
            },
            Value::BuiltinFunction { name, .. } => write!(f, "#<builtin:{}>", name),
            Value::Lambda { .. } => write!(f, "#<procedure>"),
        }
    }
}

impl Value {
    /// 获取 cons 对的 car
    pub fn car(&self) -> Option<&Value> {
        match self {
            Value::Cons(car, _) => Some(car),
            _ => None,
        }
    }

    /// 获取 cons 对的 cdr
    pub fn cdr(&self) -> Option<&Value> {
        match self {
            Value::Cons(_, cdr) => Some(cdr),
            _ => None,
        }
    }
}

/// 错误类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum SchemeError {
    /// 语法错误
    SyntaxError(String),
    /// 运行时错误
    RuntimeError(String),
    /// 类型错误
    TypeError(String),
    /// 未定义的变量
    UndefinedVariable(String),
    /// 参数个数错误
    ArityError(String),
    /// 除零错误
    DivisionByZero,
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemeError::SyntaxError(msg) => write!(f, "Syntax Error: {}", msg),
            SchemeError::RuntimeError(msg) => write!(f, "Runtime Error: {}", msg),
            SchemeError::TypeError(msg) => write!(f, "Type Error: {}", msg),
            SchemeError::UndefinedVariable(var) => write!(f, "Undefined Variable: {}", var),
            SchemeError::ArityError(msg) => write!(f, "Arity Error: {}", msg),
            SchemeError::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}

impl std::error::Error for SchemeError {}

pub type Result<T> = std::result::Result<T, SchemeError>;

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Cons(a1, a2), Value::Cons(b1, b2)) => a1 == b1 && a2 == b2,
            (Value::BuiltinFunction { name: n1, .. }, Value::BuiltinFunction { name: n2, .. }) => n1 == n2,
            // Lambda functions are compared by identity (always false for different instances)
            (Value::Lambda { .. }, Value::Lambda { .. }) => false,
            _ => false,
        }
    }
}
