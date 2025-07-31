//! 字符流抽象层
//! 
//! 提供基于字符的前瞻和消费操作，专门用于词法分析。

use std::collections::VecDeque;

/// 支持多字符前瞻的字符流
/// 
/// 专门为词法分析设计的字符流抽象，提供高效的字符前瞻和消费操作。
pub struct CharStream<I: Iterator<Item = char>> {
    iter: I,
    buffer: VecDeque<char>,
}

impl<I: Iterator<Item = char>> CharStream<I> {
    /// 创建新的字符流
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            buffer: VecDeque::new(),
        }
    }

    /// 前瞻第 n 个字符（0 表示当前字符，1 表示下一个字符）
    /// 
    /// # 参数
    /// - `n`: 前瞻的字符索引，0 表示当前字符
    /// 
    /// # 返回值
    /// 返回第 n 个字符的引用，如果没有更多字符则返回 None
    pub fn peek(&mut self, n: usize) -> Option<&char> {
        // 确保缓冲区有足够的元素
        while self.buffer.len() <= n {
            if let Some(item) = self.iter.next() {
                self.buffer.push_back(item);
            } else {
                break;
            }
        }
        
        self.buffer.get(n)
    }

    /// 前瞻当前字符（等价于 peek(0)）
    pub fn peek_current(&mut self) -> Option<&char> {
        self.peek(0)
    }

    /// 前瞻下一个字符（等价于 peek(1)）
    pub fn peek_next(&mut self) -> Option<&char> {
        self.peek(1)
    }

    /// 消费一个字符
    pub fn advance(&mut self) -> Option<char> {
        if !self.buffer.is_empty() {
            self.buffer.pop_front()
        } else {
            self.iter.next()
        }
    }

    /// 消费 n 个字符，返回消费的字符序列
    pub fn advance_many(&mut self, n: usize) -> Vec<char> {
        let mut consumed = Vec::new();
        
        for _ in 0..n {
            if let Some(ch) = self.advance() {
                consumed.push(ch);
            } else {
                break;
            }
        }
        
        consumed
    }

    /// 检查是否还有更多字符
    pub fn has_more(&mut self) -> bool {
        self.peek_current().is_some()
    }

    /// 获取当前缓冲区大小（用于调试）
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_stream_creation() {
        let chars = "hello".chars();
        let mut stream = CharStream::new(chars);
        
        assert_eq!(stream.buffer_size(), 0);
        assert!(stream.has_more());
    }

    #[test]
    fn test_peek_operations() {
        let chars = "hello".chars();
        let mut stream = CharStream::new(chars);
        
        // 测试前瞻
        assert_eq!(stream.peek(0), Some(&'h'));
        assert_eq!(stream.peek(1), Some(&'e'));
        assert_eq!(stream.peek(2), Some(&'l'));
        
        // 前瞻不应该消费字符
        assert_eq!(stream.peek_current(), Some(&'h'));
        assert_eq!(stream.peek_next(), Some(&'e'));
    }

    #[test]
    fn test_advance_operations() {
        let chars = "hello".chars();
        let mut stream = CharStream::new(chars);
        
        // 测试单字符消费
        assert_eq!(stream.advance(), Some('h'));
        assert_eq!(stream.advance(), Some('e'));
        
        // 测试多字符消费
        let consumed = stream.advance_many(2);
        assert_eq!(consumed, vec!['l', 'l']);
        
        // 测试最后一个字符
        assert_eq!(stream.advance(), Some('o'));
        assert_eq!(stream.advance(), None);
    }

    #[test]
    fn test_peek_beyond_end() {
        let chars = "hi".chars();
        let mut stream = CharStream::new(chars);
        
        assert_eq!(stream.peek(0), Some(&'h'));
        assert_eq!(stream.peek(1), Some(&'i'));
        assert_eq!(stream.peek(2), None);
        assert_eq!(stream.peek(10), None);
    }

    #[test]
    fn test_advance_many_beyond_end() {
        let chars = "hi".chars();
        let mut stream = CharStream::new(chars);
        
        let consumed = stream.advance_many(5);
        assert_eq!(consumed, vec!['h', 'i']);
        
        assert!(!stream.has_more());
    }

    #[test]
    fn test_mixed_peek_and_advance() {
        let chars = "test".chars();
        let mut stream = CharStream::new(chars);
        
        // 前瞻几个字符
        assert_eq!(stream.peek(0), Some(&'t'));
        assert_eq!(stream.peek(1), Some(&'e'));
        
        // 消费一个字符
        assert_eq!(stream.advance(), Some('t'));
        
        // 再次前瞻应该看到新的当前字符
        assert_eq!(stream.peek(0), Some(&'e'));
        assert_eq!(stream.peek(1), Some(&'s'));
        
        // 消费剩余字符
        let remaining = stream.advance_many(10);
        assert_eq!(remaining, vec!['e', 's', 't']);
    }

    #[test]
    fn test_empty_stream() {
        let chars = "".chars();
        let mut stream = CharStream::new(chars);
        
        assert!(!stream.has_more());
        assert_eq!(stream.peek_current(), None);
        assert_eq!(stream.advance(), None);
        assert_eq!(stream.advance_many(5), vec![]);
    }
}
