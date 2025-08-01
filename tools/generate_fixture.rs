use std::env;
use arbores::interpreter::parser::parse_from_string;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <test_suite>", args[0]);
        eprintln!("Available test suites: basic_expressions, edge_cases");
        std::process::exit(1);
    }
    
    let test_suite = &args[1];
    
    match test_suite.as_str() {
        "basic_expressions" => generate_basic_expressions(),
        "edge_cases" => generate_edge_cases(),
        _ => {
            eprintln!("Unknown test suite: {}", test_suite);
            eprintln!("Available test suites: basic_expressions, edge_cases");
            std::process::exit(1);
        }
    }
}

fn generate_basic_expressions() {
    let test_cases = vec![
        ("simple_integer", "解析简单整数", "42"),
        ("simple_float", "解析简单浮点数", "3.14"),
        ("simple_string", "解析简单字符串", "\"hello\""),
        ("simple_symbol", "解析简单符号", "foo"),
        ("boolean_true", "解析布尔值 true", "#t"),
        ("boolean_false", "解析布尔值 false", "#f"),
        ("character", "解析字符", "#\\a"),
        ("empty_list", "解析空列表", "()"),
        ("simple_list", "解析简单列表", "(1 2 3)"),
        ("nested_list", "解析嵌套列表", "(a (b c) d)"),
        ("empty_vector", "解析空向量", "#()"),
        ("simple_vector", "解析简单向量", "#(1 2 3)"),
        ("nested_vector", "解析嵌套向量", "#((a b) 42 \"hello\")"),
        ("quote_expression", "解析引用表达式", "'x"),
        ("quote_list", "解析引用列表", "'(a b)"),
        ("quasiquote_expression", "解析准引用表达式", "`x"),
        ("unquote_expression", "解析反引用表达式", ",x"),
        ("unquote_splicing", "解析反引用拼接", ",@x"),
        ("dotted_pair", "解析点对", "(a . b)"),
        ("string_with_escapes", "解析带转义字符的字符串", "\"hello\\nworld\""),
        ("multiline_expression", "解析多行表达式", "(define x\n  42)"),
    ];
    
    println!("# 基本表达式解析测试 fixture");
    println!("# 输入源代码 -> 解析为 SExpr -> Pretty Display 输出 -> 与预期字符串比对");
    println!();
    println!("test_cases:");
    
    for (name, description, input) in test_cases {
        let output = parse_from_string(input);
        match &output.result {
            Ok(expressions) if expressions.len() == 1 => {
                let actual = expressions[0].to_pretty_string();
                println!("  - name: \"{}\"", name);
                println!("    description: \"{}\"", description);
                println!("    input: {:?}", input);
                
                if actual.contains('\n') {
                    println!("    expected: |");
                    for line in actual.lines() {
                        println!("      {}", line);
                    }
                } else {
                    println!("    expected: {:?}", actual);
                }
                println!();
            }
            Ok(expressions) => {
                eprintln!("ERROR: {} expressions for {}: {:?}", expressions.len(), name, input);
            }
            Err(err) => {
                eprintln!("ERROR: Parse failed for {}: {} (input: {:?})", name, err, input);
            }
        }
    }
}

fn generate_edge_cases() {
    let test_cases = vec![
        ("nested_quotes", "嵌套引用", "''x"),
        ("mixed_quotes", "混合引用类型", "'`,@x"),
        ("deeply_nested_list", "深度嵌套列表", "(((())))"),
        ("list_with_many_elements", "多元素列表", "(a b c d e f g h i j)"),
        ("complex_dotted_structure", "复杂点对结构", "(a b . (c d . e))"),
        ("zero_and_negative_numbers", "零和负数", "(0 -1 -3.14)"),
        ("special_characters_in_symbols", "符号中的特殊字符", "(+ - * / = < > <= >= eq? null?)"),
        ("character_literals", "字符字面量", "(#\\a #\\space #\\newline #\\tab)"),
        ("mixed_number_formats", "混合数字格式", "(42 3.14 0 -5 -2.5)"),
        ("vector_with_many_elements", "多元素向量", "#(a b c d e f g h i j)"),
        ("mixed_vector_content", "混合内容向量", "#(42 \"hello\" #t (a b) #(1 2))"),
        ("vector_in_list", "列表中的向量", "(list #(1 2 3) #(a b))"),
        ("empty_strings_and_symbols", "空字符串", "\"\""),
        ("string_with_all_escapes", "包含所有转义字符的字符串", "\"\\\"\\\\\\n\\t\\r\""),
    ];
    
    let error_cases = vec![
        ("unclosed_paren", "未闭合的括号", "(define x"),
        ("extra_closing_paren", "多余的闭合括号", "(define x 42))"),
        ("unclosed_string", "未闭合的字符串", "\"hello"),
        ("invalid_dot_syntax", "无效的点语法", "(. a)"),
        ("multiple_dots", "多个点", "(a . b . c)"),
    ];
    
    println!("# 边界情况和错误处理测试 fixture");
    println!();
    println!("test_cases:");
    
    for (name, description, input) in test_cases {
        let output = parse_from_string(input);
        match &output.result {
            Ok(expressions) if expressions.len() == 1 => {
                let actual = expressions[0].to_pretty_string();
                println!("  - name: \"{}\"", name);
                println!("    description: \"{}\"", description);
                println!("    input: {:?}", input);
                
                if actual.contains('\n') {
                    println!("    expected: |");
                    for line in actual.lines() {
                        println!("      {}", line);
                    }
                } else {
                    println!("    expected: {:?}", actual);
                }
                println!();
            }
            Ok(expressions) => {
                eprintln!("ERROR: {} expressions for {}: {:?}", expressions.len(), name, input);
            }
            Err(err) => {
                eprintln!("ERROR: Parse failed for {}: {} (input: {:?})", name, err, input);
            }
        }
    }
    
    println!("error_cases:");
    for (name, description, input) in error_cases {
        println!("  - name: \"{}\"", name);
        println!("    description: \"{}\"", description);
        println!("    input: {:?}", input);
        println!("    should_fail: true");
        println!();
    }
}
