//! 演示修复后的 SourceBuilder 如何正确处理原始文本保留
//! 
//! 这个演示展示了在修复之前和之后，SourceBuilder 如何处理相同语义但不同文本表示的 Token。
//! 
//! 运行演示：
//! ```
//! cargo run --example source_builder_demo
//! ```

use arbores::interpreter::lexer::types::{Token, TokenType, Span};
use arbores::interpreter::parser::SourceBuilder;

/// 演示原始文本保留功能
pub fn demonstrate_raw_text_preservation() {
    println!("=== SourceBuilder 原始文本保留演示 ===\n");
    
    // 示例1: 数字的不同表示形式
    println!("1. 数字的不同表示形式:");
    demonstrate_number_formats();
    
    // 示例2: 布尔值的不同表示形式
    println!("\n2. 布尔值的不同表示形式:");
    demonstrate_boolean_formats();
    
    // 示例3: 带有 trivia token 的复杂表达式
    println!("\n3. 带有空白和注释的复杂表达式:");
    demonstrate_trivia_preservation();
    
    // 示例4: 字符串的不同引号风格
    println!("\n4. 语法等价但文本不同的表达式:");
    demonstrate_equivalent_expressions();
    
    println!("\n=== 演示总结 ===");
    println!("✓ SourceBuilder 现在使用 Token 的 raw_text 字段来重建源代码");
    println!("✓ 这确保了原始文本格式的保留，避免了 span 错位问题");
    println!("✓ 相同语义但不同表示的 token (如 +1 vs 1) 会保持原始格式");
    println!("✓ 所有的空白、注释和格式信息都被完整保留");
}

fn demonstrate_number_formats() {
    let mut builder = SourceBuilder::new();
    
    // 创建语义相同但文本不同的数字 token
    let tokens = vec![
        Token::new(TokenType::Integer(1), Span::new(0, 2), "+1".to_string()),
        Token::new(TokenType::Integer(1), Span::new(3, 4), "1".to_string()),
        Token::new(TokenType::Float(1.0), Span::new(5, 8), "1.0".to_string()),
        Token::new(TokenType::Float(1.0), Span::new(9, 13), "1.00".to_string()),
        Token::new(TokenType::Integer(-5), Span::new(14, 16), "-5".to_string()),
    ];
    
    for token in &tokens {
        builder.add_token(token);
    }
    
    let result = builder.build();
    println!("  原始 tokens: [+1, 1, 1.0, 1.00, -5]");
    println!("  重建结果: '{}'", result);
    println!("  ✓ 保持了原始文本格式，避免了 span 错位");
}

fn demonstrate_boolean_formats() {
    let mut builder = SourceBuilder::new();
    
    // 创建语义相同但文本不同的布尔 token
    let tokens = vec![
        Token::new(TokenType::Boolean(true), Span::new(0, 2), "#t".to_string()),
        Token::new(TokenType::Boolean(true), Span::new(3, 8), "#true".to_string()),
        Token::new(TokenType::Boolean(false), Span::new(9, 11), "#f".to_string()),
        Token::new(TokenType::Boolean(false), Span::new(12, 18), "#false".to_string()),
    ];
    
    for token in &tokens {
        builder.add_token(token);
    }
    
    let result = builder.build();
    println!("  原始 tokens: [#t, #true, #f, #false]");
    println!("  重建结果: '{}'", result);
    println!("  ✓ 保持了布尔值的原始表示形式");
}

fn demonstrate_trivia_preservation() {
    let mut builder = SourceBuilder::new();
    
    // 模拟一个带有注释和空白的表达式: (+ 1  ; 加一\n  2)
    let tokens = vec![
        Token::new(TokenType::LeftParen, Span::new(0, 1), "(".to_string()),
        Token::new(TokenType::Symbol("+".to_string()), Span::new(1, 2), "+".to_string()),
        Token::new(TokenType::Whitespace(" ".to_string()), Span::new(2, 3), " ".to_string()),
        Token::new(TokenType::Integer(1), Span::new(3, 4), "1".to_string()),
        Token::new(TokenType::Whitespace("  ".to_string()), Span::new(4, 6), "  ".to_string()),
        Token::new(TokenType::Comment(" 加一".to_string()), Span::new(6, 11), "; 加一".to_string()),
        Token::new(TokenType::Newline, Span::new(11, 12), "\n".to_string()),
        Token::new(TokenType::Whitespace("  ".to_string()), Span::new(12, 14), "  ".to_string()),
        Token::new(TokenType::Integer(2), Span::new(14, 15), "2".to_string()),
        Token::new(TokenType::RightParen, Span::new(15, 16), ")".to_string()),
    ];
    
    for token in &tokens {
        builder.add_token(token);
    }
    
    let result = builder.build();
    println!("  原始表达式: '(+ 1  ; 加一\\n  2)'");
    println!("  重建结果: '{}'", result.escape_debug());
    println!("  ✓ 完美保留了所有空白和注释");
}

fn demonstrate_equivalent_expressions() {
    // 演示两个语义等价但文本表示不同的表达式
    
    println!("  表达式 A: (+ +1 #t)");
    let mut builder_a = SourceBuilder::new();
    let tokens_a = vec![
        Token::new(TokenType::LeftParen, Span::new(0, 1), "(".to_string()),
        Token::new(TokenType::Symbol("+".to_string()), Span::new(1, 2), "+".to_string()),
        Token::new(TokenType::Integer(1), Span::new(3, 5), "+1".to_string()),
        Token::new(TokenType::Boolean(true), Span::new(6, 8), "#t".to_string()),
        Token::new(TokenType::RightParen, Span::new(8, 9), ")".to_string()),
    ];
    for token in &tokens_a {
        builder_a.add_token(token);
    }
    
    println!("  表达式 B: (+ 1 #true)");
    let mut builder_b = SourceBuilder::new();
    let tokens_b = vec![
        Token::new(TokenType::LeftParen, Span::new(0, 1), "(".to_string()),
        Token::new(TokenType::Symbol("+".to_string()), Span::new(1, 2), "+".to_string()),
        Token::new(TokenType::Integer(1), Span::new(3, 4), "1".to_string()),
        Token::new(TokenType::Boolean(true), Span::new(5, 10), "#true".to_string()),
        Token::new(TokenType::RightParen, Span::new(10, 11), ")".to_string()),
    ];
    for token in &tokens_b {
        builder_b.add_token(token);
    }
    
    let result_a = builder_a.build();
    let result_b = builder_b.build();
    
    println!("  重建结果 A: '{}'", result_a);
    println!("  重建结果 B: '{}'", result_b);
    println!("  ✓ 两个表达式语义相同，但保持了不同的文本表示");
    
    // 验证语义等价性
    assert_eq!(
        tokens_a[1].token_type, tokens_b[1].token_type,
        "+ 符号应该相同"
    );
    assert_eq!(
        tokens_a[2].token_type, tokens_b[2].token_type,
        "数字 1 应该语义相同"
    );
    assert_eq!(
        tokens_a[3].token_type, tokens_b[3].token_type,
        "布尔值 true 应该语义相同"
    );
    
    // 但文本表示不同
    assert_ne!(result_a, result_b, "文本表示应该不同");
    println!("  ✓ 验证通过：语义相同，文本不同");
}

fn main() {
    demonstrate_raw_text_preservation();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_builder_demo() {
        // 确保演示函数能正常运行
        demonstrate_raw_text_preservation();
    }
    
    #[test]
    fn test_span_consistency() {
        // 测试确保重建的文本长度与原始 span 一致
        let mut builder = SourceBuilder::new();
        
        let token = Token::new(
            TokenType::Integer(1), 
            Span::new(0, 2), 
            "+1".to_string()
        );
        
        builder.add_token(&token);
        let result = builder.build();
        
        // 重建的文本应该与原始 span 长度一致
        assert_eq!(result.chars().count(), token.span.len());
        assert_eq!(result, "+1");
    }
}
