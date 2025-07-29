# Environment 系统重构设计方案

## 背景

当前的 Environment 系统使用 `Rc<Environment>` 导致不可变性问题，无法实现 `define` 特殊形式。需要重构为支持可变绑定但保持 S-Expression 不可变的设计。

## 设计目标

1. **保持 S-Expression 完全不可变**：所有的 `Value` 结构保持 immutable
2. **支持变量绑定的可变性**：能够实现 `define` 和 `set!` 操作
3. **避免循环引用**：防止内存泄漏和 RefCell 借用冲突
4. **保持性能**：环境查找和更新操作高效
5. **线程安全考虑**：为未来的并发支持留下空间

## 核心方案：Environment ID 系统

### 设计原理

将环境数据和环境引用分离：
- **环境数据**：存储在中央的 `EnvironmentManager` 中
- **环境引用**：只包含 ID 和对 manager 的引用
- **S-Expression**：完全不包含可变引用

### 核心数据结构

```rust
/// 环境 ID 类型
pub type EnvironmentId = usize;

/// 中央环境管理器
pub struct EnvironmentManager {
    /// 所有环境的绑定数据
    environments: HashMap<EnvironmentId, EnvironmentData>,
    /// 下一个可用的环境 ID
    next_id: EnvironmentId,
}

/// 单个环境的数据
#[derive(Debug, Clone)]
pub struct EnvironmentData {
    /// 当前环境的变量绑定
    bindings: HashMap<String, Value>,
    /// 父环境的 ID
    parent_id: Option<EnvironmentId>,
}

/// 环境引用（轻量级，不包含实际数据）
#[derive(Debug, Clone)]
pub struct Environment {
    /// 环境的唯一标识
    id: EnvironmentId,
    /// 对环境管理器的引用（唯一的可变部分）
    manager: Rc<RefCell<EnvironmentManager>>,
}
```

### Value 类型修改

```rust
/// Lambda 函数的新定义
pub enum Value {
    // ... 其他变体保持不变
    
    /// 用户定义的函数 (lambda)
    Lambda {
        params: Vec<String>,
        body: Rc<Value>,
        env_id: EnvironmentId,  // 改为存储环境 ID
    },
}
```

## 详细实现方案

### 1. EnvironmentManager 实现

```rust
impl EnvironmentManager {
    /// 创建新的环境管理器
    pub fn new() -> Self {
        EnvironmentManager {
            environments: HashMap::new(),
            next_id: 0,
        }
    }

    /// 创建新的根环境
    pub fn create_root_env(&mut self) -> EnvironmentId {
        let id = self.next_id;
        self.next_id += 1;
        
        self.environments.insert(id, EnvironmentData {
            bindings: HashMap::new(),
            parent_id: None,
        });
        
        id
    }

    /// 创建子环境
    pub fn create_child_env(&mut self, parent_id: EnvironmentId) -> EnvironmentId {
        let id = self.next_id;
        self.next_id += 1;
        
        self.environments.insert(id, EnvironmentData {
            bindings: HashMap::new(),
            parent_id: Some(parent_id),
        });
        
        id
    }

    /// 在环境中定义变量
    pub fn define(&mut self, env_id: EnvironmentId, name: String, value: Value) -> Result<()> {
        if let Some(env_data) = self.environments.get_mut(&env_id) {
            env_data.bindings.insert(name, value);
            Ok(())
        } else {
            Err(SchemeError::RuntimeError(format!("Environment {} not found", env_id)))
        }
    }

    /// 查找变量（递归查找父环境）
    pub fn lookup(&self, env_id: EnvironmentId, name: &str) -> Result<Value> {
        if let Some(env_data) = self.environments.get(&env_id) {
            if let Some(value) = env_data.bindings.get(name) {
                Ok(value.clone())
            } else if let Some(parent_id) = env_data.parent_id {
                self.lookup(parent_id, name)
            } else {
                Err(SchemeError::UndefinedVariable(name.to_string()))
            }
        } else {
            Err(SchemeError::RuntimeError(format!("Environment {} not found", env_id)))
        }
    }

    /// 设置变量值（必须是已存在的变量）
    pub fn set(&mut self, env_id: EnvironmentId, name: &str, value: Value) -> Result<()> {
        if let Some(env_data) = self.environments.get_mut(&env_id) {
            if env_data.bindings.contains_key(name) {
                env_data.bindings.insert(name.to_string(), value);
                Ok(())
            } else if let Some(parent_id) = env_data.parent_id {
                self.set(parent_id, name, value)
            } else {
                Err(SchemeError::UndefinedVariable(name.to_string()))
            }
        } else {
            Err(SchemeError::RuntimeError(format!("Environment {} not found", env_id)))
        }
    }
}
```

### 2. Environment 包装器实现

```rust
impl Environment {
    /// 创建新的根环境
    pub fn new(manager: Rc<RefCell<EnvironmentManager>>) -> Self {
        let id = manager.borrow_mut().create_root_env();
        Environment { id, manager }
    }

    /// 创建子环境
    pub fn new_child(&self) -> Self {
        let id = self.manager.borrow_mut().create_child_env(self.id);
        Environment {
            id,
            manager: Rc::clone(&self.manager),
        }
    }

    /// 定义变量
    pub fn define(&self, name: String, value: Value) -> Result<()> {
        self.manager.borrow_mut().define(self.id, name, value)
    }

    /// 查找变量
    pub fn lookup(&self, name: &str) -> Result<Value> {
        self.manager.borrow().lookup(self.id, name)
    }

    /// 设置变量
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        self.manager.borrow_mut().set(self.id, name, value)
    }

    /// 获取环境 ID
    pub fn id(&self) -> EnvironmentId {
        self.id
    }
}
```

### 3. 求值器修改

```rust
// 在 eval 模块中
pub struct Evaluator {
    env_manager: Rc<RefCell<EnvironmentManager>>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            env_manager: Rc::new(RefCell::new(EnvironmentManager::new())),
        }
    }

    pub fn create_global_env(&self) -> Environment {
        Environment::new(Rc::clone(&self.env_manager))
    }

    // 修改 eval_lambda 函数
    fn eval_lambda(&self, params: &[Value], body: &Value, env: &Environment) -> Result<Value> {
        // ... 参数处理逻辑
        
        Ok(Value::Lambda {
            params: param_names,
            body: Rc::new(body.clone()),
            env_id: env.id(),  // 存储环境 ID 而不是环境本身
        })
    }

    // 修改函数调用逻辑
    fn call_lambda(&self, params: &[String], body: &Value, env_id: EnvironmentId, args: &[Value]) -> Result<Value> {
        // 创建新的执行环境
        let call_env = {
            let id = self.env_manager.borrow_mut().create_child_env(env_id);
            Environment {
                id,
                manager: Rc::clone(&self.env_manager),
            }
        };

        // 绑定参数
        for (param, arg) in params.iter().zip(args) {
            call_env.define(param.clone(), arg.clone())?;
        }

        // 在新环境中求值函数体
        self.eval(body, &call_env)
    }
}
```

## 优势分析

### 1. 内存安全
- **无循环引用**：Lambda 只存储 `EnvironmentId`（usize），不持有 Environment
- **清晰的所有权**：所有环境数据归 `EnvironmentManager` 所有
- **可预测的清理**：当 `EnvironmentManager` 销毁时，所有环境数据都被清理

### 2. 性能优势
- **轻量级环境创建**：创建环境只需分配 ID 和更新 HashMap
- **共享环境管理器**：多个环境共享同一个管理器，减少内存开销
- **高效查找**：HashMap 查找复杂度 O(1)

### 3. 函数式特性保持
- **S-Expression 完全不可变**：`Value` 枚举不包含任何 `RefCell`
- **安全的数据共享**：可以安全地 clone 和缓存 S-Expression
- **纯函数语义**：相同的 S-Expression 总是产生相同的结果

### 4. 扩展性
- **版本控制支持**：可以轻松实现环境快照和回滚
- **并发友好**：可以为并发访问添加读写锁
- **调试支持**：可以轻松实现环境检查和调试工具

## 实现计划

### 阶段 1：基础重构
1. 创建新的 `EnvironmentManager` 和相关数据结构
2. 修改 `Value::Lambda` 的定义
3. 更新 `Environment` 包装器

### 阶段 2：求值器更新
1. 修改 `Evaluator` 结构体
2. 更新 lambda 创建和调用逻辑
3. 实现 `define` 特殊形式

### 阶段 3：测试和验证
1. 运行现有测试确保兼容性
2. 添加 `define` 相关测试
3. 验证内存使用情况

### 阶段 4：优化和完善
1. 性能测试和优化
2. 添加环境调试工具
3. 文档更新

## 风险评估

### 低风险
- **借用检查冲突**：通过明确的借用范围和生命周期管理可以避免
- **性能回归**：HashMap 查找非常高效，预期性能影响最小

### 中等风险
- **代码复杂度**：需要仔细管理 RefCell 的借用，但通过清晰的接口可以控制
- **调试难度**：环境数据分离可能让调试变复杂，但可以通过工具解决

### 缓解策略
1. **渐进式重构**：分阶段实现，每阶段都保证测试通过
2. **完善测试**：特别关注 RefCell 借用冲突的边界情况
3. **工具支持**：开发环境检查和调试工具

## 总结

Environment ID 系统通过将环境数据和引用分离，既解决了当前的不可变性问题，又保持了 S-Expression 的函数式特性。这个设计为 Arbores 系统的 `define` 功能实现提供了坚实的基础，同时为未来的功能扩展留下了空间。
