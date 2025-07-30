# 函数式 Rust 代码规范

## 1. 设计哲学

### 1.1 函数式优先

- 优先使用纯函数，避免副作用
- 使用不可变数据结构
- 通过组合而非继承来构建复杂功能
- 优先使用表达式而非语句

### 1.2 数据类型设计

- **Struct 仅用于定义数据类型**：不用于面向对象的多态实现
- **优先使用代数数据类型**：通过 `enum` 定义复杂的数据类型和状态

## 2. 数据类型定义规范

### 2.1 Struct 使用规范

Struct 应当仅用于聚合相关数据，不应包含方法（除了基础的构造函数）：

```rust
// ✅ 正确：纯数据结构
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

// ✅ 允许：简单的构造函数
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

// ❌ 避免：在 struct 上定义业务逻辑方法
impl Point {
    // 不推荐：应该定义为独立函数
    pub fn distance_to(&self, other: &Point) -> f64 {
        // ...
    }
}
```

### 2.2 优先使用代数数据类型 (ADT)

使用 `enum` 来表示复杂的数据类型和状态机：

```rust
// ✅ 优秀：使用 enum 表示不同状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting { timeout: u64 },
    Connected { session_id: String },
    Failed { error: String, retry_count: u32 },
}

// ✅ 优秀：递归数据结构
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Symbol(String),
    List(Vec<Expr>),
    Function { name: String, args: Vec<Expr> },
}

// ✅ 优秀：Result 类型的扩展
#[derive(Debug, Clone)]
pub enum ParseResult<T> {
    Success(T),
    Warning { value: T, message: String },
    Error { message: String, position: usize },
}
```

## 3. 函数设计规范

### 3.1 纯函数优先

```rust
// ✅ 优秀：纯函数，无副作用
pub fn calculate_distance(p1: &Point, p2: &Point) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    (dx * dx + dy * dy).sqrt()
}

// ✅ 优秀：函数式数据变换
pub fn transform_points(points: &[Point], offset: Point) -> Vec<Point> {
    points.iter()
        .map(|p| Point { x: p.x + offset.x, y: p.y + offset.y })
        .collect()
}

// ❌ 避免：修改输入参数
pub fn bad_transform(points: &mut [Point], offset: Point) {
    for point in points {
        point.x += offset.x;
        point.y += offset.y;
    }
}
```

### 3.2 错误处理

使用 `Result` 类型进行错误处理，优先使用组合子：

```rust
// ✅ 优秀：使用 Result 和组合子
pub fn parse_and_evaluate(input: &str) -> Result<Value, EvalError> {
    tokenize(input)
        .and_then(parse)
        .and_then(evaluate)
}

// ✅ 优秀：自定义错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken { token: String, position: usize },
    UnbalancedParentheses { position: usize },
    InvalidNumber { text: String, position: usize },
}
```

### 3.3 高阶函数

充分利用 Rust 的闭包和高阶函数：

```rust
// ✅ 优秀：高阶函数
pub fn filter_map_collect<T, U, F, P>(
    items: &[T],
    predicate: P,
    mapper: F,
) -> Vec<U>
where
    F: Fn(&T) -> U,
    P: Fn(&T) -> bool,
{
    items.iter()
        .filter(|item| predicate(item))
        .map(mapper)
        .collect()
}

// ✅ 优秀：函数作为参数
pub fn fold_expr<T, F>(expr: &Expr, f: F, init: T) -> T
where
    F: Fn(T, &Expr) -> T,
{
    match expr {
        Expr::List(exprs) => exprs.iter().fold(f(init, expr), |acc, e| fold_expr(e, &f, acc)),
        _ => f(init, expr),
    }
}
```

## 4. 模式匹配规范

### 4.1 穷尽匹配

```rust
// ✅ 优秀：穷尽匹配
pub fn evaluate_expr(expr: &Expr, env: &Environment) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
        Expr::Symbol(name) => env.lookup(name).ok_or_else(|| {
            EvalError::UndefinedVariable { name: name.clone() }
        }),
        Expr::List(exprs) if exprs.is_empty() => {
            Err(EvalError::EmptyList)
        },
        Expr::List(exprs) => evaluate_function_call(exprs, env),
        Expr::Function { name, args } => evaluate_function(name, args, env),
    }
}

// ✅ 优秀：嵌套模式匹配
pub fn optimize_expr(expr: Expr) -> Expr {
    match expr {
        Expr::List(ref exprs) if exprs.len() == 3 => {
            match (&exprs[0], &exprs[1], &exprs[2]) {
                (Expr::Symbol(op), Expr::Literal(Value::Number(a)), Expr::Literal(Value::Number(b)))
                    if op == "+" => Expr::Literal(Value::Number(a + b)),
                _ => expr,
            }
        },
        _ => expr,
    }
}
```

## 5. 不可变性规范

### 5.1 默认不可变

```rust
// ✅ 优秀：默认不可变
pub fn process_data(input: &[String]) -> Vec<ProcessedItem> {
    let filtered: Vec<_> = input.iter()
        .filter(|s| !s.is_empty())
        .collect();
    
    let processed: Vec<_> = filtered.iter()
        .map(|s| ProcessedItem::from_string(s))
        .collect();
    
    processed
}

// ✅ 优秀：通过 Clone 实现"修改"
pub fn update_environment(env: &Environment, name: String, value: Value) -> Environment {
    let mut new_env = env.clone();
    new_env.bindings.insert(name, value);
    new_env
}
```

### 5.2 使用 Cow 优化性能

```rust
use std::borrow::Cow;

// ✅ 优秀：使用 Cow 避免不必要的克隆
pub fn normalize_symbol<'a>(symbol: &'a str) -> Cow<'a, str> {
    if symbol.chars().all(|c| c.is_lowercase()) {
        Cow::Borrowed(symbol)
    } else {
        Cow::Owned(symbol.to_lowercase())
    }
}
```

## 6. 组合与抽象

### 6.1 通过函数组合构建复杂功能

```rust
// ✅ 优秀：函数组合
pub fn compile_and_run(source: &str) -> Result<Value, Error> {
    let tokens = tokenize(source)?;
    let ast = parse(tokens)?;
    let optimized = optimize(ast)?;
    let bytecode = compile(optimized)?;
    execute(bytecode)
}

// ✅ 优秀：使用 trait 进行抽象，但避免多态
pub trait Parser<T> {
    type Error;
    fn parse(&self, input: &str) -> Result<T, Self::Error>;
}

// 为具体类型实现 trait
impl Parser<Expr> for ExprParser {
    type Error = ParseError;
    fn parse(&self, input: &str) -> Result<Expr, ParseError> {
        // 实现
    }
}
```

### 6.2 使用 newtype 模式

```rust
// ✅ 优秀：类型安全的包装器
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableName(String);

#[derive(Debug, Clone, PartialEq)]
pub struct LineNumber(usize);

impl VariableName {
    pub fn new(name: String) -> Result<Self, ValidationError> {
        if name.is_empty() {
            Err(ValidationError::EmptyName)
        } else {
            Ok(VariableName(name))
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

## 7. 测试规范

### 7.1 属性驱动测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    // ✅ 优秀：属性驱动测试
    proptest! {
        #[test]
        fn distance_is_symmetric(p1: Point, p2: Point) {
            let d1 = calculate_distance(&p1, &p2);
            let d2 = calculate_distance(&p2, &p1);
            assert!((d1 - d2).abs() < f64::EPSILON);
        }
        
        #[test]
        fn parse_and_unparse_roundtrip(expr: Expr) {
            let unparsed = unparse(&expr);
            let reparsed = parse(&unparsed).unwrap();
            assert_eq!(expr, reparsed);
        }
    }
}
```

### 7.2 纯函数测试

```rust
#[test]
fn test_expression_evaluation() {
    let env = Environment::new();
    let expr = Expr::List(vec![
        Expr::Symbol("+".to_string()),
        Expr::Literal(Value::Number(1.0)),
        Expr::Literal(Value::Number(2.0)),
    ]);
    
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(3.0));
}
```

## 8. 性能考虑

### 8.1 避免不必要的分配

```rust
// ✅ 优秀：使用迭代器避免中间分配
pub fn count_symbols(exprs: &[Expr]) -> usize {
    exprs.iter()
        .flat_map(|expr| expr.symbols())
        .filter(|symbol| symbol.starts_with('_'))
        .count()
}

// ✅ 优秀：使用 slice 而非 Vec
pub fn first_non_empty<T>(slices: &[&[T]]) -> Option<&[T]> {
    slices.iter()
        .find(|slice| !slice.is_empty())
        .copied()
}
```

## 9. 文档规范

### 9.1 函数文档

```rust
/// 计算两点之间的欧几里得距离
/// 
/// # 参数
/// 
/// * `p1` - 第一个点
/// * `p2` - 第二个点
/// 
/// # 返回值
/// 
/// 返回两点之间的距离，总是非负数
/// 
/// # 示例
/// 
/// ```rust
/// let p1 = Point::new(0.0, 0.0);
/// let p2 = Point::new(3.0, 4.0);
/// assert_eq!(calculate_distance(&p1, &p2), 5.0);
/// ```
pub fn calculate_distance(p1: &Point, p2: &Point) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    (dx * dx + dy * dy).sqrt()
}
```

## 10. 总结

这套规范强调：

1. **数据与行为分离**：struct 仅定义数据，行为通过独立函数实现
2. **代数数据类型优先**：使用 enum 表示复杂状态和数据变体
3. **函数式编程**：纯函数、不可变性、组合子
4. **类型安全**：充分利用 Rust 的类型系统
5. **测试友好**：纯函数便于测试和推理

遵循这些规范将产生更易维护、测试和理解的函数式 Rust 代码。
