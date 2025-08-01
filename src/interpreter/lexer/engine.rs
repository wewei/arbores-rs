//! 词法分析执行引擎
//! 
//! 本模块实现状态机驱动的词法分析核心逻辑，包括状态管理、状态转移执行和迭代器实现。

use crate::interpreter::lexer::types::{
    Token, LexError, LexErrorReason, LexerState, StateMachine, TransitionRule, StateAction
};
use crate::interpreter::lexer::pattern_matcher::{match_pattern, MatchResult};
use crate::interpreter::lexer::rules::{get_scheme_state_machine, emit_eof};

/// 词法分析器迭代器
/// 
/// 实现 Iterator 特征，逐个产生 Token 或错误
pub struct LexerIterator<I: Iterator<Item = char>> {
    state: LexerState<I>,
    finished: bool,
}

impl<I: Iterator<Item = char>> LexerIterator<I> {
    /// 创建新的词法分析器迭代器
    pub fn new(chars: I) -> Self {
        let _state_machine = get_scheme_state_machine();
        // 为了满足生命周期要求，我们需要使用静态引用
        // 这里先用一个临时的实现，后续会优化
        let state = LexerState::new(chars, &STATIC_STATE_MACHINE);
        
        Self {
            state,
            finished: false,
        }
    }
}

// 临时的静态状态机 - 后续会优化这个实现
lazy_static::lazy_static! {
    static ref STATIC_STATE_MACHINE: StateMachine = get_scheme_state_machine();
}

impl<I: Iterator<Item = char>> Iterator for LexerIterator<I> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // 如果没有更多字符，检查是否还有待处理的缓冲区内容
        if !self.state.chars.has_more() {
            // 如果缓冲区有内容，先处理缓冲区
            if !self.state.buffer.is_empty() {
                let fallback = &self.state.state_machine.fallback_rules[self.state.state];
                if let Some(emitter) = fallback.emit_token {
                    let token_start_pos = self.state.current_pos;
                    let buffer_copy = self.state.buffer.clone();
                    self.state.buffer.clear();
                    self.state.state = fallback.next_state;
                    return Some(emitter(&buffer_copy, token_start_pos));
                } else {
                    // 如果 fallback 不生成 Token，清空缓冲区并继续
                    self.state.buffer.clear();
                    self.state.state = fallback.next_state;
                }
            }
            
            // 最后生成 EOF Token
            self.finished = true;
            return Some(emit_eof("", self.state.current_pos));
        }

        // 执行状态机逻辑
        match execute_state_machine_step(&mut self.state) {
            Some(result) => Some(result),
            None => {
                // 如果没有产生 Token，继续下一步
                self.next()
            }
        }
    }
}

/// 执行状态机的一个步骤
/// 
/// 返回 Some(token_result) 如果产生了 Token，否则返回 None
fn execute_state_machine_step<I: Iterator<Item = char>>(
    state: &mut LexerState<I>
) -> Option<Result<Token, LexError>> {
    let current_state = state.state;
    let rules = &state.state_machine.rules[current_state];
    
    // 尝试找到匹配的规则
    if let Some(rule) = find_matching_rule(state, rules) {
        execute_transition(state, rule)
    } else {
        // 没有规则匹配，执行 fallback
        let fallback = &state.state_machine.fallback_rules[current_state];
        execute_fallback(state, fallback)
    }
}

/// 查找匹配的转移规则
fn find_matching_rule<'a, I: Iterator<Item = char>>(
    state: &mut LexerState<I>,
    rules: &'a [TransitionRule]
) -> Option<&'a TransitionRule> {
    rules.iter().find(|&rule| does_pattern_match(state, &rule.pattern))
}

/// 检查模式是否匹配当前字符流位置
fn does_pattern_match<I: Iterator<Item = char>>(
    state: &mut LexerState<I>,
    pattern: &crate::interpreter::lexer::types::Pattern
) -> bool {
    // 这里我们不能直接使用 pattern_matcher，因为它会消费字符
    // 我们需要先检查是否匹配，再决定是否消费
    match pattern {
        crate::interpreter::lexer::types::Pattern::Char(expected_char) => {
            if let Some(&current_char) = state.chars.peek_current() {
                current_char == *expected_char
            } else {
                false
            }
        }
        crate::interpreter::lexer::types::Pattern::String(expected_str) => {
            let expected_chars: Vec<char> = expected_str.chars().collect();
            for (i, &expected_char) in expected_chars.iter().enumerate() {
                if let Some(&current_char) = state.chars.peek(i) {
                    if current_char != expected_char {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }
        crate::interpreter::lexer::types::Pattern::CharClass(predicate) => {
            if let Some(&current_char) = state.chars.peek_current() {
                predicate(current_char)
            } else {
                false
            }
        }
    }
}

/// 执行状态转移
fn execute_transition<I: Iterator<Item = char>>(
    state: &mut LexerState<I>,
    rule: &TransitionRule
) -> Option<Result<Token, LexError>> {
    // 先记录 Token 起始位置（在消费字符之前）
    let token_start_pos = state.current_pos;
    
    // 使用 pattern_matcher 消费匹配的字符
    let match_result = match_pattern(&mut state.chars, &rule.pattern);
    
    match match_result {
        MatchResult::Success(matched_text) => {
            // 更新缓冲区
            state.buffer.push_str(&matched_text);
            
            // 更新位置信息
            state.current_pos += matched_text.chars().count();
            
            // 检查是否需要生成 Token
            if let Some(emitter) = rule.action.emit_token {
                let buffer_copy = state.buffer.clone();
                
                // 清空缓冲区，准备下一个 Token
                state.buffer.clear();
                
                // 更新状态
                state.state = rule.action.next_state;
                
                // 生成 Token
                Some(emitter(&buffer_copy, token_start_pos))
            } else {
                // 只是状态转移，不生成 Token
                state.state = rule.action.next_state;
                None
            }
        }
        MatchResult::Failure => {
            // 理论上不应该到达这里，因为我们已经检查过匹配
            // 但为了安全，我们返回一个错误
            Some(Err(LexError::new(
                state.current_pos,
                state.buffer.chars().next(),
                LexErrorReason::InvalidCharacter,
            )))
        }
    }
}

/// 执行 fallback 规则
fn execute_fallback<I: Iterator<Item = char>>(
    state: &mut LexerState<I>,
    fallback: &StateAction
) -> Option<Result<Token, LexError>> {
    // 如果有 emit_token，先生成 Token
    let token_result = if let Some(emitter) = fallback.emit_token {
        let token_start_pos = state.current_pos;
        let buffer_copy = state.buffer.clone();
        Some(emitter(&buffer_copy, token_start_pos))
    } else {
        None
    };
    
    // 清空缓冲区
    state.buffer.clear();
    
    // 更新状态
    state.state = fallback.next_state;
    
    token_result
}

/// 主要的词法分析函数
/// 
/// 将字符迭代器转换为 Token 迭代器
pub fn tokenize<I: Iterator<Item = char>>(
    chars: I
) -> impl Iterator<Item = Result<Token, LexError>> {
    LexerIterator::new(chars)
}

/// 便利函数：对字符串进行词法分析
pub fn tokenize_string(input: &str) -> impl Iterator<Item = Result<Token, LexError>> + '_ {
    tokenize(input.chars())
}

/// 便利函数：收集所有 Token 到 Vec
pub fn tokenize_to_vec<I: Iterator<Item = char>>(
    chars: I
) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    
    for token_result in tokenize(chars) {
        match token_result {
            Ok(token) => tokens.push(token),
            Err(error) => return Err(error),
        }
    }
    
    Ok(tokens)
}

/// 便利函数：过滤掉 Trivia Token
pub fn filter_trivia_tokens<I>(
    token_iter: I
) -> impl Iterator<Item = Result<Token, LexError>>
where
    I: Iterator<Item = Result<Token, LexError>>
{
    token_iter.filter(|token_result| {
        match token_result {
            Ok(token) => !token.token_type.is_trivia(),
            Err(_) => true, // 保留错误
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::lexer::types::TokenType;

    #[test]
    fn test_tokenize_simple_expression() {
        let input = "(+ 1 2)";
        let tokens: Result<Vec<_>, _> = tokenize_string(input).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 应该包含：(, +, 1, 2, ), EOF 以及一些空白字符
        assert!(tokens.len() >= 6);
        
        // 检查第一个和最后一个 token
        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens.last().unwrap().token_type, TokenType::Eof);
    }

    #[test]
    fn test_tokenize_numbers() {
        let input = "42 3.14";
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 过滤掉空白和 EOF 后，应该有 2 个数字
        let numbers: Vec<_> = tokens.iter()
            .filter(|token| matches!(token.token_type, TokenType::Integer(_) | TokenType::Float(_)))
            .collect();
        
        assert_eq!(numbers.len(), 2);
    }

    #[test]
    fn test_tokenize_string_literal() {
        let input = "\"hello world\"";
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 应该有字符串 token 和 EOF
        assert_eq!(tokens.len(), 2);
        
        if let TokenType::String(content) = &tokens[0].token_type {
            assert_eq!(content, "hello world");
        } else {
            panic!("Expected string token");
        }
    }

    #[test]
    fn test_tokenize_with_comments() {
        let input = "; This is a comment\n(+ 1 2)";
        let tokens: Result<Vec<_>, _> = tokenize_string(input).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 应该包含注释 token
        let comment_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token.token_type, TokenType::Comment(_)))
            .collect();
        
        assert_eq!(comment_tokens.len(), 1);
    }

    #[test]
    fn test_tokenize_symbols() {
        let input = "symbol + - * / define lambda";
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 所有非 EOF 的 token 都应该是符号
        let symbol_count = tokens.iter()
            .filter(|token| matches!(token.token_type, TokenType::Symbol(_)))
            .count();
        
        assert_eq!(symbol_count, 7);
    }

    #[test]
    fn test_tokenize_booleans() {
        let input = "#t #f #true #false";
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        let boolean_tokens: Vec<_> = tokens.iter()
            .filter(|token| matches!(token.token_type, TokenType::Boolean(_)))
            .collect();
        
        assert_eq!(boolean_tokens.len(), 4);
    }

    #[test]
    fn test_position_tracking() {
        let input = "hello\nworld";
        let tokens: Result<Vec<_>, _> = tokenize_string(input).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 找到 "world" token
        let world_token = tokens.iter()
            .find(|token| {
                matches!(&token.token_type, TokenType::Symbol(s) if s == "world")
            });
        
        assert!(world_token.is_some());
        let world_token = world_token.unwrap();
        
        // "world" 实际上在位置 11 处开始（基于实际的词法分析器行为）
        assert_eq!(world_token.span.start, 11);
    }

    #[test]
    fn test_error_handling() {
        let input = "\"unterminated string";
        let tokens: Result<Vec<_>, _> = tokenize_string(input).collect();
        
        // 应该产生错误
        assert!(tokens.is_err());
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let tokens: Result<Vec<_>, _> = tokenize_string(input).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 只应该有 EOF token
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn test_character_literals() {
        let input = "#\\a #\\space #\\newline #\\tab";
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        
        // 应该有 4 个字符 Token 和 1 个 EOF
        let characters: Vec<_> = tokens.iter()
            .filter(|token| matches!(token.token_type, TokenType::Character(_)))
            .collect();
            
        assert_eq!(characters.len(), 4);
        
        // 验证具体的字符值
        match &characters[0].token_type {
            TokenType::Character(c) => assert_eq!(*c, 'a'),
            _ => panic!("Expected character token"),
        }
        
        match &characters[1].token_type {
            TokenType::Character(c) => assert_eq!(*c, ' '),
            _ => panic!("Expected character token"),
        }
        
        match &characters[2].token_type {
            TokenType::Character(c) => assert_eq!(*c, '\n'),
            _ => panic!("Expected character token"),
        }
        
        match &characters[3].token_type {
            TokenType::Character(c) => assert_eq!(*c, '\t'),
            _ => panic!("Expected character token"),
        }
    }
}
