# Project Arbores

## Arbores 是什么？

Arbores 是 Lisp 方言 Scheme 的一个 Rust 实现。
不同于一般的解释器，Arbores 是一个有状态的解释器，它维护着一个本地的（未来或许可以支持远程的）代码数据库，并提供 API 查询，引用，编辑库中的 Scheme 代码。
在人工智能快速发展的时代，Arbores 旨在给 AI Agent 提供一个存储、积累、查询结构化知识的仓库，Scheme 语言是这种知识的载体。从 AI 的视角看，Arbores 是一个特殊的 RAG，专门保存可以用 Scheme S-表达式描述的结构化知识。

> Arbores 并不是一门 Lisp 方言的名字，Arbores 支持的编程语言就是 Scheme (目标支持 R7RS)。

## 整体架构

Arbores 整体为三层架构

* Arbores CLI (Command Line Interface)，提供用户或 AI 可以调用的 CLI 接口以及 Repl 运行环境。
* Arbores Interpreter, Arbores 的 Scheme 解释器。
* Arbores Repository Manager，Arbores 的 Scheme 代码仓库管理器。

由于采用分层架构，CLI 并不会直接和 Repository Manager 通讯，而是通过 Interpreter 执行特定的 Scheme 代码实现对 Repository 的读写。

## 代码库结构

```text
arbores-rs/
├── Cargo.toml
├── docs/
│   │── Project_Arbores.md      # 技术文档
│   └── ...
├── src/
│   ├── cli/                    # CLI 模块
│   │   ├── mod.rs
│   │   └── ...
│   ├── interpreter/            # 解释器模块
│   │   ├── mod.rs
│   │   └── ...
│   └── repo_manager/           # 仓库管理器模块
│       ├── mod.rs
│       └── ...
├── tests/                      # 集成测试目录
│   └── ...
└── README.md                   # 项目对外介绍
```

## 代码规范

Project Arbores 采用函数式风格 Rust。详细代码规范参考 [Coding_Conventions.md](./Coding_Conventions.md)。

## 参考文献
