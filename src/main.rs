use std::rc::Rc;
use gc::Gc;

use arbores::interpreter::{
    evaluator::{evaluate, Environment, RuntimeObject, RuntimeObjectCore},
    tokenize,
    parse,
};

fn main() {
    println!("Arbores 求值器测试");
    println!("==================");
    
    // 测试简单的四则运算
    test_arithmetic();
}

fn test_arithmetic() {
    println!("\n测试四则运算:");
    
    let test_cases = vec![
        ("(+ 1 2)", "3"),
        ("(- 5 2)", "3"),
        ("(* 3 4)", "12"),
        ("(/ 10 2)", "5"),
        ("(+ 1 2 3 4)", "10"),
        ("(- 10 3 2)", "5"),
        ("(* 2 3 4)", "24"),
        ("(/ 100 2 5)", "10"),
        ("(+ 42)", "42"),
        ("(- 5)", "-5"),
        ("(* 7)", "7"),
        ("(/ 2)", "0.5"),
        ("(+)", "0"),
        ("(*)", "1"),
        ("(+ 1 2.5)", "3.5"),
        ("(* 3 2.5)", "7.5"),
    ];
    
    // 创建全局环境
    let env = Gc::new(Environment::new());
    
    for (input, expected) in test_cases {
        match evaluate_expression(input, &env) {
            Ok(result) => {
                let result_str = format_runtime_object(&result);
                let status = if result_str == expected { "✓" } else { "✗" };
                println!("{} {} => {} (期望: {})", status, input, result_str, expected);
            },
            Err(e) => {
                println!("✗ {} => 错误: {:?}", input, e);
            }
        }
    }
}

fn evaluate_expression(expr_str: &str, env: &Gc<Environment>) -> Result<Rc<RuntimeObject>, Box<dyn std::error::Error>> {
    // 词法分析
    let tokens = tokenize(expr_str.chars());
    
    // 语法分析
    let parse_output = parse(tokens);
    let exprs = parse_output.result?;
    
    if exprs.is_empty() {
        return Err("没有解析到表达式".into());
    }
    
    // 求值
    let result = evaluate(Rc::new(exprs.remove(0)), env.clone())?;
    Ok(result)
}

fn format_runtime_object(obj: &RuntimeObject) -> String {
    match &obj.core {
        RuntimeObjectCore::Integer(n) => n.to_string(),
        RuntimeObjectCore::Float(n) => {
            if n.fract() == 0.0 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        },
        RuntimeObjectCore::String(s) => s.to_string(),
        RuntimeObjectCore::Boolean(b) => if *b { "#t".to_string() } else { "#f".to_string() },
        RuntimeObjectCore::Nil => "()".to_string(),
        _ => format!("{:?}", obj.core),
    }
}
