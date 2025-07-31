//! 新词法分析器示例
//! 
//! 展示新的模块化词法分析器的基本用法

use crate::interpreter::lexer::{tokenize_string, filter_trivia_tokens, TokenType};

/// 演示词法分析器的基本功能
pub fn demo_lexer() {
    println!("=== 新词法分析器演示 ===\n");

    let test_cases = vec![
        "(+ 1 2)",
        "\"hello world\"",
        "; 这是注释\n(define x 42)",
        "#t #f",
        "'(a b c)",
    ];

    for (i, input) in test_cases.iter().enumerate() {
        println!("测试用例 {}: {}", i + 1, input);
        
        let tokens: Result<Vec<_>, _> = tokenize_string(input).collect();
        
        match tokens {
            Ok(tokens) => {
                println!("  生成的 Token:");
                for token in &tokens {
                    match &token.token_type {
                        TokenType::Eof => println!("    EOF"),
                        TokenType::LeftParen => println!("    LeftParen: '('"),
                        TokenType::RightParen => println!("    RightParen: ')'"),
                        TokenType::Number(n) => println!("    Number: {}", n),
                        TokenType::String(s) => println!("    String: \"{}\"", s),
                        TokenType::Symbol(s) => println!("    Symbol: {}", s),
                        TokenType::Boolean(b) => println!("    Boolean: {}", b),
                        TokenType::Quote => println!("    Quote: '"),
                        TokenType::Comment(c) => println!("    Comment: ;{}", c),
                        TokenType::Whitespace(w) => println!("    Whitespace: {:?}", w),
                        TokenType::Newline => println!("    Newline"),
                        _ => println!("    Other: {:?}", token.token_type),
                    }
                }
                
                // 也展示过滤后的 Token
                let filtered: Vec<_> = filter_trivia_tokens(tokenize_string(input))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                    
                println!("  过滤后的 Token:");
                for token in &filtered {
                    if !matches!(token.token_type, TokenType::Eof) {
                        println!("    {:?}", token.token_type);
                    }
                }
            }
            Err(err) => {
                println!("  错误: {:?}", err);
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_lexer() {
        // 这个测试只是确保 demo 函数能运行而不崩溃
        demo_lexer();
    }
}
