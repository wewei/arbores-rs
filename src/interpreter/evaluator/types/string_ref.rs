//! StringRef 结构定义
//! 
//! 使用 Rc 包装的字符串，支持共享和不可变访问

use std::rc::Rc;
use gc::{Trace, Finalize};

/// StringRef - 使用 Rc 包装的字符串，支持共享
#[derive(Debug, Clone, Trace, Finalize)]
pub struct StringRef {
    #[unsafe_ignore_trace]
    inner: Rc<String>,
}

impl StringRef {
    /// 创建新的 StringRef
    pub fn new(s: String) -> Self {
        Self { inner: Rc::new(s) }
    }
    
    /// 从字符串切片创建 StringRef
    pub fn from_str(s: &str) -> Self {
        Self { inner: Rc::new(s.to_string()) }
    }
    
    /// 获取字符串切片
    pub fn as_str(&self) -> &str {
        &self.inner
    }
    
    /// 获取字符串长度
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    /// 检查字符串是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::fmt::Display for StringRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl PartialEq for StringRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl From<String> for StringRef {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for StringRef {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}
