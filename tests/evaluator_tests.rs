//! Evaluator 测试

use std::rc::Rc;
use arbores::interpreter::evaluator::*;
use arbores::interpreter::evaluator::builtins::create_global_environment;
use arbores::interpreter::{SExpr, SExprContent, Value};
use arbores::interpreter::lexer::types::Span;

fn create_test_span() -> Span {
    Span { start: 0, end: 0 }
}

#[test]
fn test_evaluate_numbers() {
    let env = Environment::new();
    let expr = SExpr::with_span(
        SExprContent::Atom(Value::Number(42.0)),
        create_test_span(),
    );
    
    let result = evaluate(expr, env).unwrap();
    assert_eq!(result, RuntimeValue::Number(42.0));
}

#[test]
fn test_evaluate_strings() {
    let env = Environment::new();
    let expr = SExpr::with_span(
        SExprContent::Atom(Value::String("hello".to_string())),
        create_test_span(),
    );
    
    let result = evaluate(expr, env).unwrap();
    assert_eq!(result, RuntimeValue::String("hello".to_string()));
}

#[test]
fn test_evaluate_booleans() {
    let env = Environment::new();
    let expr = SExpr::with_span(
        SExprContent::Atom(Value::Boolean(true)),
        create_test_span(),
    );
    
    let result = evaluate(expr, env).unwrap();
    assert_eq!(result, RuntimeValue::Boolean(true));
}

#[test]
fn test_evaluate_nil() {
    let env = Environment::new();
    let expr = SExpr::with_span(
        SExprContent::Nil,
        create_test_span(),
    );
    
    let result = evaluate(expr, env).unwrap();
    assert_eq!(result, RuntimeValue::Nil);
}

#[test]
fn test_quote_special_form() {
    let env = Environment::new();
    let quoted_expr = SExpr::with_span(
        SExprContent::Atom(Value::Symbol("x".to_string())),
        create_test_span(),
    );
    let quote_expr = SExpr::with_span(
        SExprContent::Cons {
            car: Rc::new(SExpr::with_span(
                SExprContent::Atom(Value::Symbol("quote".to_string())),
                create_test_span(),
            )),
            cdr: Rc::new(SExpr::with_span(
                SExprContent::Cons {
                    car: Rc::new(quoted_expr),
                    cdr: Rc::new(SExpr::with_span(SExprContent::Nil, create_test_span())),
                },
                create_test_span(),
            )),
        },
        create_test_span(),
    );
    
    let result = evaluate(quote_expr, env).unwrap();
    assert_eq!(result, RuntimeValue::Symbol("x".to_string()));
}

#[test]
fn test_arithmetic_functions() {
    let env = create_global_environment();
    
    // 测试加法
    let add_expr = SExpr::with_span(
        SExprContent::Cons {
            car: Rc::new(SExpr::with_span(
                SExprContent::Atom(Value::Symbol("+".to_string())),
                create_test_span(),
            )),
            cdr: Rc::new(SExpr::with_span(
                SExprContent::Cons {
                    car: Rc::new(SExpr::with_span(
                        SExprContent::Atom(Value::Number(3.0)),
                        create_test_span(),
                    )),
                    cdr: Rc::new(SExpr::with_span(
                        SExprContent::Cons {
                            car: Rc::new(SExpr::with_span(
                                SExprContent::Atom(Value::Number(4.0)),
                                create_test_span(),
                            )),
                            cdr: Rc::new(SExpr::with_span(SExprContent::Nil, create_test_span())),
                        },
                        create_test_span(),
                    )),
                },
                create_test_span(),
            )),
        },
        create_test_span(),
    );
    
    let result = evaluate(add_expr, env).unwrap();
    assert_eq!(result, RuntimeValue::Number(7.0));
}
