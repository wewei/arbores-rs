#[cfg(test)]
mod generate_expected_outputs {
    use arbores::interpreter::parser::parse_from_string;
    
    #[test]
    fn generate_basic_expressions() {
        let test_cases = vec![
            ("simple_integer", "42"),
            ("simple_float", "3.14"),
            ("simple_string", "\"hello\""),
            ("simple_symbol", "foo"),
            ("boolean_true", "#t"),
            ("boolean_false", "#f"),
            ("character", "#\\a"),
            ("empty_list", "()"),
            ("simple_list", "(1 2 3)"),
            ("nested_list", "(a (b c) d)"),
            ("quote_expression", "'x"),
            ("quote_list", "'(a b)"),
            ("quasiquote_expression", "`x"),
            ("unquote_expression", ",x"),
            ("unquote_splicing", ",@x"),
            ("dotted_pair", "(a . b)"),
            ("string_with_escapes", "\"hello\\nworld\""),
            ("multiline_expression", "(define x\n  42)"),
        ];
        
        println!("# Updated fixture expectations:");
        for (name, input) in test_cases {
            let output = parse_from_string(input);
            match &output.result {
                Ok(expressions) if expressions.len() == 1 => {
                    let actual = expressions[0].to_pretty_string();
                    println!("  - name: \"{}\"", name);
                    println!("    input: {:?}", input);
                    println!("    expected: {:?}", actual);
                    println!();
                }
                Ok(expressions) => {
                    println!("  # ERROR: {} expressions for {}: {:?}", expressions.len(), name, input);
                }
                Err(err) => {
                    println!("  # ERROR: Parse failed for {}: {} (input: {:?})", name, err, input);
                }
            }
        }
    }
}
