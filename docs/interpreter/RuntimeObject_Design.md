# RuntimeObject 设计文档

## 概述

为了更准确地表达运行时的概念，我们引入 `RuntimeObject` 来替代 `RuntimeValue`。`RuntimeObject` 代表了一个可变的运行时对象，避免了 "Value" 给人最终运算结果的印象。

## 设计动机

### 问题分析
1. **命名误导**：`RuntimeValue` 中的 "Value" 暗示这是一个最终的计算结果，但实际上它代表的是运行时的对象
2. **可变性表达不清**：当前设计没有明确表达对象的可变性特征
3. **引用类型混乱**：不同类型的引用（Rc、Weak、Gc）混在一起，缺乏清晰的分类

### 解决方案
引入 `RuntimeObject` 概念，明确区分四种不同类型的运行时对象：
1. **原子值**：直接存储，不可变
2. **Rc 引用值**：强引用，不可变内容但可共享
3. **Weak 引用值**：弱引用，用于避免循环引用
4. **GC 引用值**：垃圾回收引用，支持可变操作

## RuntimeObject 四种分类

### 1. 原子值（Atomic Objects）
包括：`integer`, `float`, `boolean`, `nil`

**特点**：
- 直接存储值，不需要引用计数
- 不可变，但可以复制
- 性能最佳，无内存管理开销

**实现**：
```rust
// 原子值直接存储在枚举中
Number(f64),
Character(char),
Boolean(bool),
Nil,
```

### 2. Rc 引用值（Rc Reference Objects）
包括：`Rc<String>`（字符串、符号）

**特点**：
- 使用 `Rc` 强引用计数
- 内容不可变，但可以共享
- 共享的对象不会造成循环引用
- 避免不必要的字符串克隆

**实现**：
```rust
// 不可变字符串和符号使用 Rc 包装
String(Rc<String>),
Symbol(Rc<String>),
```

### 3. Weak 引用值（Weak Reference Objects）
包括：`Weak<BuiltinFunction>`

**特点**：
- 使用 `Weak` 弱引用，避免循环引用
- 不会阻止对象被垃圾回收
- 引用对象具有全局生命周期
- 适用于内置函数等不需要强引用的场景

**实现**：
```rust
// 内置函数使用 Weak 引用，避免循环引用
BuiltinFunction(Weak<BuiltinFunction>),
```

### 4. GC 引用值（GC Reference Objects）
包括：`Gc<Cons>`, `Gc<Vector>`, `Gc<Lambda>`, `Gc<Continuation>`

**特点**：
- 使用 `Gc` 垃圾回收
- 对象可能会造成循环引用
- 支持可变操作（set-car!, set-cdr!, vector-set!）
- 自动内存管理

**实现**：
```rust
// 可变数据结构使用 Gc 包装
Cons(Gc<MutableCons>),
Vector(Gc<MutableVector>),
Lambda(Gc<Lambda>),
Continuation(Gc<Continuation>),
```

## 核心数据结构设计

### RuntimeObjectCore 枚举
```rust
/// 运行时对象核心 - 表示运行时的所有可能对象类型
/// 按照引用类型分为四类：
/// 1. 原子值：integer, float, boolean, nil - 直接存储
/// 2. Rc 引用值：Rc<String> - 强引用，不可变内容
/// 3. Weak 引用值：Weak<BuiltinFunction> - 弱引用，避免循环引用
/// 4. GC 引用值：Gc<Cons>, Gc<Vector>, Gc<Lambda>, Gc<Continuation> - 垃圾回收，支持可变操作
#[derive(Trace, Finalize, Debug, Clone)]
pub enum RuntimeObjectCore {
    // === 1. 原子值（Atomic Objects）- 直接存储 ===
    /// 整数 - 原子值，直接存储
    Integer(i64),
    /// 浮点数 - 原子值，直接存储
    Float(f64),
    /// 有理数 - 原子值，直接存储
    Rational(i64, i64),  // 分子, 分母
    /// 字符 - 原子值，直接存储
    Character(char),
    /// 布尔值 - 原子值，直接存储
    Boolean(bool),
    /// 空列表 - 原子值，直接存储
    Nil,
    
    // === 2. Rc 引用值（Rc Reference Objects）- 强引用 ===
    /// 字符串 - Rc 引用值，不可变内容但可共享
    String(Rc<String>),
    /// 符号 - Rc 引用值，不可变内容但可共享
    Symbol(Rc<String>),
    
    // === 3. Weak 引用值（Weak Reference Objects）- 弱引用 ===
    /// 内置函数 - Weak 引用值，避免循环引用
    BuiltinFunction(Weak<BuiltinFunction>),
    
    // === 4. GC 引用值（GC Reference Objects）- 垃圾回收 ===
    /// 可变列表（cons 结构）- GC 引用值，支持可变操作
    Cons(Gc<MutableCons>),
    /// 可变向量 - GC 引用值，支持可变操作
    Vector(Gc<MutableVector>),
    /// Lambda 函数 - GC 引用值，支持环境可变
    Lambda(Gc<Lambda>),
    /// 续延 - GC 引用值，支持 call/cc
    Continuation(Gc<Continuation>),
}
```

### 可变数据结构
```rust
/// 可变 Cons 结构 - 支持 set-car! 和 set-cdr! 操作
#[derive(Trace, Finalize, Debug)]
pub struct MutableCons {
    /// car 部分 - 使用 GcCell 支持可变性
    pub car: GcCell<RuntimeObject>,
    /// cdr 部分 - 使用 GcCell 支持可变性
    pub cdr: GcCell<RuntimeObject>,
}

/// 可变向量 - 支持 vector-set! 操作
#[derive(Trace, Finalize, Debug)]
pub struct MutableVector {
    /// 向量元素 - 使用 GcCell 支持可变性
    pub elements: GcCell<Vec<RuntimeObject>>,
}
```

### 可变数据结构设计说明
```rust
// 为什么使用 GcCell<RuntimeObject> 而不是 GcCell<Gc<RuntimeObject>>？

// 1. RuntimeObject 本身比较小（约 24-32 字节）
pub struct RuntimeObject {
    pub core: RuntimeObjectCore,        // 枚举，最大变体约 16-24 字节
    pub source: Option<Rc<SExpr>>,      // 8 字节
}

// 2. 大对象已经通过引用存储
RuntimeObject::Cons(Gc<MutableCons>)     // 实际数据在 Gc<MutableCons> 中
RuntimeObject::Vector(Gc<MutableVector>) // 实际数据在 Gc<MutableVector> 中
RuntimeObject::Lambda(Gc<Lambda>)        // 实际数据在 Gc<Lambda> 中

// 3. 简化引用层次
// 旧设计：GcCell<Gc<RuntimeObject>> - 两层引用
// 新设计：GcCell<RuntimeObject> - 一层引用，更简洁

/// Lambda 函数 - 用户定义的函数
#[derive(Trace, Finalize, Debug)]
pub struct Lambda {
    /// 参数名列表
    pub parameters: Vec<String>,
    /// 函数体（语法结构）
    pub body: Rc<SExpr>,
    /// 闭包环境（可变，用 Gc 包装）
    pub closure: Gc<Environment>,
}

/// 内置函数结构
#[derive(Trace, Finalize, Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub arity: FunctionArity,
    pub implementation: BuiltinImpl,
}

/// 续延结构 - 支持 call/cc
#[derive(Trace, Finalize, Debug)]
pub struct Continuation {
    /// 续延函数 - 捕获了必要的上下文（环境、调用栈等）
    pub func: Box<dyn Fn(Gc<RuntimeObject>) -> EvaluateResult>,
}

impl Continuation {
    /// 创建新的续延
    pub fn new<F>(func: F) -> Self 
    where 
        F: Fn(Gc<RuntimeObject>) -> EvaluateResult + 'static
    {
        Self { func: Box::new(func) }
    }
    
    /// 调用续延函数
    pub fn call(&self, value: Gc<RuntimeObject>) -> EvaluateResult {
        (self.func)(value)
    }
}```

### 环境结构
```rust
/// 环境结构 - 可变的链式结构，支持变量绑定修改
#[derive(Trace, Finalize, Debug)]
pub struct Environment {
    /// 当前环境的变量绑定表
    pub bindings: HashMap<String, RuntimeObject>,
    /// 上级环境（链式结构）
    pub parent: Option<Gc<Environment>>,
}
```

### 调用栈帧结构
```rust
/// 调用栈帧 - 支持函数调用和续延
#[derive(Trace, Finalize, Debug)]
pub struct Frame {
    /// 当前环境
    pub env: Gc<Environment>,
    /// 续延
    pub continuation: Gc<Continuation>,  // GC 包装的续延函数
    /// 父栈帧
    pub parent: Option<Gc<Frame>>,
}
```

### RuntimeObject 包装结构
```rust
/// 运行时对象 - 包含核心对象和可选的源表达式
/// RuntimeObject 本身是一个比较小的对象，可以直接 Clone
#[derive(Trace, Finalize, Debug, Clone)]
pub struct RuntimeObject {
    /// 核心运行时对象
    pub core: RuntimeObjectCore,
    /// 可选的源表达式，用于保存计算出该 RuntimeObject 的 SExpr
    pub source: Option<Rc<SExpr>>,
}

impl RuntimeObject {
    /// 创建新的运行时对象
    pub fn new(core: RuntimeObjectCore) -> Self {
        Self {
            core,
            source: None,
        }
    }
    
    /// 创建带源表达式的运行时对象
    pub fn with_source(core: RuntimeObjectCore, source: Rc<SExpr>) -> Self {
        Self {
            core,
            source: Some(source),
        }
    }
    
    /// 获取核心对象
    pub fn core(&self) -> &RuntimeObjectCore {
        &self.core
    }
    
    /// 获取源表达式
    pub fn source(&self) -> Option<&Rc<SExpr>> {
        self.source.as_ref()
    }
}
```

## 内存管理策略

### 1. 原子值
- **策略**：直接存储，无内存管理
- **优势**：性能最佳，无开销
- **适用场景**：整数、浮点数、有理数、字符、布尔值、空值

### 2. Rc 引用值
- **策略**：引用计数，自动释放
- **优势**：支持共享，避免克隆，不会造成循环引用
- **适用场景**：字符串、符号等不可变但需要共享的对象

### 3. Weak 引用值
- **策略**：弱引用，不阻止垃圾回收
- **优势**：避免循环引用，引用对象具有全局生命周期
- **适用场景**：内置函数等不需要强引用的对象

### 4. GC 引用值
- **策略**：垃圾回收，支持可变操作
- **优势**：自动内存管理，支持可变性，处理循环引用
- **适用场景**：列表、向量、Lambda 函数、续延等可变对象

## 引用关系设计

### Frame, Environment, Continuation 之间的引用
```rust
// 所有可变结构都使用 Gc 引用
Frame {
    env: Gc<Environment>,           // 环境可变
    continuation: Gc<Continuation>, // 续延（GC 包装的函数）
    parent: Option<Gc<Frame>>,      // 父栈帧可变
}

Environment {
    bindings: HashMap<String, RuntimeObject>, // 绑定可变
    parent: Option<Gc<Environment>>,          // 父环境可变
}

Continuation {
    func: Box<dyn Fn(Gc<RuntimeObject>) -> EvaluateResult>, // 续延函数
}
```

### Continuation 设计说明
1. **Continuation 不需要 env**：环境信息已经包含在 Frame 中，续延函数作为闭包会捕获必要的上下文
2. **Continuation 不需要 parent**：调用栈关系通过 Frame 的 parent 字段维护，Continuation 是函数而不是栈帧
3. **简化设计**：Continuation 定义为 `Box<dyn Fn>`，在使用处用 `Gc` 包装以保持一致性
4. **内存安全**：使用 GC 管理续延函数，支持复杂的引用关系
5. **设计统一**：所有可变对象都使用 `Gc` 包装，保持设计一致性

### 引用选择原则
1. **Gc 引用**：用于可能造成循环引用的可变对象
2. **Rc 引用**：用于不会造成循环引用的不可变对象
3. **Weak 引用**：用于具有全局生命周期的对象
4. **直接存储**：用于原子值

## 优势分析

### 1. 概念清晰
- **RuntimeObject**：明确表达运行时的对象概念
- **四种分类**：根据引用类型清晰分类
- **内存管理**：每种类型都有明确的内存管理策略

### 2. 类型安全
- **编译时检查**：强制区分不同类型的对象
- **运行时安全**：避免错误的可变操作
- **内存安全**：正确的引用管理

### 3. 性能优化
- **原子值**：无内存管理开销
- **Rc 引用**：避免不必要的克隆
- **Weak 引用**：避免循环引用
- **GC 引用**：只在需要可变性的地方使用

### 4. 扩展性强
- **新类型**：容易添加新的对象类型
- **新操作**：支持新的可变操作
- **新策略**：支持新的内存管理策略

## 总结

RuntimeObject 设计通过明确的四种分类和合理的引用关系设计，提供了更清晰、更安全、更高效的运行时对象管理方案。这种设计不仅符合 Scheme 的语义要求，也为未来的扩展（如 call/cc）提供了良好的基础。

重构过程将采用渐进式方法，确保系统的稳定性和向后兼容性，最终实现一个更符合现代编程语言设计理念的解释器。
