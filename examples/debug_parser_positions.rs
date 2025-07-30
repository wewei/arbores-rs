use std::fs;
use arbores::parser::Parser;

fn main() {
    let content = fs::read_to_string("test_detailed_callstack.scm").unwrap();
    println!("解析文件内容：\n{}", content);
    
    let located_exprs = Parser::parse_multiple_located(&content).unwrap();
    
    for (i, located_expr) in located_exprs.iter().enumerate() {
        println!("\n表达式 {}: {:?}", i + 1, located_expr.value);
        println!("位置: {:?}", located_expr.position);
        
        // 如果是函数定义，尝试查看其结构
        if let arbores::types::Value::Cons(_, _) = &located_expr.value {
            if let Some(list) = located_expr.value.to_vec() {
                if list.len() >= 2 {
                    if let arbores::types::Value::Symbol(op) = &list[0] {
                        if op == "define" {
                            println!("  这是一个函数定义");
                            println!("  函数名/参数: {:?}", list[1]);
                            println!("  函数体: {:?}", list[2]);
                        }
                    }
                }
            }
        }
    }
}
