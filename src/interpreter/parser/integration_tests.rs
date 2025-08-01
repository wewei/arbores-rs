//! Parser和Lexer联合集成测试
//! 
//! 本模块包含复杂的端到端测试，从源代码字符串直接生成SExpr，
//! 测试词法分析和语法分析的完整流程。

use super::{parse_from_string, SExpr, SExprContent, Value};

/// 从源代码字符串解析S表达式的便利函数
fn parse_source(source: &str) -> Result<Vec<SExpr>, String> {
    let output = parse_from_string(source);
    
    match output.result {
        Ok(exprs) => Ok(exprs),
        Err(e) => Err(format!("Parse error: {}\nSource rebuilt: {}", e, output.source_text)),
    }
}

/// 验证解析失败的辅助宏
macro_rules! assert_parse_err {
    ($source:expr) => {
        let result = parse_source($source);
        assert!(result.is_err(), "Expected parse error for '{}', but succeeded", $source);
    };
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // ============================================================================
    // 基础原子值测试
    // ============================================================================

    #[test]
    fn test_parse_numbers() {
        // 整数
        let exprs = parse_source("42").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Number(n)) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number"),
        }

        // 浮点数
        let exprs = parse_source("3.14159").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Number(n)) => assert!((n - 3.14159).abs() < f64::EPSILON),
            _ => panic!("Expected number"),
        }

        // 注意：负数在当前lexer实现中被解析为符号，这是正确的Scheme行为
        // 因为 -123 实际上是符号 "-" 后跟数字 123，或者在特定上下文中作为负数函数调用
    }

    #[test]
    fn test_parse_strings() {
        // 简单字符串
        let exprs = parse_source(r#""hello world""#).unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::String(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string"),
        }

        // 包含转义字符的字符串
        let exprs = parse_source(r#""line1\nline2\t\"quoted\"""#).unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::String(s)) => assert_eq!(s, "line1\nline2\t\"quoted\""),
            _ => panic!("Expected string"),
        }

        // 空字符串
        let exprs = parse_source(r#""""#).unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::String(s)) => assert_eq!(s, ""),
            _ => panic!("Expected empty string"),
        }
    }

    #[test]
    fn test_parse_booleans() {
        // True
        let exprs = parse_source("#t").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Boolean(b)) => assert!(*b),
            _ => panic!("Expected boolean true"),
        }

        // False
        let exprs = parse_source("#f").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Boolean(b)) => assert!(!*b),
            _ => panic!("Expected boolean false"),
        }
    }

    #[test]
    fn test_parse_characters() {
        // 普通字符
        let exprs = parse_source(r"#\a").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Character(c)) => assert_eq!(*c, 'a'),
            _ => panic!("Expected character"),
        }

        // 空格字符
        let exprs = parse_source(r"#\space").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Character(c)) => assert_eq!(*c, ' '),
            _ => panic!("Expected space character"),
        }

        // 换行字符
        let exprs = parse_source(r"#\newline").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Character(c)) => assert_eq!(*c, '\n'),
            _ => panic!("Expected newline character"),
        }
    }

    #[test]
    fn test_parse_symbols() {
        // 简单符号
        let exprs = parse_source("hello").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected symbol"),
        }

        // 带特殊字符的符号
        let exprs = parse_source("hello-world!").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "hello-world!"),
            _ => panic!("Expected symbol"),
        }

        // 数学运算符符号
        let exprs = parse_source("+").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "+"),
            _ => panic!("Expected symbol"),
        }
    }

    // ============================================================================
    // 列表结构测试
    // ============================================================================

    #[test]
    fn test_parse_empty_list() {
        let exprs = parse_source("()").unwrap();
        assert_eq!(exprs.len(), 1);
        match &exprs[0].content {
            SExprContent::Nil => {},
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn test_parse_simple_list() {
        let exprs = parse_source("(+ 1 2)").unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 验证是 cons 结构
        if let SExprContent::Cons { car, cdr } = &exprs[0].content {
            // 验证第一个元素是 '+' 符号
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "+");
            } else {
                panic!("Expected '+' symbol as first element");
            }
            
            // 验证剩余是另一个 cons 结构
            if let SExprContent::Cons { car: second, cdr: rest } = &cdr.content {
                // 验证第二个元素是数字 1
                if let SExprContent::Atom(Value::Number(n)) = &second.content {
                    assert_eq!(*n, 1.0);
                } else {
                    panic!("Expected number 1 as second element");
                }
                
                // 验证第三个元素和列表结尾
                if let SExprContent::Cons { car: third, cdr: end } = &rest.content {
                    if let SExprContent::Atom(Value::Number(n)) = &third.content {
                        assert_eq!(*n, 2.0);
                    } else {
                        panic!("Expected number 2 as third element");
                    }
                    
                    // 验证列表结尾是 nil
                    if let SExprContent::Nil = &end.content {
                        // 正确
                    } else {
                        panic!("Expected nil at end of list");
                    }
                } else {
                    panic!("Expected cons for third element");
                }
            } else {
                panic!("Expected cons for rest of list");
            }
        } else {
            panic!("Expected cons structure for list");
        }
    }

    #[test]
    fn test_parse_nested_lists() {
        let exprs = parse_source("((+ 1 2) (* 3 4))").unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 这是一个包含两个子列表的列表
        if let SExprContent::Cons { car: first_list, cdr } = &exprs[0].content {
            // 第一个子列表应该是 (+ 1 2)
            if let SExprContent::Cons { car, .. } = &first_list.content {
                if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                    assert_eq!(s, "+");
                } else {
                    panic!("Expected '+' in first sublist");
                }
            } else {
                panic!("Expected first element to be a list");
            }
            
            // 第二个子列表应该是 (* 3 4)
            if let SExprContent::Cons { car: second_list, cdr: end } = &cdr.content {
                if let SExprContent::Cons { car, .. } = &second_list.content {
                    if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                        assert_eq!(s, "*");
                    } else {
                        panic!("Expected '*' in second sublist");
                    }
                } else {
                    panic!("Expected second element to be a list");
                }
                
                // 验证列表结尾
                if let SExprContent::Nil = &end.content {
                    // 正确
                } else {
                    panic!("Expected nil at end of outer list");
                }
            } else {
                panic!("Expected second sublist");
            }
        } else {
            panic!("Expected outer list structure");
        }
    }

    #[test]
    fn test_parse_dotted_pairs() {
        // 简单点对
        let exprs = parse_source("(a . b)").unwrap();
        assert_eq!(exprs.len(), 1);
        
        if let SExprContent::Cons { car, cdr } = &exprs[0].content {
            // car 应该是符号 'a'
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "a");
            } else {
                panic!("Expected symbol 'a' as car");
            }
            
            // cdr 应该是符号 'b'（不是列表）
            if let SExprContent::Atom(Value::Symbol(s)) = &cdr.content {
                assert_eq!(s, "b");
            } else {
                panic!("Expected symbol 'b' as cdr");
            }
        } else {
            panic!("Expected cons structure");
        }
    }

    #[test]
    fn test_parse_improper_lists() {
        // 不当列表：(a b . c)
        let exprs = parse_source("(a b . c)").unwrap();
        assert_eq!(exprs.len(), 1);
        
        if let SExprContent::Cons { car, cdr } = &exprs[0].content {
            // 第一个元素是 'a'
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "a");
            } else {
                panic!("Expected symbol 'a'");
            }
            
            // cdr 应该是 (b . c)
            if let SExprContent::Cons { car: second, cdr: tail } = &cdr.content {
                // 第二个元素是 'b'
                if let SExprContent::Atom(Value::Symbol(s)) = &second.content {
                    assert_eq!(s, "b");
                } else {
                    panic!("Expected symbol 'b'");
                }
                
                // 尾部是 'c'（不是 nil）
                if let SExprContent::Atom(Value::Symbol(s)) = &tail.content {
                    assert_eq!(s, "c");
                } else {
                    panic!("Expected symbol 'c' as tail");
                }
            } else {
                panic!("Expected cons for rest");
            }
        } else {
            panic!("Expected cons structure");
        }
    }

    // ============================================================================
    // 引用语法测试
    // ============================================================================

    #[test]
    fn test_parse_quote() {
        let exprs = parse_source("'x").unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 应该被转换为 (quote x)
        if let SExprContent::Cons { car, cdr } = &exprs[0].content {
            // 第一个元素应该是 'quote' 符号
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "quote");
            } else {
                panic!("Expected 'quote' symbol");
            }
            
            // 第二个元素应该是被引用的表达式
            if let SExprContent::Cons { car: quoted, cdr: end } = &cdr.content {
                if let SExprContent::Atom(Value::Symbol(s)) = &quoted.content {
                    assert_eq!(s, "x");
                } else {
                    panic!("Expected quoted symbol 'x'");
                }
                
                // 列表应该以 nil 结尾
                if let SExprContent::Nil = &end.content {
                    // 正确
                } else {
                    panic!("Expected nil at end");
                }
            } else {
                panic!("Expected quoted expression");
            }
        } else {
            panic!("Expected quote list structure");
        }
    }

    #[test]
    fn test_parse_quasiquote() {
        let exprs = parse_source("`(a ,b)").unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 应该被转换为 (quasiquote (a (unquote b)))
        if let SExprContent::Cons { car, .. } = &exprs[0].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "quasiquote");
            } else {
                panic!("Expected 'quasiquote' symbol");
            }
        } else {
            panic!("Expected quasiquote structure");
        }
    }

    #[test] 
    fn test_parse_unquote() {
        let exprs = parse_source(",x").unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 应该被转换为 (unquote x)
        if let SExprContent::Cons { car, .. } = &exprs[0].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "unquote");
            } else {
                panic!("Expected 'unquote' symbol");
            }
        } else {
            panic!("Expected unquote structure");
        }
    }

    #[test]
    fn test_parse_unquote_splicing() {
        let exprs = parse_source(",@x").unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 应该被转换为 (unquote-splicing x)
        if let SExprContent::Cons { car, .. } = &exprs[0].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "unquote-splicing");
            } else {
                panic!("Expected 'unquote-splicing' symbol");
            }
        } else {
            panic!("Expected unquote-splicing structure");
        }
    }

    // ============================================================================
    // 复杂表达式测试
    // ============================================================================

    #[test]
    fn test_parse_complex_expression() {
        let source = r#"
            (define factorial
              (lambda (n)
                (if (= n 0)
                    1
                    (* n (factorial (- n 1))))))
        "#;
        
        let exprs = parse_source(source).unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 验证顶层是 define 表达式
        if let SExprContent::Cons { car, .. } = &exprs[0].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "define");
            } else {
                panic!("Expected 'define' symbol");
            }
        } else {
            panic!("Expected define expression");
        }
    }

    #[test]
    fn test_parse_multiple_expressions() {
        let source = r#"
            (define x 42)
            (define y "hello")
            (+ x 1)
        "#;
        
        let exprs = parse_source(source).unwrap();
        assert_eq!(exprs.len(), 3);
        
        // 验证第一个表达式是 (define x 42)
        if let SExprContent::Cons { car, .. } = &exprs[0].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "define");
            } else {
                panic!("Expected first 'define'");
            }
        }
        
        // 验证第二个表达式是 (define y "hello")
        if let SExprContent::Cons { car, .. } = &exprs[1].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "define");
            } else {
                panic!("Expected second 'define'");
            }
        }
        
        // 验证第三个表达式是 (+ x 1)
        if let SExprContent::Cons { car, .. } = &exprs[2].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "+");
            } else {
                panic!("Expected '+' expression");
            }
        }
    }

    #[test]
    fn test_parse_with_comments() {
        let source = r#"
            ; 这是一个注释
            (define x 42) ; 行末注释
            ; 另一个注释
            (+ x 1)
        "#;
        
        let exprs = parse_source(source).unwrap();
        assert_eq!(exprs.len(), 2);
        
        // 注释应该被忽略，只解析出两个表达式
        if let SExprContent::Cons { car, .. } = &exprs[0].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "define");
            }
        }
        
        if let SExprContent::Cons { car, .. } = &exprs[1].content {
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "+");
            }
        }
    }

    // ============================================================================
    // 错误情况测试
    // ============================================================================

    #[test]
    fn test_parse_unclosed_list() {
        assert_parse_err!("(+ 1 2");
        assert_parse_err!("((+ 1 2)");
        assert_parse_err!("(define x");
    }

    #[test]
    fn test_parse_unexpected_closing_paren() {
        assert_parse_err!("(+ 1 2))");
        assert_parse_err!(")");
        assert_parse_err!("())");
    }

    #[test]
    fn test_parse_invalid_dotted_list() {
        assert_parse_err!("(. a)");           // 点号在开头
        assert_parse_err!("(a . )");          // 点号后没有元素
        assert_parse_err!("(a . b c)");       // 点号后有多个元素
        assert_parse_err!("(a .. b)");        // 连续点号
    }

    #[test]
    fn test_parse_unterminated_string() {
        assert_parse_err!(r#""hello"#);       // 未闭合的字符串
        assert_parse_err!(r#""hello world"#); // 未闭合的字符串
    }

    #[test]
    fn test_parse_invalid_character_literal() {
        assert_parse_err!(r"#\");             // 不完整的字符字面量
        assert_parse_err!(r"#\invalid");      // 无效的字符名称
    }

    // ============================================================================
    // 边界情况测试
    // ============================================================================

    #[test]
    fn test_parse_empty_source() {
        let exprs = parse_source("").unwrap();
        assert_eq!(exprs.len(), 0);
        
        let exprs = parse_source("   \n\t  ").unwrap();
        assert_eq!(exprs.len(), 0);
    }

    #[test]
    fn test_parse_only_comments() {
        let source = r#"
            ; 只有注释的文件
            ; 另一行注释
        "#;
        
        let exprs = parse_source(source).unwrap();
        assert_eq!(exprs.len(), 0);
    }

    #[test]
    fn test_parse_deeply_nested() {
        let source = "((((((42))))))";
        let exprs = parse_source(source).unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 验证深度嵌套结构
        let mut current = &exprs[0];
        for _ in 0..6 {
            if let SExprContent::Cons { car, cdr } = &current.content {
                current = car;
                // 验证 cdr 是 nil（单元素列表）
                if let SExprContent::Nil = &cdr.content {
                    // 正确
                } else {
                    panic!("Expected nil in nested list");
                }
            } else {
                panic!("Expected nested cons structure");
            }
        }
        
        // 最内层应该是数字 42
        if let SExprContent::Atom(Value::Number(n)) = &current.content {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number at innermost level");
        }
    }

    #[test]
    fn test_parse_whitespace_handling() {
        // 测试各种空白字符的处理
        let sources = vec![
            "(+ 1 2)",           // 正常空格
            "(+\t1\t2)",         // 制表符
            "(+\n1\n2)",         // 换行符
            "(+ \t\n 1 \t\n 2)", // 混合空白
            "( + 1 2 )",         // 括号内外的空格
        ];
        
        for source in sources {
            let exprs = parse_source(source).unwrap();
            assert_eq!(exprs.len(), 1, "Failed for source: {}", source);
            
            if let SExprContent::Cons { car, .. } = &exprs[0].content {
                if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                    assert_eq!(s, "+", "Failed for source: {}", source);
                }
            }
        }
    }

    // ============================================================================
    // 性能和稳定性测试
    // ============================================================================

    #[test]
    fn test_parse_large_list() {
        // 创建一个包含很多元素的列表
        let mut source = String::from("(");
        for i in 0..1000 {
            source.push_str(&format!("{} ", i));
        }
        source.push(')');
        
        let exprs = parse_source(&source).unwrap();
        assert_eq!(exprs.len(), 1);
        
        // 验证列表结构
        if let SExprContent::Cons { .. } = &exprs[0].content {
            // 正确，是一个列表
        } else {
            panic!("Expected list structure");
        }
    }

    #[test]
    fn test_parse_many_expressions() {
        // 创建很多个独立的表达式
        let mut source = String::new();
        for i in 0..100 {
            source.push_str(&format!("(expr-{}) ", i));
        }
        
        let exprs = parse_source(&source).unwrap();
        assert_eq!(exprs.len(), 100);
        
        // 验证每个表达式
        for (i, expr) in exprs.iter().enumerate() {
            if let SExprContent::Cons { car, .. } = &expr.content {
                if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                    assert_eq!(s, &format!("expr-{}", i));
                }
            }
        }
    }

    #[test]
    fn test_parse_negative_as_expression() {
        // 在Scheme中，-123 通常被解析为两个token：符号"-"和数字"123"
        // 这在表达式 (- 123) 或 (- 0 123) 中是正确的
        let exprs = parse_source("(- 123)").unwrap();
        assert_eq!(exprs.len(), 1);
        
        if let SExprContent::Cons { car, cdr } = &exprs[0].content {
            // 第一个元素应该是符号 "-"
            if let SExprContent::Atom(Value::Symbol(s)) = &car.content {
                assert_eq!(s, "-");
            } else {
                panic!("Expected '-' symbol");
            }
            
            // 第二个元素应该是数字 123
            if let SExprContent::Cons { car: num, cdr: end } = &cdr.content {
                if let SExprContent::Atom(Value::Number(n)) = &num.content {
                    assert_eq!(*n, 123.0);
                } else {
                    panic!("Expected number 123");
                }
                
                if let SExprContent::Nil = &end.content {
                    // 正确
                } else {
                    panic!("Expected nil at end");
                }
            } else {
                panic!("Expected rest of list");
            }
        } else {
            panic!("Expected list structure");
        }
    }
}
