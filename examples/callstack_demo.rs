use arbores::repl::Repl;
use arbores::eval::EvaluationContext;
use arbores::types::Position;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new()?;
    
    println!("=== Callstack Demo ===\n");
    
    // 创建一个模拟的调用栈情境
    let root_context = EvaluationContext::new();
    let main_context = root_context.enter_call(
        Some(Position::new(1, 1)), 
        Some("main".to_string())
    );
    let calculation_context = main_context.enter_call(
        Some(Position::new(2, 5)), 
        Some("calculate".to_string())
    );
    let division_context = calculation_context.enter_call(
        Some(Position::new(3, 10)), 
        Some("divide".to_string())
    );

    println!("1. 演示简单的除零错误（无调用栈）：");
    match repl.eval("(/ 1 0)", None) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}\n", e),
    }

    println!("2. 演示带调用栈的除零错误：");
    match repl.eval("(/ 1 0)", Some(&division_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}\n", e),
    }

    println!("3. 演示未定义变量错误（带调用栈）：");
    match repl.eval("unknown-variable", Some(&division_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}\n", e),
    }

    println!("4. 演示类型错误（带调用栈）：");
    match repl.eval("(+ 1 \"hello\")", Some(&division_context)) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}\n", e),
    }

    Ok(())
}
