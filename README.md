# Arbores - AI 知识库系统

一个基于 Scheme 的结构化知识库系统，专为 AI 使用而设计。

## 项目愿景

**Arbores 是什么？**
- **对 AI Agent：** 一组 MCP (Model Context Protocol) 工具，可以存储、查询、执行 Lisp 代码
- **对 Scheme 开发者：** 带有可查询、可编辑、可按需加载的巨大 Preload 库的 Scheme 解释器  
- **对 AI 用户：** 让 AI 变得更聪明、更有行动力的知识增强系统

## 核心特性

### 🧠 S-Expression 为核心的知识存储
- 每个 S-Expression 都有全局唯一 ID 和丰富元数据
- 支持语义描述、类型标注、symbol names 管理
- 自动追踪 S-Expression 间的依赖关系

### 🔍 强大的搜索和查询
- **语义搜索：** 基于自然语言描述查找相关代码
- **符号匹配：** 支持前缀、通配符、正则表达式模式
- **依赖分析：** 正向/反向依赖查询，自动生成闭包代码

### 📚 不可变版本管理
- Copy-on-Write 的不可变存储，类似 Git 的版本控制
- 完整的版本历史和 reflog，支持版本切换和回滚
- 原子性事务操作，保证数据一致性

### 🔒 分层权限系统
- **T0 (系统级)：** 版本切换等敏感操作
- **T1 (读写级)：** 允许修改知识库内容
- **T2 (只读级)：** 仅查询和只读代码执行

## 快速开始

```bash
# 运行基础 Scheme 解释器
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
├── repl/             # REPL 实现
├── storage/          # 知识库存储 (待实现)
├── search/           # 搜索和索引 (待实现)
└── api/              # Arbores API 接口 (待实现)
```

## API 接口预览

### 版本管理
```scheme
(arb:current-version)          ; 获取当前版本
(arb:reflog)                   ; 查看版本历史
(arb:switch-version 12345)     ; 切换版本
```

### 知识存储和查询
```scheme
; 创建新的 S-Expression
(arb:create "(define (factorial n) ...)" 
            '() "阶乘函数" "function" '("factorial"))

; 语义搜索
(arb:semantic-search "排序算法")

; 符号搜索
(arb:search-by-symbol "arb:" "prefix")

; 获取元数据
(arb:get-metadata 123)
```

### 安全执行
```scheme
; 只读执行（不影响知识库）
(arb:eval-readonly '(factorial 5))

; 读写执行（可修改知识库）
(arb:eval '(arb:create ...))

; 事务性修改
(transaction
  (arb:create ...)
  (arb:update ...)
  (arb:delete ...))
```

## 开发状态

### ✅ 已完成 (MVP 第一阶段)
- **完整的 Scheme 解释器核心**：词法分析、语法分析、求值器
- **基础数据类型**：数字、字符串、符号、列表、布尔值
- **核心特殊形式**：quote、if、lambda、let、begin
- **内置函数库**：算术运算、比较运算、列表操作、类型谓词
- **交互式 REPL**：支持多行输入、历史记录、错误处理

### 🚧 进行中 (MVP 第二阶段)
- **S-Expression 存储系统**：设计中
- **基础 Arbores API**：规划中
- **元数据管理**：设计中

### 📋 计划中
- **版本管理系统** (MVP 第三阶段)
- **权限和执行系统** (MVP 第四阶段)  
- **索引和语义搜索** (MVP 第五阶段)

## 开发计划

参见 [PLAN.md](PLAN.md) 了解详细的 MVP 迭代计划。

## 许可证

MIT License
