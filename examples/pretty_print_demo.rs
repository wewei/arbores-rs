//! SExpr 美化输出演示
//! 
//! 本示例演示如何使用 SExpr 的美化输出功能，在每个表达式的 span 注释后
//! 添加换行和缩进，使复杂的嵌套结构更易读。

use arbores::interpreter::parser::parse_from_string;

fn main() {
    println!("=== SExpr 美化输出演示 ===\n");
    
    let examples = vec![
        ("简单原子", "42"),
        ("简单列表", "(a b c)"),
        ("嵌套列表", "(define x (+ 1 2))"),
        ("引用表达式", "'(a b (c d) e)"),
        ("复杂表达式", "(let ((x 5) (y 10)) (+ x y))"),
    ];

    for (description, input) in examples {
        println!("--- {} ---", description);
        println!("输入: {}", input);
        
        let output = parse_from_string(input);
        match output.result {
            Ok(exprs) => {
                if let Some(expr) = exprs.first() {
                    println!("\n紧凑输出:");
                    println!("{}", expr);
                    
                    println!("\n美化输出:");
                    println!("{}", expr.to_pretty_string());
                }
            }
            Err(e) => {
                println!("解析错误: {:?}", e);
            }
        }
        println!("\n{}\n", "=".repeat(50));
    }
}
