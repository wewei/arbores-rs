# Arbores 知识库系统开发计划

## MVP 第一阶段：基础 Scheme 解释器

### 目标：完成可运行的 Scheme 解释器核心
- [x] 词法分析器和语法分析器
- [x] 基础数据类型和求值器
- [x] 核心特殊形式（quote, if, lambda, let, begin）
- [x] 基础内置函数（算术、比较、列表操作）
- [x] REPL 交互界面

### 当前状态
✅ **已完成** - 基础 Scheme 解释器可以运行简单程序

## MVP 第二阶段：S-Expression 存储和查询

### 目标：实现核心知识库功能
- [x] S-Expression 数据结构设计
  - [x] 全局唯一 ID 分配
  - [x] 元数据存储（描述、类型、symbol names）
  - [x] 依赖关系管理
- [x] 基础存储接口
  - [x] `arb:create` - 创建 S-Expression
  - [x] `arb:get-metadata` - 获取元数据
  - [x] `arb:get-dependencies` - 查询依赖关系
- [x] 简单查询接口
  - [x] `arb:search-by-symbol` - 按 symbol name 查询
  - [x] `arb:semantic-search` - 基础语义搜索

### 验收标准
- ✅ 能够存储和检索带有元数据的 S-Expression
- ✅ 支持基于 symbol name 的精确查询和前缀匹配
- ✅ 依赖关系正确追踪

### 当前状态
✅ **已完成** - 核心知识库功能已实现，包含完整的存储、查询和依赖管理

## MVP 第三阶段：版本管理基础

### 目标：实现不可变存储和版本追踪
- [ ] Copy-on-Write 存储机制
  - [ ] 森林结构维护 S-Expression ID 表
  - [ ] 版本快照创建和切换
- [ ] 版本管理接口
  - [ ] `arb:current-version` - 获取当前版本
  - [ ] `arb:reflog` - 版本历史
  - [ ] `arb:version-info` - 版本详情
- [ ] 修改操作版本化
  - [ ] `arb:update` - 更新 S-Expression
  - [ ] `arb:delete` - 删除 S-Expression
  - [ ] 所有修改操作产生新版本

### 验收标准
- 所有修改操作创建新版本，保持历史不变
- 可以查看版本历史和切换版本
- 版本间的 delta 正确记录

## MVP 第四阶段：执行和权限系统

### 目标：实现安全的代码执行环境
- [ ] 权限层级系统
  - [ ] T0 (系统级) - 版本切换等敏感操作
  - [ ] T1 (读写级) - 允许修改知识库
  - [ ] T2 (只读级) - 仅查询和只读执行
- [ ] 执行接口
  - [ ] `arb:eval-readonly` - 只读执行
  - [ ] `arb:eval` - 读写执行
- [ ] 特殊形式
  - [ ] `arbores-ref` - ID 引用替换
  - [ ] `transaction` - 事务性修改

### 验收标准
- 权限系统正确限制不同级别的操作
- 只读执行不影响知识库状态
- 事务操作要么全部成功，要么全部回滚

## MVP 第五阶段：索引和高级查询

### 目标：实现高效的语义搜索
- [ ] 索引系统
  - [ ] 基于描述的语义索引
  - [ ] Symbol names 的倒排索引
  - [ ] 依赖关系图索引
- [ ] 高级查询接口
  - [ ] 改进的语义搜索算法
  - [ ] 模式匹配支持（通配符、正则）
  - [ ] `arb:closure` - 依赖闭包生成
- [ ] Builtin 函数集成
  - [ ] 预留 ID 空间给 builtin 函数
  - [ ] 所有 arb: 接口在知识库中可查

### 验收标准
- 语义搜索返回相关度排序的结果
- 支持复杂的符号匹配模式
- Builtin 函数通过统一接口可发现

## 后续扩展阶段

### 持久化和性能优化
- [ ] 磁盘持久化存储
- [ ] 索引优化和缓存
- [ ] 并发访问支持

### 接口扩展
- [ ] HTTP API 封装
- [ ] MCP (Model Context Protocol) 集成
- [ ] CLI 工具完善

### 高级特性
- [ ] 宏系统支持
- [ ] 模块化和命名空间
- [ ] 分布式知识库同步

## 测试策略

### 单元测试
- [x] 词法分析器测试
- [x] 语法分析器测试  
- [x] 求值器核心功能测试
- [x] 内置函数测试

### 集成测试
- [x] 端到端解释器测试
- [x] REPL 功能测试
- [x] 错误处理测试

### Scheme 兼容性测试
- [ ] R5RS 标准测试用例
- [ ] 经典 Scheme 程序测试
- [ ] 性能基准测试

## 里程碑

1. **Alpha 版本** (第2阶段完成)：基本的 Scheme 求值器 - ✅ **已完成**
2. **Beta 版本** (第3阶段完成)：完整的内置函数库 - 🚧 **进行中**
3. **Release 1.0** (第4阶段完成)：功能完整的 Scheme 解释器 - 🚧 **进行中**
4. **Release 2.0** (第5阶段完成)：扩展功能和优化

## 当前状态 (2025-07-28)

### ✅ 已完成的核心功能
- **完整的词法分析器**：
  - 支持所有基本 Token 类型、注释、转义字符、负数解析
  - **符合 R5RS 标准的标识符支持**：支持冒号及其他特殊字符 `! $ % & * + - . / : < = > ? @ ^ _ ~`
  - 这对于实现 Arbores 接口（如 `arb:create`, `arb:search`）至关重要
- **完整的语法分析器**：支持 S-表达式、嵌套列表、引用语法  
- **核心求值器**：支持自求值表达式、变量查找、函数调用
- **特殊形式**：`quote`, `if`, `lambda`, `let`, `begin`, `and`, `or`, `cond`
- **内置函数**：
  - 算术运算：`+`, `-`, `*`, `/`
  - 比较运算：`=`, `<`, `>`, `<=`, `>=`
  - 数学函数：`abs`, `max`, `min`
  - 列表操作：`cons`, `car`, `cdr`, `list`
  - 类型谓词：`null?`, `pair?`, `number?`, `string?`, `symbol?`
- **增强版 REPL 系统**：
  - 基于 `rustyline` 的现代化终端体验
  - 命令历史记录和行编辑功能
  - 多行输入支持（自动括号匹配检测）
  - 特殊命令支持（`:help`, `:symbols`, `:reset` 等）
  - 交互式和批处理模式、错误处理、管道输入支持
  - 可选的简单 REPL 模式作为后备
- **完整的测试覆盖**：每个模块都有对应的单元测试（41个通过的测试）

### 🚧 当前已知问题
1. **位置信息部分集成**：词法分析器和语法分析器支持位置跟踪，错误结构已升级支持位置信息，但实际显示位置信息的功能还未完全实现
2. **高阶函数缺失**：`map`, `filter`, `fold` 等函数式编程核心函数

### 🎯 下一步优先级
1. **完成位置信息显示**：在错误报告中实际显示位置信息，完成第一阶段剩余工作
2. **添加高阶函数**：实现 `map`, `filter`, `fold` 等函数式编程核心函数
3. **添加字符串处理函数**：`string-length`, `substring`, `string-append` 等
4. **实现尾递归优化**：提高递归函数性能
5. **添加更多语法高亮和自动补全功能**：改进 REPL 用户体验

### 📋 Environment 系统重构计划
**设计方案**：已完成，详见 `docs/environment_refactor_design.md`
- **核心思路**：Environment ID 系统，环境数据和引用分离
- **优势**：避免循环引用，保持 S-Expression 不可变，支持变量绑定修改

**实现阶段**：
- [x] **阶段 1**：创建 `EnvironmentManager` 和基础数据结构 ✅ **已完成**
- [x] **阶段 2**：更新 `Value::Lambda` 和求值器逻辑 ✅ **已完成**
- [x] **阶段 3**：实现 `define` 特殊形式 ✅ **已完成**
- [x] **阶段 4**：测试验证和性能优化 ✅ **已完成**

### 📊 完成度估算
- **第一阶段（基础架构）**：🚧 95% (位置信息基础结构已完成，但错误显示功能还需完善)
- **第二阶段（核心求值器）**：✅ 100% (已实现 `define`、逻辑运算、`cond` 和完整的 Environment 系统重构)
- **第三阶段（内置函数库）**：🚧 80% (新增比较运算符和数学函数，缺高阶函数)
- **第四阶段（高级特性）**：✅ 85% (增强版 REPL 完成，缺尾递归优化和高级语法高亮)

## 参考资料

- [Structure and Interpretation of Computer Programs](https://mitpress.mit.edu/sites/default/files/sicp/index.html)
- [The Scheme Programming Language](https://www.scheme.com/tspl4/)
- [Revised⁵ Report on Scheme](https://schemers.org/Documents/Standards/R5RS/)
- [Crafting Interpreters](https://craftinginterpreters.com/)
