// Demo showcasing the error position system implementation
// Run this with: cargo run --example error_position_demo

use arbores::legacy::eval::context::EvaluationContext;
use arbores::legacy::eval::Evaluator;
use arbores::legacy::parser::Parser;
use arbores::legacy::types::{SchemeError, Position};

fn main() {
    println!("🚀 Arbores Error Position System Demo");
    println!("====================================");
    
    let evaluator = Evaluator::new();
    
    // Demo 1: Basic evaluation without context (normal mode)
    println!("\n📍 Demo 1: Normal Mode (no context)");
    println!("Input: (+ 1 2 3)");
    match evaluator.eval_string("(+ 1 2 3)", None) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Demo 2: Evaluation with context (debug mode)
    println!("\n📍 Demo 2: Debug Mode (with context)");
    let root_context = EvaluationContext::new();
    let debug_context = root_context.enter_call(
        Some(Position::new(1, 1)), 
        Some("main".to_string())
    );
    
    println!("Input: (+ 1 2 3)");
    match evaluator.eval_string("(+ 1 2 3)", Some(&debug_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Demo 3: Nested function calls with context propagation
    println!("\n📍 Demo 3: Nested Context Propagation");
    let nested_code = r#"
(define factorial
  (lambda (n)
    (if (= n 0)
        1
        (* n (factorial (- n 1))))))
(factorial 5)
"#;
    
    println!("Input: {}", nested_code.trim());
    match evaluator.eval_string(nested_code, Some(&debug_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Demo 4: Error with position information
    println!("\n📍 Demo 4: Error with Position Information");
    let error_code = r#"
(define test-func
  (lambda (x)
    (+ x undefined-variable)))
(test-func 42)
"#;
    
    println!("Input: {}", error_code.trim());
    match evaluator.eval_string(error_code, Some(&debug_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Call stack: {}", debug_context.format_call_stack());
        }
    }
    
    // Demo 5: Parser position information
    println!("\n📍 Demo 5: Parser Position Information");
    let parse_error_code = "(+ 1 2";
    println!("Input: {}", parse_error_code);
    match Parser::parse(parse_error_code) {
        Ok(result) => println!("Parsed: {}", result),
        Err(SchemeError::SyntaxError(msg, Some(pos))) => {
            println!("Parse Error: {} at {}", msg, pos);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    // Demo 6: Complex nested expression with debugging
    println!("\n📍 Demo 6: Complex Expression with Context Chain");
    let complex_code = r#"
(define outer
  (lambda (a)
    (define inner
      (lambda (b)
        (if (> b 0)
            (+ a b)
            (/ a b))))
    (inner a)))
(outer -5)
"#;
    
    let main_context = EvaluationContext::new();
    let program_context = main_context.enter_call(
        Some(Position::new(1, 1)),
        Some("program".to_string())
    );
    
    println!("Input: {}", complex_code.trim());
    match evaluator.eval_string(complex_code, Some(&program_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Call stack: {}", program_context.format_call_stack());
        }
    }
    
    // Demo 7: Show context chain functionality
    println!("\n📍 Demo 7: Context Chain Demonstration");
    let root = EvaluationContext::new();
    let level1 = root.enter_call(
        Some(Position::new(1, 1)),
        Some("main".to_string())
    );
    let level2 = level1.enter_call(
        Some(Position::new(5, 10)),
        Some("calculate".to_string())
    );
    let level3 = level2.enter_call(
        Some(Position::new(8, 15)),
        Some("helper".to_string())
    );
    
    println!("Context chain created:");
    println!("{}", level3.format_call_stack());
    
    println!("\n✅ Error Position System Demo Complete!");
    println!("The system now supports:");
    println!("  - Chain-based immutable evaluation contexts");
    println!("  - Position information propagation");
    println!("  - Call stack tracking in debug mode");
    println!("  - Backward compatible API (None for normal mode)");
    println!("  - Enhanced error reporting with position details");
}
