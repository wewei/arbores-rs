use arbores::interpreter::parser::parse_from_string;

fn main() {
    println!("=== SExpr 美化输出演示 - span 注释对齐 ===\n");
    
    let examples = vec![
        ("简单原子", "42"),
        ("简单列表", "(a b c)"),
        ("嵌套列表", "(define x (+ 1 2))"),
        ("引用表达式", "'(a b (c d) e)"),
        ("符号操作符", "(+ - * / = < > <= >= eq? null?)"),
    ];

    for (description, input) in examples {
        println!("--- {} ---", description);
        println!("输入: {}", input);
        
        let output = parse_from_string(input);
        match output.result {
            Ok(exprs) => {
                if exprs.len() == 1 {
                    println!("紧凑输出: {}", exprs[0]);
                    println!("美化输出:");
                    println!("{}", exprs[0].to_pretty_string());
                } else {
                    println!("解析出 {} 个表达式", exprs.len());
                }
            }
            Err(err) => {
                println!("解析错误: {}", err);
            }
        }
        println!();
    }
}
