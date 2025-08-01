use arbores::interpreter::parser::parse_from_string;

fn main() {
    let edge_cases = vec![
        ("nested_quotes", "''x"),
        ("mixed_quotes", "'`,@x"),
        ("deeply_nested_list", "(((())))" ),
        ("list_with_many_elements", "(a b c d e f g h i j)"),
        ("complex_dotted_structure", "(a b . (c d . e))"),
        ("zero_and_negative_numbers", "(0 -1 -3.14)"),
        ("special_characters_in_symbols", "(+ - * / = < > <= >= eq? null?)"),
        ("character_literals", "(#\\a #\\space #\\newline #\\tab)"),
        ("mixed_number_formats", "(42 3.14 0 -5 -2.5)"),
        ("empty_strings_and_symbols", "\"\""),
        ("string_with_all_escapes", "\"\\\"\\\\\\n\\t\\r\""),
    ];
    
    for (name, input) in edge_cases {
        println!("  - name: \"{}\"", name);
        println!("    input: {:?}", input);
        
        let output = parse_from_string(input);
        match output.result {
            Ok(expressions) => {
                if expressions.len() == 1 {
                    let actual = expressions[0].to_pretty_string();
                    // 检查是否是多行
                    if actual.contains('\n') {
                        println!("    expected: |");
                        for line in actual.lines() {
                            println!("      {}", line);
                        }
                    } else {
                        println!("    expected: {:?}", actual);
                    }
                } else {
                    println!("    # ERROR: {} expressions", expressions.len());
                }
            }
            Err(err) => {
                println!("    # ERROR: {}", err);
            }
        }
        println!();
    }
}
