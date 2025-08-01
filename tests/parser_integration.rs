//! 解析器和词法分析器联合集成测试

use arbores::interpreter::parser::{parse_from_string, SExpr, SExprContent, Value};

/// 从源代码字符串解析S表达式的便利函数
fn parse_source(source: &str) -> Result<Vec<SExpr>, String> {
    let output = parse_from_string(source);
    
    match output.result {
        Ok(exprs) => Ok(exprs),
        Err(e) => Err(format!("Parse error: {}\nSource rebuilt: {}", e, output.source_text)),
    }
}

#[test]
fn test_parse_numbers() {
    let test_cases = vec![
        ("42", Value::Number(42.0)),
        ("-123", Value::Number(-123.0)),
        ("3.14", Value::Number(3.14)),
        ("-2.718", Value::Number(-2.718)),
        ("1e10", Value::Number(1e10)),
        ("2.5e-3", Value::Number(2.5e-3)),
    ];
    
    for (input, expected) in test_cases {
        let exprs = parse_source(input).expect(&format!("Failed to parse: {}", input));
        assert_eq!(exprs.len(), 1, "Expected one expression for: {}", input);
        
        if let SExprContent::Atom(value) = &exprs[0].content {
            assert_eq!(*value, expected, "Failed for input: {}", input);
        } else {
            panic!("Expected atom for input: {}", input);
        }
    }
}

#[test]
fn test_parse_simple_list() {
    let source = "(+ 1 2)";
    let exprs = parse_source(source).expect("Failed to parse simple list");
    
    assert_eq!(exprs.len(), 1);
    
    // 解析 (+ 1 2) 为 cons 链表结构
    let expr = &exprs[0].content;
    
    // 简单验证这是一个表达式
    match expr {
        SExprContent::Cons { car, cdr } => {
            // 检查第一个元素是符号 '+'
            if let SExprContent::Atom(Value::Symbol(op)) = &car.content {
                assert_eq!(op, "+");
            } else {
                panic!("Expected symbol for operator");
            }
            
            // 验证这不是空列表
            assert!(!matches!(cdr.content, SExprContent::Nil));
        },
        SExprContent::Nil => panic!("Expected non-empty list"),
        SExprContent::Atom(_) => panic!("Expected list, not atom"),
        SExprContent::Vector(_) => panic!("Expected list, not vector"),
    }
}

#[test]
fn test_parse_nested_lists() {
    let source = "(+ (* 2 3) 4)";
    let exprs = parse_source(source).expect("Failed to parse nested list");
    
    assert_eq!(exprs.len(), 1);
    
    // 简单验证这是一个表达式
    let expr = &exprs[0].content;
    
    match expr {
        SExprContent::Cons { car, cdr: _ } => {
            // 检查第一个元素是符号 '+'
            if let SExprContent::Atom(Value::Symbol(op)) = &car.content {
                assert_eq!(op, "+");
            } else {
                panic!("Expected + operator");
            }
        },
        _ => panic!("Expected cons structure for nested list"),
    }
}

#[test]
fn test_parse_strings() {
    let test_cases = vec![
        (r#""hello""#, "hello"),
        (r#""world with spaces""#, "world with spaces"),
        (r#""escape\ntest""#, "escape\ntest"),
        (r#""quote\"test""#, "quote\"test"),
    ];
    
    for (input, expected) in test_cases {
        let exprs = parse_source(input).expect(&format!("Failed to parse: {}", input));
        assert_eq!(exprs.len(), 1, "Expected one expression for: {}", input);
        
        if let SExprContent::Atom(Value::String(s)) = &exprs[0].content {
            assert_eq!(s, expected, "Failed for input: {}", input);
        } else {
            panic!("Expected string for input: {}", input);
        }
    }
}

#[test]
fn test_parse_booleans() {
    let test_cases = vec![
        ("#t", true),
        ("#f", false),
        ("#true", true),
        ("#false", false),
    ];
    
    for (input, expected) in test_cases {
        let exprs = parse_source(input).expect(&format!("Failed to parse: {}", input));
        assert_eq!(exprs.len(), 1, "Expected one expression for: {}", input);
        
        if let SExprContent::Atom(Value::Boolean(b)) = &exprs[0].content {
            assert_eq!(*b, expected, "Failed for input: {}", input);
        } else {
            panic!("Expected boolean for input: {}", input);
        }
    }
}

#[test]
fn test_parse_symbols() {
    let test_cases = vec![
        "x", "variable", "function-name", "long_symbol_name", "+", "-", "*", "/",
        "<=", ">=", "=", "atom?", "null?", "zero?",
    ];
    
    for input in test_cases {
        let exprs = parse_source(input).expect(&format!("Failed to parse: {}", input));
        assert_eq!(exprs.len(), 1, "Expected one expression for: {}", input);
        
        if let SExprContent::Atom(Value::Symbol(s)) = &exprs[0].content {
            assert_eq!(s, input, "Failed for input: {}", input);
        } else {
            panic!("Expected symbol for input: {}", input);
        }
    }
}
