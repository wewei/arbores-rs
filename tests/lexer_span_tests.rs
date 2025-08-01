#[cfg(test)]
mod lexer_span_tests {
    use arbores::interpreter::lexer;
    
    #[test]
    fn test_single_integer_span() {
        let input = "42";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: Integer(42) at {0, 2}, EOF at {2, 2}
        assert_eq!(tokens.len(), 2);
        
        match &tokens[0] {
            Ok(token) => {
                println!("Token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "Integer token should start at position 0");
                assert_eq!(token.span.end, 2, "Integer token should end at position 2");
            }
            Err(e) => panic!("Expected token, got error: {:?}", e),
        }
        
        match &tokens[1] {
            Ok(token) => {
                assert_eq!(token.span.start, 2, "EOF token should start at position 2");
                assert_eq!(token.span.end, 2, "EOF token should end at position 2");
            }
            Err(e) => panic!("Expected EOF token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_single_symbol_span() {
        let input = "foo";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: Symbol("foo") at {0, 3}, EOF at {3, 3}
        assert_eq!(tokens.len(), 2);
        
        match &tokens[0] {
            Ok(token) => {
                println!("Token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "Symbol token should start at position 0");
                assert_eq!(token.span.end, 3, "Symbol token should end at position 3");
            }
            Err(e) => panic!("Expected token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_multiple_tokens_span() {
        let input = "(+ 1 2)";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望的 token spans:
        // "(" -> {0, 1}
        // "+" -> {1, 2}  
        // " " -> 跳过 (whitespace)
        // "1" -> {3, 4}
        // " " -> 跳过 (whitespace)
        // "2" -> {5, 6}
        // ")" -> {6, 7}
        // EOF -> {7, 7}
        
        let non_whitespace_tokens: Vec<_> = tokens.into_iter()
            .filter(|token_result| {
                match token_result {
                    Ok(token) => !token.token_type.is_trivia(),
                    Err(_) => true,
                }
            })
            .collect();
        
        println!("Non-whitespace tokens count: {}", non_whitespace_tokens.len());
        for (i, token_result) in non_whitespace_tokens.iter().enumerate() {
            match token_result {
                Ok(token) => println!("  [{}]: {:?} at {:?}", i, token.token_type, token.span),
                Err(e) => println!("  [{}]: Error: {:?}", i, e),
            }
        }
        
        // 应该有 5 个非空白符号: (, +, 1, 2, ), EOF
        assert_eq!(non_whitespace_tokens.len(), 6);
        
        // 检查每个 token 的位置
        let expected_spans = vec![
            (0, 1),  // (
            (1, 2),  // +
            (3, 4),  // 1
            (5, 6),  // 2
            (6, 7),  // )
            (7, 7),  // EOF
        ];
        
        for (i, ((expected_start, expected_end), token_result)) in 
            expected_spans.iter().zip(non_whitespace_tokens.iter()).enumerate() {
            match token_result {
                Ok(token) => {
                    assert_eq!(token.span.start, *expected_start, 
                               "Token {} should start at position {}", i, expected_start);
                    assert_eq!(token.span.end, *expected_end, 
                               "Token {} should end at position {}", i, expected_end);
                }
                Err(e) => panic!("Expected token at index {}, got error: {:?}", i, e),
            }
        }
    }
    
    #[test]
    fn test_string_span() {
        let input = "\"hello\"";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: String("hello") at {0, 7}, EOF at {7, 7}
        assert_eq!(tokens.len(), 2);
        
        match &tokens[0] {
            Ok(token) => {
                println!("Token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "String token should start at position 0");
                assert_eq!(token.span.end, 7, "String token should end at position 7");
            }
            Err(e) => panic!("Expected token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_character_span() {
        let input = "#\\a";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: Character('a') at {0, 3}, EOF at {3, 3}
        assert_eq!(tokens.len(), 2);
        
        match &tokens[0] {
            Ok(token) => {
                println!("Token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "Character token should start at position 0");
                assert_eq!(token.span.end, 3, "Character token should end at position 3");
            }
            Err(e) => panic!("Expected token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_boolean_span() {
        let input = "#t";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: Boolean(true) at {0, 2}, EOF at {2, 2}
        assert_eq!(tokens.len(), 2);
        
        match &tokens[0] {
            Ok(token) => {
                println!("Token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "Boolean token should start at position 0");
                assert_eq!(token.span.end, 2, "Boolean token should end at position 2");
            }
            Err(e) => panic!("Expected token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_float_span() {
        let input = "3.14";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: Float(3.14) at {0, 4}, EOF at {4, 4}
        assert_eq!(tokens.len(), 2);
        
        match &tokens[0] {
            Ok(token) => {
                println!("Token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "Float token should start at position 0");
                assert_eq!(token.span.end, 4, "Float token should end at position 4");
            }
            Err(e) => panic!("Expected token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_quote_span() {
        let input = "'x";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: Quote at {0, 1}, Symbol("x") at {1, 2}, EOF at {2, 2}
        let non_whitespace_tokens: Vec<_> = tokens.into_iter()
            .filter(|token_result| {
                match token_result {
                    Ok(token) => !token.token_type.is_trivia(),
                    Err(_) => true,
                }
            })
            .collect();
        
        assert_eq!(non_whitespace_tokens.len(), 3);
        
        match &non_whitespace_tokens[0] {
            Ok(token) => {
                println!("Quote token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "Quote token should start at position 0");
                assert_eq!(token.span.end, 1, "Quote token should end at position 1");
            }
            Err(e) => panic!("Expected quote token, got error: {:?}", e),
        }
        
        match &non_whitespace_tokens[1] {
            Ok(token) => {
                println!("Symbol token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 1, "Symbol token should start at position 1");
                assert_eq!(token.span.end, 2, "Symbol token should end at position 2");
            }
            Err(e) => panic!("Expected symbol token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_empty_input_span() {
        let input = "";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        // 期望: 只有 EOF at {0, 0}
        assert_eq!(tokens.len(), 1);
        
        match &tokens[0] {
            Ok(token) => {
                println!("EOF token: {:?}, Span: {:?}", token.token_type, token.span);
                assert_eq!(token.span.start, 0, "EOF token should start at position 0");
                assert_eq!(token.span.end, 0, "EOF token should end at position 0");
            }
            Err(e) => panic!("Expected EOF token, got error: {:?}", e),
        }
    }
    
    #[test]
    fn test_whitespace_preserves_position() {
        let input = "  42";
        let tokens: Vec<_> = lexer::tokenize(input.chars()).collect();
        
        println!("All tokens for '  42':");
        for (i, token_result) in tokens.iter().enumerate() {
            match token_result {
                Ok(token) => println!("  [{}]: {:?} at {:?}", i, token.token_type, token.span),
                Err(e) => println!("  [{}]: Error: {:?}", i, e),
            }
        }
        
        // 查找数字 token
        let number_token = tokens.iter()
            .find(|token_result| {
                match token_result {
                    Ok(token) => matches!(token.token_type, arbores::interpreter::lexer::types::TokenType::Integer(_)),
                    Err(_) => false,
                }
            });
        
        match number_token {
            Some(Ok(token)) => {
                assert_eq!(token.span.start, 2, "Number token should start at position 2 (after whitespace)");
                assert_eq!(token.span.end, 4, "Number token should end at position 4");
            }
            _ => panic!("Expected to find integer token"),
        }
    }
}
