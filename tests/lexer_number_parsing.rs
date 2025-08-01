//! 词法分析器数值解析集成测试

use arbores::interpreter::lexer::{tokenize_string, TokenType, filter_trivia_tokens};

#[test]
fn test_positive_integers() {
    let test_cases = vec![
        ("0", TokenType::Integer(0)),
        ("1", TokenType::Integer(1)),
        ("42", TokenType::Integer(42)),
        ("123456789", TokenType::Integer(123456789)),
    ];
    
    for (input, expected) in test_cases {
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
        assert_eq!(tokens.len(), 2); // 数字 + EOF
        assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
        assert_eq!(tokens[0].raw_text, input);
    }
}

#[test]
fn test_negative_integers() {
    let test_cases = vec![
        ("-0", TokenType::Integer(0)),
        ("-1", TokenType::Integer(-1)),
        ("-42", TokenType::Integer(-42)),
        ("-123456789", TokenType::Integer(-123456789)),
    ];
    
    for (input, expected) in test_cases {
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
        assert_eq!(tokens.len(), 2); // 数字 + EOF
        assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
        assert_eq!(tokens[0].raw_text, input);
    }
}

#[test]
fn test_scientific_notation() {
    let test_cases = vec![
        ("1e0", TokenType::Float(1e0)),
        ("1E0", TokenType::Float(1E0)),
        ("1e1", TokenType::Float(1e1)),
        ("1e-1", TokenType::Float(1e-1)),
        ("1e+1", TokenType::Float(1e+1)),
        ("2.5e3", TokenType::Float(2.5e3)),
        ("2.5E3", TokenType::Float(2.5E3)),
        ("1.23e4", TokenType::Float(1.23e4)),
        ("-2.5E-3", TokenType::Float(-2.5E-3)),
        ("1.0e-10", TokenType::Float(1.0e-10)),
        ("6.02e23", TokenType::Float(6.02e23)),
    ];
    
    for (input, expected) in test_cases {
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
        assert_eq!(tokens.len(), 2); // 数字 + EOF
        assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
        assert_eq!(tokens[0].raw_text, input);
    }
}

#[test]
fn test_signed_numbers_in_expressions() {
    // 测试原始问题："-123.45" 应该被解析为一个数字 token，而不是多个
    let input = "-123.45";
    let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
    let tokens = tokens.expect("Failed to tokenize -123.45");
    
    assert_eq!(tokens.len(), 2); // 数字 + EOF
    assert_eq!(tokens[0].token_type, TokenType::Float(-123.45));
    assert_eq!(tokens[0].raw_text, "-123.45");
}

#[test]
fn test_plus_minus_as_symbols() {
    // 测试单独的正负号应该被解析为符号，而不是数字
    let simple_cases = vec![
        ("+", TokenType::Symbol("+".to_string())),
        ("-", TokenType::Symbol("-".to_string())),
    ];
    
    for (input, expected) in simple_cases {
        let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
        let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
        assert_eq!(tokens.len(), 2); // 符号 + EOF
        assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
    }
}
