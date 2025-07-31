use arbores::interpreter::lexer::{tokenize_string, Token, TokenType, Span};

fn main() {
    println!("=== New Lexer Design Demo ===\n");
    
    let input = r#"(+ 42 "hello world")"#;
    println!("Input: {}", input);
    println!("Tokens:");
    
    let mut token_count = 0;
    for (i, token_result) in tokenize_string(input).enumerate() {
        match token_result {
            Ok(token) => {
                token_count += 1;
                println!("  {}: {:?}", i, token.token_type);
                println!("      Raw text: {:?}", token.raw_text);
                println!("      Span: {} -> {}", token.span.start, token.span.end);
                println!("      Length: {} chars", token.span.len());
                println!();
            }
            Err(error) => {
                println!("  Error: {}", error);
                break;
            }
        }
    }
    
    println!("Total tokens: {}", token_count);
    
    // Test Span functionality
    println!("\n=== Span Functionality Demo ===");
    let span = Span::from_char_range(0, 5);  // "hello" 有 5 个字符
    println!("Span for 'hello': start={}, end={}, len={}", 
             span.start, span.end, span.len());
    
    // Test empty span creation
    let empty_span = Span::empty(10);
    println!("Empty span at position 10: start={}, end={}, len={}, is_empty={}",
             empty_span.start, empty_span.end, empty_span.len(), empty_span.is_empty());
    
    // Test token creation methods
    println!("\n=== Token Creation Demo ===");
    let token1 = Token::from_text(TokenType::Symbol("test".to_string()), "test", 0);
    println!("Token created with from_text: {:?}", token1.token_type);
    println!("Token span: {:?}", token1.span);
    
    // Test trivia detection
    println!("\n=== Trivia Token Demo ===");
    let whitespace_token = Token::from_text(
        TokenType::Whitespace("   ".to_string()), 
        "   ", 
        0
    );
    println!("Whitespace token is trivia: {}", whitespace_token.token_type.is_trivia());
    
    let symbol_token = Token::from_text(
        TokenType::Symbol("symbol".to_string()),
        "symbol",
        0
    );
    println!("Symbol token is trivia: {}", symbol_token.token_type.is_trivia());
}
