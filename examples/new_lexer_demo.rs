use arbores::interpreter::lexer::{tokenize_string, Token, TokenType, Position, Span};

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
                println!("      Span: {:?} -> {:?}", token.start_pos(), token.end_pos());
                println!("      Length: {} bytes", token.span.len());
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
    let start = Position::start();
    let span = Span::from_text("hello", start);
    println!("Span for 'hello': start={:?}, end={:?}, len={}", 
             span.start, span.end, span.len());
    
    // Test position advancement
    let new_pos = start.advance_by_text("hello\nworld");
    println!("Position after 'hello\\nworld': line={}, column={}, offset={}",
             new_pos.line, new_pos.column, new_pos.byte_offset);
    
    // Test token creation methods
    println!("\n=== Token Creation Demo ===");
    let token1 = Token::from_text(TokenType::Symbol("test".to_string()), "test", start);
    println!("Token created with from_text: {:?}", token1.token_type);
    println!("Token span: {:?}", token1.span);
    
    // Test trivia detection
    println!("\n=== Trivia Token Demo ===");
    let whitespace_token = Token::from_text(
        TokenType::Whitespace("   ".to_string()), 
        "   ", 
        start
    );
    println!("Whitespace token is trivia: {}", whitespace_token.token_type.is_trivia());
    
    let symbol_token = Token::from_text(
        TokenType::Symbol("symbol".to_string()),
        "symbol",
        start
    );
    println!("Symbol token is trivia: {}", symbol_token.token_type.is_trivia());
}
