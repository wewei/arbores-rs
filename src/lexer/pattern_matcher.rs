//! 模式匹配器
//! 
//! 基于 CharStream 和 Pattern 实现词法模式匹配，返回匹配的字符串或失配结果。

use crate::lexer::char_stream::CharStream;
use crate::lexer::types::Pattern;

/// 模式匹配结果
#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
    /// 成功匹配，包含匹配的字符串
    Success(String),
    /// 匹配失败
    Failure,
}

/// 基于 CharStream 进行模式匹配
/// 
/// # 参数
/// - `stream`: 字符流
/// - `pattern`: 要匹配的模式
/// 
/// # 返回值
/// 返回匹配结果，如果成功则包含匹配的字符串，失败则返回 Failure
pub fn match_pattern<I: Iterator<Item = char>>(
    stream: &mut CharStream<I>, 
    pattern: &Pattern
) -> MatchResult {
    match pattern {
        Pattern::Char(expected_char) => match_char(stream, *expected_char),
        Pattern::String(expected_str) => match_string(stream, expected_str),
        Pattern::CharClass(predicate) => match_char_class(stream, predicate),
    }
}

/// 匹配单个字符
fn match_char<I: Iterator<Item = char>>(
    stream: &mut CharStream<I>, 
    expected: char
) -> MatchResult {
    if let Some(&current_char) = stream.peek_current() {
        if current_char == expected {
            stream.advance();
            MatchResult::Success(expected.to_string())
        } else {
            MatchResult::Failure
        }
    } else {
        MatchResult::Failure
    }
}

/// 匹配字符串常量
fn match_string<I: Iterator<Item = char>>(
    stream: &mut CharStream<I>, 
    expected: &str
) -> MatchResult {
    let expected_chars: Vec<char> = expected.chars().collect();
    
    // 先检查是否能完全匹配，不消费字符
    for (i, &expected_char) in expected_chars.iter().enumerate() {
        if let Some(&current_char) = stream.peek(i) {
            if current_char != expected_char {
                return MatchResult::Failure;
            }
        } else {
            return MatchResult::Failure;
        }
    }
    
    // 如果检查通过，消费所有匹配的字符
    let consumed = stream.advance_many(expected_chars.len());
    let matched_string: String = consumed.into_iter().collect();
    
    MatchResult::Success(matched_string)
}

/// 匹配字符类（符合某个条件的单个字符）
fn match_char_class<I: Iterator<Item = char>>(
    stream: &mut CharStream<I>, 
    predicate: &fn(char) -> bool
) -> MatchResult {
    if let Some(&current_char) = stream.peek_current() {
        if predicate(current_char) {
            stream.advance();
            MatchResult::Success(current_char.to_string())
        } else {
            MatchResult::Failure
        }
    } else {
        MatchResult::Failure
    }
}

/// 连续匹配字符类，直到遇到不匹配的字符
/// 
/// 这是一个扩展功能，用于匹配连续的相同类型字符（如数字、字母等）
pub fn match_char_class_sequence<I: Iterator<Item = char>, F>(
    stream: &mut CharStream<I>, 
    predicate: F
) -> MatchResult 
where 
    F: Fn(char) -> bool
{
    let mut matched_chars = Vec::new();
    
    // 连续匹配符合条件的字符
    while let Some(&current_char) = stream.peek_current() {
        if predicate(current_char) {
            matched_chars.push(current_char);
            stream.advance();
        } else {
            break;
        }
    }
    
    if matched_chars.is_empty() {
        MatchResult::Failure
    } else {
        let matched_string: String = matched_chars.into_iter().collect();
        MatchResult::Success(matched_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_char_success() {
        let chars = "hello".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_pattern(&mut stream, &Pattern::Char('h'));
        assert_eq!(result, MatchResult::Success("h".to_string()));
        
        // 检查字符已被消费
        assert_eq!(stream.peek_current(), Some(&'e'));
    }

    #[test]
    fn test_match_char_failure() {
        let chars = "hello".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_pattern(&mut stream, &Pattern::Char('x'));
        assert_eq!(result, MatchResult::Failure);
        
        // 检查字符未被消费
        assert_eq!(stream.peek_current(), Some(&'h'));
    }

    #[test]
    fn test_match_string_success() {
        let chars = "hello world".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_pattern(&mut stream, &Pattern::String("hello"));
        assert_eq!(result, MatchResult::Success("hello".to_string()));
        
        // 检查正确的字符被消费
        assert_eq!(stream.peek_current(), Some(&' '));
    }

    #[test]
    fn test_match_string_failure() {
        let chars = "hello world".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_pattern(&mut stream, &Pattern::String("hi"));
        assert_eq!(result, MatchResult::Failure);
        
        // 检查没有字符被消费
        assert_eq!(stream.peek_current(), Some(&'h'));
    }

    #[test]
    fn test_match_string_partial_failure() {
        let chars = "help".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_pattern(&mut stream, &Pattern::String("hello"));
        assert_eq!(result, MatchResult::Failure);
        
        // 检查没有字符被消费
        assert_eq!(stream.peek_current(), Some(&'h'));
    }

    #[test]
    fn test_match_char_class_success() {
        let chars = "123abc".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_pattern(&mut stream, &Pattern::CharClass(|c| c.is_ascii_digit()));
        assert_eq!(result, MatchResult::Success("1".to_string()));
        
        // 检查正确的字符被消费
        assert_eq!(stream.peek_current(), Some(&'2'));
    }

    #[test]
    fn test_match_char_class_failure() {
        let chars = "abc123".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_pattern(&mut stream, &Pattern::CharClass(|c| c.is_ascii_digit()));
        assert_eq!(result, MatchResult::Failure);
        
        // 检查没有字符被消费
        assert_eq!(stream.peek_current(), Some(&'a'));
    }

    #[test]
    fn test_match_char_class_sequence() {
        let chars = "123abc".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_char_class_sequence(&mut stream, |c| c.is_ascii_digit());
        assert_eq!(result, MatchResult::Success("123".to_string()));
        
        // 检查正确的字符被消费
        assert_eq!(stream.peek_current(), Some(&'a'));
    }

    #[test]
    fn test_match_char_class_sequence_empty() {
        let chars = "abc123".chars();
        let mut stream = CharStream::new(chars);
        
        let result = match_char_class_sequence(&mut stream, |c| c.is_ascii_digit());
        assert_eq!(result, MatchResult::Failure);
        
        // 检查没有字符被消费
        assert_eq!(stream.peek_current(), Some(&'a'));
    }

    #[test]
    fn test_match_at_end_of_stream() {
        let chars = "h".chars();
        let mut stream = CharStream::new(chars);
        
        // 成功匹配唯一的字符
        let result = match_pattern(&mut stream, &Pattern::Char('h'));
        assert_eq!(result, MatchResult::Success("h".to_string()));
        
        // 尝试在流结束时匹配应该失败
        let result = match_pattern(&mut stream, &Pattern::Char('i'));
        assert_eq!(result, MatchResult::Failure);
    }
}
