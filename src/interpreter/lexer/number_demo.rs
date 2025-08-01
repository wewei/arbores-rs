//! 数值解析演示程序
//! 
//! 演示新的数值解析功能，包括：
//! 1. 正负整数
//! 2. 正负浮点数  
//! 3. 科学计数法
//! 4. 边界情况处理

use crate::interpreter::lexer::{tokenize_string, TokenType};

pub fn run_number_parsing_demo() {
    println!("=== 数值解析功能演示 ===\n");
    
    let test_cases = vec![
        // 基本整数
        "42",
        "-123",
        "+456",
        "0",
        
        // 浮点数
        "3.14",
        "-2.718",
        "+1.414",
        "0.0",
        "-0.0",
        
        // 科学计数法
        "1e10",
        "2.5e-3",
        "-1.23E+4",
        "6.02e23",
        "1.0e-10",
        
        // 原始问题：带符号的数字
        "-123.45",
        "+987.654",
        
        // 边界情况
        "9223372036854775807",  // i64::MAX
        "-9223372036854775808", // i64::MIN
        
        // 分隔符测试
        "42)",
        "(123",
        "-456.78]",
        
        // 符号 vs 数字
        "+",
        "-",
        "+ 1",
        "- 1",
    ];
    
    for input in test_cases {
        println!("输入: \"{}\"", input);
        
        match tokenize_and_analyze(input) {
            Ok(tokens) => {
                println!("  解析结果:");
                for (i, token) in tokens.iter().enumerate() {
                    match &token.token_type {
                        TokenType::Integer(n) => {
                            println!("    [{}] 整数: {} (原文: '{}')", i, n, token.raw_text);
                        }
                        TokenType::Float(n) => {
                            println!("    [{}] 浮点数: {} (原文: '{}')", i, n, token.raw_text);
                        }
                        TokenType::Symbol(s) => {
                            println!("    [{}] 符号: {} (原文: '{}')", i, s, token.raw_text);
                        }
                        TokenType::LeftParen => {
                            println!("    [{}] 左括号: ( (原文: '{}')", i, token.raw_text);
                        }
                        TokenType::RightParen => {
                            println!("    [{}] 右括号: ) (原文: '{}')", i, token.raw_text);
                        }
                        TokenType::LeftBracket => {
                            println!("    [{}] 左方括号: [ (原文: '{}')", i, token.raw_text);
                        }
                        TokenType::RightBracket => {
                            println!("    [{}] 右方括号: ] (原文: '{}')", i, token.raw_text);
                        }
                        TokenType::Eof => {
                            // 不显示 EOF token
                        }
                        _ => {
                            println!("    [{}] 其他: {:?} (原文: '{}')", i, token.token_type, token.raw_text);
                        }
                    }
                }
            }
            Err(e) => {
                println!("  错误: {}", e);
            }
        }
        println!();
    }
    
    // 特别演示原始问题的解决
    println!("=== 原始问题验证 ===");
    println!("问题: \"-123.45\" 应该解析为一个数字 token，而不是多个 token\n");
    
    let input = "-123.45";
    match tokenize_and_analyze(input) {
        Ok(tokens) => {
            let non_eof_tokens: Vec<_> = tokens.iter()
                .filter(|t| !matches!(t.token_type, TokenType::Eof))
                .collect();
            
            println!("输入: \"{}\"", input);
            println!("Token 数量: {} (不包括 EOF)", non_eof_tokens.len());
            
            if non_eof_tokens.len() == 1 {
                match &non_eof_tokens[0].token_type {
                    TokenType::Float(n) => {
                        println!("✅ 成功！解析为单个浮点数: {}", n);
                        println!("   原始文本: \"{}\"", non_eof_tokens[0].raw_text);
                    }
                    _ => {
                        println!("❌ 失败！解析为: {:?}", non_eof_tokens[0].token_type);
                    }
                }
            } else {
                println!("❌ 失败！解析为 {} 个 token:", non_eof_tokens.len());
                for (i, token) in non_eof_tokens.iter().enumerate() {
                    println!("  [{}] {:?} ('{}')", i, token.token_type, token.raw_text);
                }
            }
        }
        Err(e) => {
            println!("❌ 解析错误: {}", e);
        }
    }
}

fn tokenize_and_analyze(input: &str) -> Result<Vec<crate::interpreter::lexer::Token>, Box<dyn std::error::Error>> {
    let tokens: Result<Vec<_>, _> = tokenize_string(input).collect();
    Ok(tokens?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_runs_without_panic() {
        // 只是确保演示程序不会崩溃
        run_number_parsing_demo();
    }

    #[test]
    fn test_original_problem_solved() {
        // 专门测试原始问题是否解决
        let input = "-123.45";
        let tokens = tokenize_and_analyze(input).expect("Should tokenize successfully");
        
        let non_eof_tokens: Vec<_> = tokens.iter()
            .filter(|t| !matches!(t.token_type, TokenType::Eof))
            .collect();
        
        assert_eq!(non_eof_tokens.len(), 1, "Should parse as single token");
        
        match &non_eof_tokens[0].token_type {
            TokenType::Float(n) => {
                assert_eq!(*n, -123.45);
                assert_eq!(non_eof_tokens[0].raw_text, "-123.45");
            }
            _ => panic!("Should parse as Float, got {:?}", non_eof_tokens[0].token_type),
        }
    }
}
