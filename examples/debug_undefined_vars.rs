// Simple test to check undefined variable error handling

use arbores::{Evaluator, eval::context::EvaluationContext};

fn main() {
    println!("Testing undefined variable error handling...");
    
    let evaluator = Evaluator::new();
    let context = EvaluationContext::new();
    let debug_context = context.enter_call(
        Some(arbores::types::Position::new(1, 1)),
        Some("test".to_string())
    );
    
    // Test 1: Simple undefined variable
    println!("\n🧪 Test 1: Simple undefined variable");
    match evaluator.eval_string("undefined-var", Some(&debug_context)) {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Test 2: Undefined variable in function call
    println!("\n🧪 Test 2: Undefined variable in function call");
    let code = "(+ 1 undefined-var 3)";
    match evaluator.eval_string(code, Some(&debug_context)) {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Test 3: Function that doesn't exist
    println!("\n🧪 Test 3: Function that doesn't exist");
    let code = "(undefined-function 1 2 3)";
    match evaluator.eval_string(code, Some(&debug_context)) {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
