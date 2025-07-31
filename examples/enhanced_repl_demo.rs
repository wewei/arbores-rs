// Enhanced demo showcasing position-aware error reporting
// Run this with: cargo run --example enhanced_repl_demo

use arbores::legacy::eval::Evaluator;
use arbores::legacy::eval::context::EvaluationContext;
use arbores::legacy::types::Position;

fn main() {
    println!("🚀 Enhanced Arbores Error Position Demo");
    println!("=======================================");
    
    let evaluator = Evaluator::new();
    
    // Demo 1: Position-aware evaluation 
    println!("\n📍 Demo 1: Position-Aware Evaluation");
    let context = EvaluationContext::new();
    let program_context = context.enter_call(
        Some(Position::new(1, 1)),
        Some("program".to_string())
    );
    
    println!("Input: (+ 1 2 3)");
    match evaluator.eval_string_located("(+ 1 2 3)", Some(&program_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Demo 2: Error with precise position information
    println!("\n📍 Demo 2: Undefined Variable with Position");
    println!("Input: undefined-variable");
    match evaluator.eval_string_located("undefined-variable", Some(&program_context)) {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Call stack:");
            println!("{}", program_context.format_call_stack());
        }
    }
    
    // Demo 3: Multi-line expression with error
    println!("\n📍 Demo 3: Multi-line Expression Error");
    let multiline_code = r#"(define test
  (lambda (x)
    (+ x missing-var)))"#;
    
    println!("Input:");
    println!("{}", multiline_code);
    match evaluator.eval_string_located(multiline_code, Some(&program_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Call stack:");
            println!("{}", program_context.format_call_stack());
        }
    }
    
    // Demo 4: Nested function calls with position tracking
    println!("\n📍 Demo 4: Nested Function Calls");
    let nested_code = r#"
(define outer-func
  (lambda (a)
    (define inner-func
      (lambda (b)
        (+ a b c))) ; 'c' is undefined
    (inner-func a)))
"#;
    
    println!("Input:");
    println!("{}", nested_code.trim());
    match evaluator.eval_string_located(nested_code, Some(&program_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Call stack:");
            println!("{}", program_context.format_call_stack());
        }
    }
    
    // Demo 5: Type error with position
    println!("\n📍 Demo 5: Type Error with Position");
    let type_error_code = r#"(+ 1 "not a number")"#;
    
    println!("Input: {}", type_error_code);
    match evaluator.eval_string_located(type_error_code, Some(&program_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Call stack:");
            println!("{}", program_context.format_call_stack());
        }
    }
    
    // Demo 6: Arity error with position
    println!("\n📍 Demo 6: Arity Error with Position");
    let better_arity_error = r#"(= 1)"#; // = expects exactly 2 arguments
    
    println!("Input: {}", better_arity_error);
    match evaluator.eval_string_located(better_arity_error, Some(&program_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Call stack:");
            println!("{}", program_context.format_call_stack());
        }
    }
    
    // Demo 7: Complex nested context
    println!("\n📍 Demo 7: Complex Nested Context Simulation");
    let root = EvaluationContext::new();
    let main_ctx = root.enter_call(
        Some(Position::new(1, 1)),
        Some("main".to_string())
    );
    let func1_ctx = main_ctx.enter_call(
        Some(Position::new(5, 10)),
        Some("calculate".to_string())
    );
    let func2_ctx = func1_ctx.enter_call(
        Some(Position::new(12, 20)),
        Some("helper".to_string())
    );
    
    println!("Simulated deep call stack:");
    println!("{}", func2_ctx.format_call_stack());
    
    // Test an error in the deep context
    println!("\nError in deep context:");
    match evaluator.eval_string_located("nonexistent-function", Some(&func2_ctx)) {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => {
            println!("Error: {}", e);
            println!("Full call stack:");
            println!("{}", func2_ctx.format_call_stack());
        }
    }
    
    println!("\n✅ Enhanced Error Position Demo Complete!");
    println!("Features demonstrated:");
    println!("  ✓ Position-aware expression evaluation");
    println!("  ✓ Automatic position extraction from parsed expressions");
    println!("  ✓ Enhanced error messages with precise locations");
    println!("  ✓ Full call stack tracking in debug mode");
    println!("  ✓ Multiple error types with position information");
    println!("  ✓ Multi-line expression support");
    println!("  ✓ Nested context propagation");
}
