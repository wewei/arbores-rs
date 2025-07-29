# REPL 改进调研报告

## 当前 REPL 状态

我们的 Scheme 解释器目前使用了一个基础的 REPL 实现，使用标准 `io::stdin().read_line()` 进行输入读取。虽然功能正常，但用户体验还有很大改进空间。

## 推荐的 Rust Crates

### 1. 首选推荐：reedline-repl-rs

**Crate**: `reedline-repl-rs` (v1.2.1)
- **下载量**: 81,236 总下载量，10,504 近期下载量
- **优势**:
  - 基于 `reedline` 和 `clap` 构建，提供现代化 REPL 体验
  - 支持语法高亮、自动补全、历史记录
  - 支持多行输入
  - 易于集成，文档完善
  - 积极维护的项目

**集成示例**:
```toml
[dependencies]
reedline-repl-rs = "1.2"
clap = { version = "4.0", features = ["derive"] }
```

### 2. 轻量级选择：simple-repl

**Crate**: `simple-repl` (v0.1.4)
- **下载量**: 5,230 总下载量，749 近期下载量
- **优势**:
  - 非常轻量级，学习成本低
  - 提供基础的 REPL 功能
  - 简单易用的 API

**集成示例**:
```toml
[dependencies]
simple-repl = "0.1"
```

### 3. 异步友好：mini_async_repl

**Crate**: `mini_async_repl` (v0.2.1)
- **下载量**: 1,358 总下载量，218 近期下载量
- **优势**:
  - 异步优先设计
  - 适合需要异步处理的场景
  - 现代化的架构

### 4. 底层控制：rustyline

**Crate**: `rustyline` (需要单独搜索版本信息)
- **优势**:
  - 类似 GNU Readline 的功能
  - 完全的终端控制
  - 支持历史记录、编辑、补全
  - 广泛使用，成熟稳定

### 5. 现代化替代：reedline (直接使用)

**Crate**: `reedline`
- **优势**:
  - 现代 Rust 终端输入库
  - 支持语法高亮、补全、历史
  - 灵活的插件系统
  - Nu shell 项目使用的核心组件

## 功能对比

| 功能 | 当前实现 | reedline-repl-rs | simple-repl | rustyline | reedline |
|------|---------|------------------|-------------|-----------|----------|
| 基础输入 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 历史记录 | ❌ | ✅ | ❌ | ✅ | ✅ |
| 语法高亮 | ❌ | ✅ | ❌ | ❌ | ✅ |
| 自动补全 | ❌ | ✅ | ❌ | ✅ | ✅ |
| 多行输入 | ❌ | ✅ | ❌ | ✅ | ✅ |
| 键盘快捷键 | 基础 | ✅ | 基础 | ✅ | ✅ |
| 易于集成 | - | ✅ | ✅ | 中等 | 中等 |

## 推荐的改进步骤

### 阶段 1：集成 reedline-repl-rs (推荐首选)

1. **添加依赖**:
```toml
[dependencies]
reedline-repl-rs = "1.2"
clap = { version = "4.0", features = ["derive"] }
ctrlc = "3.4"
```

2. **创建新的 REPL 模块**: `src/repl/advanced.rs`

3. **实现功能**:
   - 语法高亮（Scheme 关键字）
   - 括号匹配
   - 历史记录持久化
   - 自动补全（内置函数）
   - 多行输入支持

### 阶段 2：增强功能

1. **自定义语法高亮**:
   - Scheme 关键字高亮
   - 括号匹配提示
   - 字符串和数字高亮

2. **智能补全**:
   - 内置函数补全
   - 用户定义函数补全
   - 变量名补全

3. **历史功能**:
   - 命令历史持久化
   - 搜索历史
   - 历史统计

### 阶段 3：高级功能

1. **调试支持**:
   - 断点设置
   - 步进执行
   - 变量查看

2. **文档集成**:
   - 内置帮助系统
   - 函数文档查看
   - 示例代码

## 具体实现计划

### 1. 创建增强版 REPL

创建一个新的 REPL 实现，保留现有的简单版本作为后备：

```rust
// src/repl/enhanced.rs
use reedline_repl_rs::*;

pub struct EnhancedRepl {
    evaluator: crate::eval::Evaluator,
}

impl Repl for EnhancedRepl {
    // 实现 reedline-repl-rs 的 Repl trait
}
```

### 2. 添加命令行选项

使用 `clap` 添加 REPL 模式选择：

```bash
# 使用简单 REPL
arbores --repl=simple

# 使用增强 REPL (默认)
arbores --repl=enhanced
arbores
```

### 3. 渐进式迁移

- 保持现有 REPL 作为后备选项
- 新的增强 REPL 作为默认选项
- 用户可以选择使用哪种 REPL

## 预期收益

1. **用户体验**:
   - 历史记录避免重复输入
   - 语法高亮提高可读性
   - 自动补全提高效率

2. **开发效率**:
   - 更好的错误提示
   - 调试功能支持
   - 文档集成

3. **专业性**:
   - 现代化的终端体验
   - 与其他现代解释器一致的用户界面

## 风险评估

1. **依赖增加**: 新增外部依赖可能影响编译时间和二进制大小
2. **复杂性**: 代码复杂度会有所增加
3. **兼容性**: 需要确保在不同平台上的兼容性

## 建议

1. **立即行动**: 集成 `reedline-repl-rs` 作为主要改进
2. **保持兼容**: 保留简单 REPL 作为后备选项
3. **渐进实现**: 分阶段实现各种功能
4. **用户选择**: 让用户可以选择使用哪种 REPL 模式

这个改进将显著提升 arbores-rs 的用户体验，使其更接近现代化的编程语言解释器标准。
