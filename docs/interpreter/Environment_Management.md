# 环境管理设计

状态：Draft-1

## 概述

环境管理模块负责管理变量绑定和作用域，支持 Scheme 的词法作用域特性。采用函数式设计，数据与行为分离。

## 模块职责（功能性需求）

- **变量绑定管理**：创建、查找、更新变量绑定
- **作用域链管理**：维护词法作用域的层级关系
- **生命周期管理**：环境的创建、销毁和垃圾回收
- **类型支持**：同时支持 Value 和 MValue 类型的环境

## 设计目标（非功能性需求）

- **作用域正确性**：准确实现 Scheme 的词法作用域语义
- **查找性能**：变量查找操作的高效实现
- **内存安全**：避免循环引用，确保内存正确释放
- **函数式设计**：不可变操作，纯函数实现

## 关键数据类型

### EnvironmentId

环境的唯一标识符：

```rust
/// 环境 ID - 使用 newtype 模式确保类型安全
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnvironmentId(usize);
```

### Environment

环境引用的纯数据结构：

```rust
/// 环境引用 - 纯数据结构
#[derive(Debug, Clone)]
pub struct Environment<V> {
    pub id: EnvironmentId,
    pub manager: Rc<RefCell<EnvironmentManager<V>>>,
}
```

### EnvironmentData

环境数据的纯数据结构：

```rust
/// 环境数据 - 纯数据结构
#[derive(Debug, Clone)]
pub struct EnvironmentData<V> {
    pub bindings: HashMap<String, V>,
    pub parent_id: Option<EnvironmentId>,
    pub env_type: EnvironmentType,
}
```

### EnvironmentManager

环境管理器的状态数据：

```rust
/// 环境管理器 - 纯数据结构
pub struct EnvironmentManager<V> {
    pub environments: HashMap<EnvironmentId, EnvironmentData<V>>,
    pub next_id: EnvironmentId,
    pub global_env_id: Option<EnvironmentId>,
}
```

### EnvironmentType

环境类型的代数数据类型：

```rust
/// 环境类型 - 使用 enum 表示不同环境类型
#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentType {
    /// 全局环境
    Global,
    /// 函数环境
    Function { function_name: Option<String> },
    /// 块环境
    Block,
    /// 闭包环境
    Closure { captured_vars: Vec<String> },
}
```

### EnvironmentError

环境操作错误的代数数据类型：

```rust
/// 环境错误 - 使用 enum 表示不同错误情况
#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentError {
    /// 未定义变量
    UndefinedVariable { name: String, env_id: EnvironmentId },
    /// 环境不存在
    EnvironmentNotFound(EnvironmentId),
    /// 重复定义
    AlreadyDefined { name: String, env_id: EnvironmentId },
    /// 父环境不存在
    ParentNotFound(EnvironmentId),
}
```

## 关键功能函数接口

### 环境管理器操作

#### create_environment_manager

| 参数名 | 类型 | 描述 |
|--------|------|------|
| - | - | 无参数 |

| 类型 | 描述 |
|------|------|
| `EnvironmentManager<V>` | 新创建的环境管理器 |

#### create_global_environment

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&mut EnvironmentManager<V>` | 环境管理器的可变引用 |

| 类型 | 描述 |
|------|------|
| `EnvironmentId` | 全局环境的 ID |

#### create_child_environment

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&mut EnvironmentManager<V>` | 环境管理器的可变引用 |
| parent_id | `EnvironmentId` | 父环境的 ID |
| env_type | `EnvironmentType` | 环境类型 |

| 类型 | 描述 |
|------|------|
| `Result<EnvironmentId, EnvironmentError>` | 新环境的 ID 或错误 |

### 变量绑定操作

#### define_variable

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&mut EnvironmentManager<V>` | 环境管理器的可变引用 |
| env_id | `EnvironmentId` | 目标环境的 ID |
| name | `String` | 变量名 |
| value | `V` | 变量值 |

| 类型 | 描述 |
|------|------|
| `Result<(), EnvironmentError>` | 成功或错误 |

#### lookup_variable

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&EnvironmentManager<V>` | 环境管理器的引用 |
| env_id | `EnvironmentId` | 起始环境的 ID |
| name | `&str` | 变量名 |

| 类型 | 描述 |
|------|------|
| `Result<V, EnvironmentError>` | 变量值或错误 |

#### set_variable

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&mut EnvironmentManager<V>` | 环境管理器的可变引用 |
| env_id | `EnvironmentId` | 起始环境的 ID |
| name | `&str` | 变量名 |
| value | `V` | 新值 |

| 类型 | 描述 |
|------|------|
| `Result<(), EnvironmentError>` | 成功或错误 |

### 作用域链操作

#### find_variable_environment

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&EnvironmentManager<V>` | 环境管理器的引用 |
| env_id | `EnvironmentId` | 起始环境的 ID |
| name | `&str` | 变量名 |

| 类型 | 描述 |
|------|------|
| `Option<EnvironmentId>` | 包含该变量的环境 ID |

#### get_parent_environment

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&EnvironmentManager<V>` | 环境管理器的引用 |
| env_id | `EnvironmentId` | 子环境的 ID |

| 类型 | 描述 |
|------|------|
| `Option<EnvironmentId>` | 父环境的 ID |

#### collect_scope_chain

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&EnvironmentManager<V>` | 环境管理器的引用 |
| env_id | `EnvironmentId` | 起始环境的 ID |

| 类型 | 描述 |
|------|------|
| `Vec<EnvironmentId>` | 作用域链上的所有环境 ID |

### 环境查询操作

#### environment_exists

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&EnvironmentManager<V>` | 环境管理器的引用 |
| env_id | `EnvironmentId` | 环境 ID |

| 类型 | 描述 |
|------|------|
| `bool` | 环境是否存在 |

#### get_environment_type

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&EnvironmentManager<V>` | 环境管理器的引用 |
| env_id | `EnvironmentId` | 环境 ID |

| 类型 | 描述 |
|------|------|
| `Option<&EnvironmentType>` | 环境类型的引用 |

#### list_variables

| 参数名 | 类型 | 描述 |
|--------|------|------|
| manager | `&EnvironmentManager<V>` | 环境管理器的引用 |
| env_id | `EnvironmentId` | 环境 ID |

| 类型 | 描述 |
|------|------|
| `Result<Vec<String>, EnvironmentError>` | 变量名列表或错误 |

## 关键设计问题

### 问题：环境 ID 系统与引用计数的内存管理策略

TODO

### 问题：作用域链查找的性能优化和缓存机制

TODO

### 问题：闭包环境的变量捕获和生命周期管理

TODO

### 问题：垃圾回收时环境的清理策略和循环引用检测

TODO

### 问题：并发访问环境时的线程安全和锁策略

TODO
