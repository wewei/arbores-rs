//! 数值解析专项测试
//! 
//! 本模块包含对数值词法分析的全面测试，涵盖：
//! 1. 正负整数
//! 2. 正负浮点数
//! 3. 科学计数法
//! 4. 边界情况和错误处理

#[cfg(test)]
mod tests {
    use crate::interpreter::lexer::{tokenize_string, TokenType, filter_trivia_tokens};

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
    fn test_positive_floats() {
        let test_cases = vec![
            ("0.0", TokenType::Float(0.0)),
            ("1.0", TokenType::Float(1.0)),
            ("3.14", TokenType::Float(3.14)),
            ("123.456", TokenType::Float(123.456)),
            ("0.123", TokenType::Float(0.123)),
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
    fn test_negative_floats() {
        let test_cases = vec![
            ("-0.0", TokenType::Float(-0.0)),
            ("-1.0", TokenType::Float(-1.0)),
            ("-3.14", TokenType::Float(-3.14)),
            ("-123.456", TokenType::Float(-123.456)),
            ("-0.123", TokenType::Float(-0.123)),
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
    fn test_numbers_with_separators() {
        // 测试数字后跟分隔符的情况
        let test_cases = vec![
            ("42 ", vec![TokenType::Integer(42)]),
            ("3.14)", vec![TokenType::Float(3.14), TokenType::RightParen]),
            ("(123", vec![TokenType::LeftParen, TokenType::Integer(123)]),
            ("-456.78]", vec![TokenType::Float(-456.78), TokenType::RightBracket]),
        ];
        
        for (input, expected_types) in test_cases {
            let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
            let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
            
            // 减去 EOF token
            let token_types: Vec<_> = tokens.iter()
                .take(tokens.len() - 1)
                .map(|t| t.token_type.clone())
                .collect();
            
            assert_eq!(token_types, expected_types, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_plus_minus_as_symbols() {
        // 测试单独的正负号应该被解析为符号，而不是数字
        let simple_cases = vec![
            ("+", TokenType::Symbol("+".to_string())),
            ("-", TokenType::Symbol("-".to_string())),
        ];
        
        let complex_cases = vec![
            ("+ 1", vec![TokenType::Symbol("+".to_string()), TokenType::Integer(1)]),
            ("- 1", vec![TokenType::Symbol("-".to_string()), TokenType::Integer(1)]),
        ];
        
        for (input, expected) in simple_cases {
            let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
            let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
            assert_eq!(tokens.len(), 2); // 符号 + EOF
            assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
        }
        
        for (input, expected_types) in complex_cases {
            let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
            let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
            
            let token_types: Vec<_> = tokens.iter()
                .take(tokens.len() - 1)
                .map(|t| t.token_type.clone())
                .collect();
            
            assert_eq!(token_types, expected_types, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_edge_cases() {
        // 测试边界情况
        let test_cases = vec![
            ("0", TokenType::Integer(0)),
            ("-0", TokenType::Integer(0)),
            ("0.0", TokenType::Float(0.0)),
            ("-0.0", TokenType::Float(-0.0)),
            ("1.0e0", TokenType::Float(1.0)),
            ("-1.0e0", TokenType::Float(-1.0)),
        ];
        
        for (input, expected) in test_cases {
            let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
            let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
            assert_eq!(tokens.len(), 2); // 数字 + EOF
            assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_invalid_numbers() {
        // 测试无效的数字格式
        let invalid_cases = vec![
            "1.2.3",     // 多个小数点
            "1e",        // 科学计数法不完整
            "1e+",       // 科学计数法指数不完整
            "1e-",       // 科学计数法指数不完整
            ".123",      // 开头的小数点（可能根据实现决定是否支持）
            "123.",      // 结尾的小数点（可能根据实现决定是否支持）
        ];
        
        for input in invalid_cases {
            let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
            // 这些应该要么产生错误，要么被解析为多个token（如符号）
            // 具体行为取决于词法分析器的实现策略
            let _result = tokens; // 先不严格要求错误，只是不崩溃
        }
    }

    #[test]
    fn test_large_numbers() {
        // 测试大数值
        let test_cases = vec![
            ("9223372036854775807", TokenType::Integer(9223372036854775807)), // i64::MAX
            ("-9223372036854775808", TokenType::Integer(-9223372036854775808)), // i64::MIN
        ];
        
        for (input, expected) in test_cases {
            let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
            let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
            assert_eq!(tokens.len(), 2); // 数字 + EOF
            assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_float_precision() {
        // 测试浮点数精度
        let test_cases = vec![
            ("3.141592653589793", TokenType::Float(3.141592653589793)),
            ("-2.718281828459045", TokenType::Float(-2.718281828459045)),
        ];
        
        for (input, expected) in test_cases {
            let tokens: Result<Vec<_>, _> = filter_trivia_tokens(tokenize_string(input)).collect();
            let tokens = tokens.expect(&format!("Failed to tokenize: {}", input));
            assert_eq!(tokens.len(), 2); // 数字 + EOF
            assert_eq!(tokens[0].token_type, expected, "Failed for input: {}", input);
        }
    }
}
