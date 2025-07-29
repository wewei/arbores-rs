# MVP 第二阶段演示

本文档展示了 Arbores 知识库系统第二阶段的核心功能。

## 快速开始

运行演示程序：

```bash
cargo run --example stage2_demo
```

运行测试：

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test storage
cargo test arbores
```

## 已实现的功能

### 1. S-Expression 存储系统

- **全局唯一 ID 分配**：每个 S-Expression 自动分配唯一的 64 位整数 ID
- **丰富的元数据**：支持语义描述、类型标注、symbol names、依赖关系
- **时间戳管理**：自动记录创建和修改时间

### 2. 核心 API 接口

#### `arb:create` - 创建 S-Expression
```rust
let id = arbores.create(
    "(define (factorial n) ...)",    // Scheme 代码
    vec![],                          // 依赖的 S-Expression ID 列表
    Some("阶乘函数".to_string()),    // 语义描述
    Some("function".to_string()),    // 类型描述
    vec!["factorial".to_string()]    // 建议的 symbol names
)?;
```

#### `arb:get-metadata` - 查询元数据
```rust
let metadata = arbores.get_metadata(id)?;
// 返回 association list 格式的元数据
```

#### `arb:get-dependencies` - 查询依赖关系
```rust
let deps = arbores.get_dependencies(id)?;
// 返回依赖的 S-Expression ID 列表
```

### 3. 搜索和查询系统

#### `arb:search-by-symbol` - 按符号搜索
```rust
// 前缀匹配
let results = arbores.search_by_symbol("fact", Some("prefix"))?;

// 精确匹配
let results = arbores.search_by_symbol("factorial", Some("exact"))?;
```

#### `arb:semantic-search` - 语义搜索
```rust
let results = arbores.semantic_search("递归")?;
// 基于描述内容的语义匹配
```

### 4. 依赖关系管理

- **自动索引**：创建 S-Expression 时自动建立依赖关系索引
- **双向查询**：支持查询某个 S-Expression 的依赖和被依赖关系
- **一致性维护**：更新和删除操作自动维护索引一致性

## 数据格式

### S-Expression 元数据格式
```scheme
(("id" . 1)
 ("description" . "计算阶乘的递归函数")
 ("type" . "function")
 ("symbol-names" "factorial" "fact")
 ("dependencies" 2 3)
 ("code" define (factorial n) ...))
```

### 搜索结果格式
```scheme
;; 符号搜索结果
((("id" . 1)
  ("symbol-names" "factorial" "fact")
  ("description" . "计算阶乘的递归函数")))

;; 语义搜索结果
((("id" . 1)
  ("score" . 0.8)
  ("description" . "计算阶乘的递归函数")))
```

## 技术特性

### 存储架构
- **内存存储**：基于 HashMap 的高效内存存储
- **索引系统**：symbol name 索引、依赖关系索引
- **类型安全**：Rust 类型系统保证内存安全

### 接口设计
- **统一格式**：所有输出使用 S-Expression 格式
- **错误处理**：完整的错误类型和处理机制
- **测试覆盖**：每个功能都有对应的单元测试

## 演示输出示例

```
=== Arbores MVP 第二阶段演示 ===

1. 创建 S-Expression
   创建阶乘函数，ID: 1
   创建斐波那契函数，ID: 2
   创建测试函数，ID: 3

2. 查询元数据
   阶乘函数的元数据:
   (("id" . 1) ("description" . "计算阶乘的递归函数") ...)

3. 查询依赖关系
   测试函数的依赖: (1 2)

4. 按符号搜索
   前缀搜索 'fact':
   ((("id" . 1) ("symbol-names" "factorial" "fact") ...))

5. 语义搜索
   语义搜索 '递归':
   ((("id" . 1) ("score" . 0.8) ("description" . "计算阶乘的递归函数")) ...)

=== 演示完成 ===
```

## 下一步计划

第二阶段完成后，接下来将进入第三阶段：版本管理基础。主要包括：

- Copy-on-Write 存储机制
- 版本快照和切换
- 版本历史追踪
- Delta 操作记录

## 测试覆盖

```bash
$ cargo test
running 27 tests
...
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

所有测试通过，确保功能稳定可靠。
