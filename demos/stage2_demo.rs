use arbores::Arbores;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Arbores MVP 第二阶段演示 ===\n");

    // 创建 Arbores 实例
    let arbores = Arbores::new();

    println!("1. 创建 S-Expression");
    
    // 创建第一个函数
    let factorial_id = arbores.create(
        "(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))",
        vec![],
        Some("计算阶乘的递归函数".to_string()),
        Some("function".to_string()),
        vec!["factorial".to_string(), "fact".to_string()],
    )?;
    
    println!("   创建阶乘函数，ID: {}", factorial_id);

    // 创建第二个函数
    let fibonacci_id = arbores.create(
        "(define (fibonacci n) (if (<= n 2) 1 (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))",
        vec![],
        Some("计算斐波那契数列的递归函数".to_string()),
        Some("function".to_string()),
        vec!["fibonacci".to_string(), "fib".to_string()],
    )?;
    
    println!("   创建斐波那契函数，ID: {}", fibonacci_id);

    // 创建一个使用前面函数的函数
    let test_id = arbores.create(
        "(define (test-math x) (+ (factorial x) (fibonacci x)))",
        vec![factorial_id, fibonacci_id],
        Some("测试数学函数的组合".to_string()),
        Some("function".to_string()),
        vec!["test-math".to_string()],
    )?;
    
    println!("   创建测试函数，ID: {}", test_id);

    println!("\n2. 查询元数据");
    
    // 查询阶乘函数的元数据
    let metadata = arbores.get_metadata(factorial_id)?;
    println!("   阶乘函数的元数据:");
    println!("   {}", format_value(&metadata, 2));

    println!("\n3. 查询依赖关系");
    
    // 查询测试函数的依赖
    let deps = arbores.get_dependencies(test_id)?;
    println!("   测试函数的依赖: {}", format_value(&deps, 0));

    println!("\n4. 按符号搜索");
    
    // 前缀搜索
    let search_results = arbores.search_by_symbol("fact", Some("prefix"))?;
    println!("   前缀搜索 'fact':");
    println!("   {}", format_value(&search_results, 2));

    // 精确搜索
    let exact_results = arbores.search_by_symbol("fibonacci", Some("exact"))?;
    println!("   精确搜索 'fibonacci':");
    println!("   {}", format_value(&exact_results, 2));

    println!("\n5. 语义搜索");
    
    // 语义搜索
    let semantic_results = arbores.semantic_search("递归")?;
    println!("   语义搜索 '递归':");
    println!("   {}", format_value(&semantic_results, 2));

    println!("\n=== 演示完成 ===");

    Ok(())
}

/// 格式化 Value 为可读字符串
fn format_value(value: &arbores::Value, indent: usize) -> String {
    let _prefix = "  ".repeat(indent);
    
    match value {
        arbores::Value::Nil => "()".to_string(),
        arbores::Value::Bool(b) => format!("#{}", if *b { "t" } else { "f" }),
        arbores::Value::Integer(i) => i.to_string(),
        arbores::Value::Float(f) => f.to_string(),
        arbores::Value::String(s) => format!("\"{}\"", s),
        arbores::Value::Symbol(s) => s.clone(),
        arbores::Value::Cons(car, cdr) => {
            if let Some(list) = value.to_vec() {
                // 格式化为列表
                let items: Vec<String> = list.iter()
                    .map(|v| format_value(v, 0))
                    .collect();
                format!("({})", items.join(" "))
            } else {
                // 格式化为点对
                format!("({} . {})", format_value(car, 0), format_value(cdr, 0))
            }
        }
        arbores::Value::BuiltinFunction { name, .. } => format!("#<builtin:{}>", name),
        arbores::Value::Lambda { params, .. } => format!("#<lambda:({})>", params.join(" ")),
    }
}
