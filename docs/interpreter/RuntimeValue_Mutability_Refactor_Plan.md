# RuntimeValue 可变性重构计划

## 概述

当前 Arbores 的求值器直接处理 `SExpr`（语法表达式），但根据 R7RS Scheme 标准，运行时需要支持可变的数据结构。本次重构的核心思路是：

**求值器应该处理 `RuntimeValue` 而不是 `SExpr`**

`RuntimeValue` 本质上是 `SExpr` 的运行时表示，两者一一对应，只有两个关键区别：
1. **引用计数机制**：`Rc` 改为 `Gc`，以支持可变 `RuntimeValue` 的垃圾回收
2. **内置函数支持**：`RuntimeValue` 包含 `BuiltinFunction` 类型，而 `SExpr` 不包含

## RuntimeValue 三类分类

根据可变性需求，RuntimeValue 可以分为三类：

### 1. 原子值（Atomic Values）
包括：`float`, `integer`, `char`, `nil`
- **特点**：可以直接复制，不需要引用计数
- **实现**：直接存储值，使用 `Copy` trait
- **优势**：性能最佳，无内存管理开销

### 2. 不可变引用（Immutable References）
包括：`string`, `symbol`, `builtin-function`
- **特点**：内容不可变，但需要共享引用
- **实现**：使用 `Rc` 引用计数
- **优势**：支持共享，避免不必要的克隆

### 3. 可变引用（Mutable References）
包括：`cons`, `vector`, `lambda`
- **特点**：内容可变，需要垃圾回收
- **实现**：使用 `Gc` 垃圾回收
- **优势**：支持可变操作，自动内存管理

## 设计理念

### 编译时 vs 运行时

```
源代码 → SExpr (语法结构) → RuntimeValue (运行时值) → 求值结果
   ↓           ↓                    ↓
  解析      语法分析             运行时执行
```

- **SExpr**：编译时的语法结构，不可变，用于表示从源代码解析出的语法树
- **RuntimeValue**：运行时的值，支持可变性，用于实际的求值过程

### 转换过程

从 `SExpr` 到 `RuntimeValue` 的转换是一个"加载"过程：

```rust
/// 将 SExpr 转换为 RuntimeValue
fn load_sexpr_to_runtime(expr: &SExpr) -> RuntimeValue {
    match &expr.content {
        SExprContent::Atom(value) => value.into(),
        SExprContent::Cons { car, cdr } => {
            let car_runtime = load_sexpr_to_runtime(car);
            let cdr_runtime = load_sexpr_to_runtime(cdr);
            RuntimeValue::Cons(Gc::new(MutableCons::new(
                Gc::new(car_runtime), 
                Gc::new(cdr_runtime)
            )))
        },
        SExprContent::Nil => RuntimeValue::Nil,
        SExprContent::Vector(elements) => {
            let elements_runtime: Vec<Gc<RuntimeValue>> = elements
                .iter()
                .map(|e| Gc::new(load_sexpr_to_runtime(e)))
                .collect();
            RuntimeValue::Vector(Gc::new(MutableVector::new(elements_runtime)))
        },
    }
}
```

## 重构目标

1. **架构清晰**：明确区分编译时（SExpr）和运行时（RuntimeValue）
2. **支持可变性**：使用 `gc` crate 支持 `set-car!`、`set-cdr!`、`vector-set!` 等操作
3. **保持性能**：最小化 GC 开销，只在需要可变性的地方使用 GC
4. **向后兼容**：保持现有的 API 接口不变
5. **类型安全**：确保可变操作的类型安全

## 技术方案

### 1. 依赖更新

```toml
[dependencies]
gc = { version = "0.5", features = ["derive"] }
```

### 2. 核心数据结构设计

#### 2.1 可变 Cons 结构
```rust
/// 可变 Cons 结构 - 支持 set-car! 和 set-cdr! 操作
#[derive(Trace, Finalize, Debug, Clone)]
pub struct MutableCons {
    /// car 部分 - 使用 GcCell 支持可变性
    pub car: GcCell<Gc<RuntimeValue>>,
    /// cdr 部分 - 使用 GcCell 支持可变性
    pub cdr: GcCell<Gc<RuntimeValue>>,
}

impl MutableCons {
    /// 创建新的可变 Cons 结构
    pub fn new(car: Gc<RuntimeValue>, cdr: Gc<RuntimeValue>) -> Self {
        Self {
            car: GcCell::new(car),
            cdr: GcCell::new(cdr),
        }
    }
    
    /// 获取 car 值
    pub fn car(&self) -> Gc<RuntimeValue> {
        self.car.borrow().clone()
    }
    
    /// 获取 cdr 值
    pub fn cdr(&self) -> Gc<RuntimeValue> {
        self.cdr.borrow().clone()
    }
    
    /// 设置 car 值 (set-car!)
    pub fn set_car(&self, value: Gc<RuntimeValue>) {
        *self.car.borrow_mut() = value;
    }
    
    /// 设置 cdr 值 (set-cdr!)
    pub fn set_cdr(&self, value: Gc<RuntimeValue>) {
        *self.cdr.borrow_mut() = value;
    }
}
```

#### 2.2 可变 Vector 结构
```rust
/// 可变 Vector 结构 - 支持 vector-set! 操作
#[derive(Trace, Finalize, Debug, Clone)]
pub struct MutableVector {
    /// 向量元素 - 使用 GcCell 支持可变性
    pub elements: GcCell<Vec<Gc<RuntimeValue>>>,
}

impl MutableVector {
    /// 创建新的可变 Vector
    pub fn new(elements: Vec<Gc<RuntimeValue>>) -> Self {
        Self {
            elements: GcCell::new(elements),
        }
    }
    
    /// 获取向量长度
    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }
    
    /// 检查向量是否为空
    pub fn is_empty(&self) -> bool {
        self.elements.borrow().is_empty()
    }
    
    /// 获取指定索引的元素
    pub fn get(&self, index: usize) -> Option<Gc<RuntimeValue>> {
        self.elements.borrow().get(index).cloned()
    }
    
    /// 设置指定索引的元素 (vector-set!)
    pub fn set(&self, index: usize, value: Gc<RuntimeValue>) -> Result<(), String> {
        let mut elements = self.elements.borrow_mut();
        if index < elements.len() {
            elements[index] = value;
            Ok(())
        } else {
            Err(format!("Index {} out of bounds for vector of length {}", index, elements.len()))
        }
    }
    
    /// 添加元素到向量末尾
    pub fn push(&self, value: Gc<RuntimeValue>) {
        self.elements.borrow_mut().push(value);
    }
    
    /// 获取所有元素的克隆
    pub fn to_vec(&self) -> Vec<Gc<RuntimeValue>> {
        self.elements.borrow().clone()
    }
}
```

#### 2.3 环境结构（可变，用 Gc 包装）
```rust
/// 环境结构 - 可变的链式结构，支持变量绑定修改
#[derive(Trace, Finalize, Debug)]
pub struct Environment {
    /// 当前环境的变量绑定表 - 直接拥有，不需要 Gc 包装
    pub bindings: HashMap<String, RuntimeValue>,
    /// 上级环境（链式结构）
    pub parent: Option<Gc<Environment>>,
}

impl Environment {
    /// 创建新的空环境
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    /// 创建带父环境的新环境
    pub fn with_parent(parent: Gc<Environment>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }
    
    /// 在当前环境中定义变量（返回新环境）
    pub fn define(&self, name: String, value: RuntimeValue) -> Self {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(name, value);
        Self {
            bindings: new_bindings,
            parent: self.parent.clone(),
        }
    }
    
    /// 设置变量值（如果变量存在，返回新环境）
    pub fn set(&self, name: &str, value: RuntimeValue) -> Result<Self, String> {
        // 先在当前环境查找
        if self.bindings.contains_key(name) {
            let mut new_bindings = self.bindings.clone();
            new_bindings.insert(name.to_string(), value);
            return Ok(Self {
                bindings: new_bindings,
                parent: self.parent.clone(),
            });
        }
        
        // 递归在父环境查找
        if let Some(parent) = &self.parent {
            let new_parent = parent.set(name, value)?;
            Ok(Self {
                bindings: self.bindings.clone(),
                parent: Some(Gc::new(new_parent)),
            })
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }
    
    /// 查找变量值（递归向上查找）
    pub fn lookup(&self, name: &str) -> Option<RuntimeValue> {
        // 先在当前环境查找
        if let Some(value) = self.bindings.get(name) {
            return Some(value.clone());
        }
        
        // 递归在父环境查找
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}
```

#### 2.4 Lambda 函数结构
```rust
/// Lambda 函数 - 用户定义的函数（可变引用，用 Gc 包装）
#[derive(Trace, Finalize, Debug)]
pub struct Lambda {
    /// 参数名列表
    pub parameters: Vec<String>,
    /// 函数体（语法结构）
    pub body: Rc<SExpr>,
    /// 闭包环境（可变，用 Gc 包装）
    pub closure: Gc<Environment>,
}

// 为 Lambda 手动实现 PartialEq，比较引用是否相等
impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters &&
        Rc::ptr_eq(&self.body, &other.body) && 
        Gc::ptr_eq(&self.closure, &other.closure)
    }
}
```

#### 2.5 RuntimeValue 枚举
```rust
/// 运行时值 - 表示求值过程中的所有可能值类型
/// 按照可变性分为三类：
/// 1. 原子值：float, integer, char, nil - 可以直接复制
/// 2. 不可变引用：string, symbol, lambda, builtin-function - 用 Rc 引用
/// 3. 可变引用：cons, vector - 用 Gc 引用
#[derive(Trace, Finalize, Debug, Clone)]
pub enum RuntimeValue {
    // === 1. 原子值（Atomic Values）- 可直接复制 ===
    /// 数字（统一使用 f64）- 原子值，可直接复制
    Number(f64),
    /// 字符 - 原子值，可直接复制
    Character(char),
    /// 布尔值 - 原子值，可直接复制
    Boolean(bool),
    /// 空列表 - 原子值，可直接复制
    Nil,
    
    // === 2. 不可变引用（Immutable References）- 用 Rc 包装 ===
    /// 字符串 - 不可变引用，用 Rc 包装
    String(Rc<String>),
    /// 符号 - 不可变引用，用 Rc 包装
    Symbol(Rc<String>),
    /// 内置函数 - 不可变引用，用 Rc 包装
    BuiltinFunction(Rc<BuiltinFunction>),
    
    // === 3. 可变引用（Mutable References）- 用 Gc 包装 ===
    /// 用户定义的 Lambda 函数 - 可变引用，用 Gc 包装（因为 Environment 可变）
    Lambda(Gc<Lambda>),
    /// 可变列表（cons 结构）- 可变引用，用 Gc 包装
    Cons(Gc<MutableCons>),
    /// 可变向量 - 可变引用，用 Gc 包装
    Vector(Gc<MutableVector>),
}

// 手动实现 PartialEq for RuntimeValue
impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // 原子值直接比较
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a == b,
            (RuntimeValue::Character(a), RuntimeValue::Character(b)) => a == b,
            (RuntimeValue::Boolean(a), RuntimeValue::Boolean(b)) => a == b,
            (RuntimeValue::Nil, RuntimeValue::Nil) => true,
            
            // 不可变引用比较引用是否相等
            (RuntimeValue::String(a), RuntimeValue::String(b)) => Rc::ptr_eq(a, b),
            (RuntimeValue::Symbol(a), RuntimeValue::Symbol(b)) => Rc::ptr_eq(a, b),
            (RuntimeValue::BuiltinFunction(a), RuntimeValue::BuiltinFunction(b)) => Rc::ptr_eq(a, b),
            
            // 可变引用比较引用是否相等
            (RuntimeValue::Lambda(a), RuntimeValue::Lambda(b)) => Gc::ptr_eq(a, b),
            (RuntimeValue::Cons(a), RuntimeValue::Cons(b)) => Gc::ptr_eq(a, b),
            (RuntimeValue::Vector(a), RuntimeValue::Vector(b)) => Gc::ptr_eq(a, b),
            
            _ => false,
        }
    }
}

#### 2.6 辅助类型定义
```rust
/// 内置函数结构 - 不可变引用，用 Rc 包装
#[derive(Trace, Finalize, Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub arity: FunctionArity,
    pub implementation: BuiltinImpl,
}

/// Lambda 函数结构 - 可变引用，用 Gc 包装
#[derive(Trace, Finalize, Debug)]
pub struct Lambda {
    pub parameters: Vec<String>,
    pub body: Rc<SExpr>,
    pub closure: Gc<Environment>,
}

/// 函数参数个数要求
#[derive(Debug, Clone, PartialEq, Trace, Finalize)]
pub enum FunctionArity {
    /// 固定参数个数
    Exact(usize),
    /// 最少参数个数（支持可变参数）
    AtLeast(usize),
    /// 参数个数范围
    Range(usize, usize),
}

/// 内置函数实现类型
#[derive(Debug, Clone, Trace, Finalize)]
pub struct BuiltinImpl {
    /// 函数实现（接收参数列表，返回结果或错误）
    pub func: fn(&[RuntimeValue]) -> Result<RuntimeValue, EvaluateError>,
}

// 为 BuiltinImpl 手动实现 PartialEq，因为函数指针不能直接比较
impl PartialEq for BuiltinImpl {
    fn eq(&self, other: &Self) -> bool {
        // 比较函数指针地址
        std::ptr::eq(self.func as *const (), other.func as *const ())
    }
}
```

## 结论

这次重构的核心是将求值器从处理 `SExpr` 改为处理 `RuntimeValue`，通过明确的"加载"过程将编译时的语法结构转换为运行时的值。这种设计：

1. **架构清晰**：明确区分编译时和运行时
2. **三类分类**：根据可变性需求合理分配内存管理策略
   - 原子值：直接复制，性能最佳
   - 不可变引用：Rc 引用计数，支持共享
   - 可变引用：Gc 垃圾回收，支持可变操作
3. **支持可变性**：通过 GC 管理支持 R7RS 标准的可变操作
4. **保持一致性**：RuntimeValue 与 SExpr 一一对应，易于理解和维护
5. **扩展性强**：支持内置函数等运行时特有的概念
6. **性能优化**：根据数据类型特点选择合适的内存管理策略

重构过程将采用渐进式方法，确保系统的稳定性和向后兼容性，最终实现一个更符合 Scheme 标准的解释器。
