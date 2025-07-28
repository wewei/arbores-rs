# Arbores - Rust Scheme 解释器

一个用 Rust 实现的 Scheme 兼容 Lisp 解释器。

## 项目目标

实现一个功能完整的 Scheme 解释器，支持：
- 基本数据类型（数字、字符串、符号、列表等）
- 核心语法结构（define、lambda、if、cond 等）
- 内置函数和运算符
- 尾递归优化
- 词法作用域
- 宏系统（可选）

## 快速开始

```bash
# 运行解释器
cargo run

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

## 项目结构

```
src/
├── main.rs           # 主程序入口
├── lib.rs            # 库入口
├── lexer/            # 词法分析器
├── parser/           # 语法分析器
├── eval/             # 求值器
├── env/              # 环境管理
├── types/            # 数据类型定义
├── builtins/         # 内置函数
└── repl/             # REPL 实现
```

## 开发计划

参见 [PLAN.md](PLAN.md) 了解详细的开发计划。

## 许可证

MIT License
