//! Evaluator 性能基准测试

use std::time::Instant;
use std::rc::Rc;
use arbores::interpreter::evaluator::*;
use arbores::interpreter::evaluator::state::init_eval_state;
use arbores::interpreter::{SExpr, SExprContent, Value};
use arbores::interpreter::lexer::types::Span;

fn create_test_span() -> Span {
    Span { start: 0, end: 0 }
}

fn create_simple_expr() -> SExpr {
    SExpr::with_span(
        SExprContent::Atom(Value::Number(42.0)),
        create_test_span(),
    )
}

fn create_nested_expr(depth: usize) -> SExpr {
    let mut expr = create_simple_expr();
    for _ in 0..depth {
        expr = SExpr::with_span(
            SExprContent::Cons {
                car: Rc::new(expr),
                cdr: Rc::new(SExpr::with_span(SExprContent::Nil, create_test_span())),
            },
            create_test_span(),
        );
    }
    expr
}

#[test]
fn benchmark_eval_state_rc_clone() {
    let env = Environment::new();
    let expr = create_simple_expr();
    let state = Rc::new(init_eval_state(expr, env));
    
    let iterations = 100_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _cloned_state = state.clone();
    }
    
    let duration = start.elapsed();
    println!("Rc<EvalState> clone benchmark:");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", duration);
    println!("  Average time per clone: {:?}", duration / iterations);
    println!("  Clones per second: {:.0}", iterations as f64 / duration.as_secs_f64());
}

#[test]
fn benchmark_environment_clone() {
    let mut env = Environment::new();
    env.define("x".to_string(), RuntimeValue::Number(1.0));
    env.define("y".to_string(), RuntimeValue::Number(2.0));
    env.define("z".to_string(), RuntimeValue::Number(3.0));
    
    let iterations = 100_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _cloned_env = env.clone();
    }
    
    let duration = start.elapsed();
    println!("Environment clone benchmark:");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", duration);
    println!("  Average time per clone: {:?}", duration / iterations);
    println!("  Clones per second: {:.0}", iterations as f64 / duration.as_secs_f64());
}

#[test]
fn benchmark_s_expr_clone() {
    let expr = create_nested_expr(10); // 创建深度为10的嵌套表达式
    
    let iterations = 100_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _cloned_expr = expr.clone();
    }
    
    let duration = start.elapsed();
    println!("SExpr clone benchmark (depth 10):");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", duration);
    println!("  Average time per clone: {:?}", duration / iterations);
    println!("  Clones per second: {:.0}", iterations as f64 / duration.as_secs_f64());
}

#[test]
fn benchmark_frame_clone() {
    let env = Environment::new();
    let continuation = Continuation {
        func: Rc::new(|_| EvaluateResult::Completed(RuntimeValue::Number(0.0))),
    };
    let frame = Frame {
        env: Rc::new(env),
        continuation,
        parent: None,
    };
    
    let iterations = 100_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _cloned_frame = frame.clone();
    }
    
    let duration = start.elapsed();
    println!("Frame clone benchmark:");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", duration);
    println!("  Average time per clone: {:?}", duration / iterations);
    println!("  Clones per second: {:.0}", iterations as f64 / duration.as_secs_f64());
}

#[test]
fn benchmark_frame_with_parent_clone() {
    let parent_env = Environment::new();
    let parent_continuation = Continuation {
        func: Rc::new(|_| EvaluateResult::Completed(RuntimeValue::Number(0.0))),
    };
    let parent_frame = Frame {
        env: Rc::new(parent_env),
        continuation: parent_continuation,
        parent: None,
    };
    
    let child_env = Environment::new();
    let child_continuation = Continuation {
        func: Rc::new(|_| EvaluateResult::Completed(RuntimeValue::Number(0.0))),
    };
    let child_frame = Frame {
        env: Rc::new(child_env),
        continuation: child_continuation,
        parent: Some(Rc::new(parent_frame)),
    };
    
    let iterations = 100_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _cloned_frame = child_frame.clone();
    }
    
    let duration = start.elapsed();
    println!("Frame with parent clone benchmark:");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", duration);
    println!("  Average time per clone: {:?}", duration / iterations);
    println!("  Clones per second: {:.0}", iterations as f64 / duration.as_secs_f64());
}

#[test]
fn benchmark_runtime_value_clone() {
    let values = vec![
        RuntimeValue::Number(42.0),
        RuntimeValue::String("hello world".to_string()),
        RuntimeValue::Boolean(true),
        RuntimeValue::Symbol("x".to_string()),
        RuntimeValue::Cons {
            car: Rc::new(RuntimeValue::Number(1.0)),
            cdr: Rc::new(RuntimeValue::Number(2.0)),
        },
        RuntimeValue::Vector(Rc::new(vec![
            RuntimeValue::Number(1.0),
            RuntimeValue::Number(2.0),
            RuntimeValue::Number(3.0),
        ])),
    ];
    
    let iterations = 100_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        for value in &values {
            let _cloned_value = value.clone();
        }
    }
    
    let duration = start.elapsed();
    println!("RuntimeValue clone benchmark:");
    println!("  Iterations: {}", iterations);
    println!("  Values per iteration: {}", values.len());
    println!("  Total clones: {}", iterations * values.len());
    println!("  Total time: {:?}", duration);
    let total_clones = iterations * values.len();
    println!("  Average time per clone: {:?}", duration / total_clones as u32);
    println!("  Clones per second: {:.0}", (iterations * values.len()) as f64 / duration.as_secs_f64());
}

#[test]
fn benchmark_memory_usage() {
    use std::mem;
    
    println!("Memory usage analysis:");
    println!("  EvalState: {} bytes", mem::size_of::<EvalState>());
    println!("  Frame: {} bytes", mem::size_of::<Frame>());
    println!("  Environment: {} bytes", mem::size_of::<Environment>());
    println!("  SExpr: {} bytes", mem::size_of::<SExpr>());
    println!("  RuntimeValue: {} bytes", mem::size_of::<RuntimeValue>());
    println!("  Continuation: {} bytes", mem::size_of::<Continuation>());
    println!("  TailContext: {} bytes", mem::size_of::<TailContext>());
}
