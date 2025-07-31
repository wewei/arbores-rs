use crate::legacy::types::Value;
use crate::legacy::env::Environment;
use crate::legacy::builtins;

/// 注册所有内置函数到环境中
pub fn register_builtins(env: &Environment) {
    // 算术运算
    env.define("+".to_string(), Value::BuiltinFunction {
        name: "+".to_string(),
        func: builtins::add,
        arity: None, // 可变参数
    }).unwrap();
    
    env.define("-".to_string(), Value::BuiltinFunction {
        name: "-".to_string(),
        func: builtins::subtract,
        arity: None,
    }).unwrap();
    
    env.define("*".to_string(), Value::BuiltinFunction {
        name: "*".to_string(),
        func: builtins::multiply,
        arity: None,
    }).unwrap();
    
    env.define("/".to_string(), Value::BuiltinFunction {
        name: "/".to_string(),
        func: builtins::divide,
        arity: None,
    }).unwrap();

    // 比较运算
    env.define("=".to_string(), Value::BuiltinFunction {
        name: "=".to_string(),
        func: builtins::equal,
        arity: Some(2),
    }).unwrap();
    
    env.define("<".to_string(), Value::BuiltinFunction {
        name: "<".to_string(),
        func: builtins::less_than,
        arity: Some(2),
    }).unwrap();
    
    env.define(">".to_string(), Value::BuiltinFunction {
        name: ">".to_string(),
        func: builtins::greater_than,
        arity: Some(2),
    }).unwrap();
    
    env.define("<=".to_string(), Value::BuiltinFunction {
        name: "<=".to_string(),
        func: builtins::less_equal,
        arity: Some(2),
    }).unwrap();
    
    env.define(">=".to_string(), Value::BuiltinFunction {
        name: ">=".to_string(),
        func: builtins::greater_equal,
        arity: Some(2),
    }).unwrap();

    // 数学函数
    env.define("abs".to_string(), Value::BuiltinFunction {
        name: "abs".to_string(),
        func: builtins::abs_func,
        arity: Some(1),
    }).unwrap();
    
    env.define("max".to_string(), Value::BuiltinFunction {
        name: "max".to_string(),
        func: builtins::max_func,
        arity: None,
    }).unwrap();
    
    env.define("min".to_string(), Value::BuiltinFunction {
        name: "min".to_string(),
        func: builtins::min_func,
        arity: None,
    }).unwrap();

    // 列表操作
    env.define("cons".to_string(), Value::BuiltinFunction {
        name: "cons".to_string(),
        func: builtins::cons,
        arity: Some(2),
    }).unwrap();
    
    env.define("car".to_string(), Value::BuiltinFunction {
        name: "car".to_string(),
        func: builtins::car,
        arity: Some(1),
    }).unwrap();
    
    env.define("cdr".to_string(), Value::BuiltinFunction {
        name: "cdr".to_string(),
        func: builtins::cdr,
        arity: Some(1),
    }).unwrap();
    
    env.define("list".to_string(), Value::BuiltinFunction {
        name: "list".to_string(),
        func: builtins::list,
        arity: None,
    }).unwrap();

    // 类型谓词
    env.define("null?".to_string(), Value::BuiltinFunction {
        name: "null?".to_string(),
        func: builtins::is_null,
        arity: Some(1),
    }).unwrap();
    
    env.define("pair?".to_string(), Value::BuiltinFunction {
        name: "pair?".to_string(),
        func: builtins::is_pair,
        arity: Some(1),
    }).unwrap();
    
    env.define("number?".to_string(), Value::BuiltinFunction {
        name: "number?".to_string(),
        func: builtins::is_number,
        arity: Some(1),
    }).unwrap();
    
    env.define("symbol?".to_string(), Value::BuiltinFunction {
        name: "symbol?".to_string(),
        func: builtins::is_symbol,
        arity: Some(1),
    }).unwrap();
    
    env.define("string?".to_string(), Value::BuiltinFunction {
        name: "string?".to_string(),
        func: builtins::is_string,
        arity: Some(1),
    }).unwrap();
} 