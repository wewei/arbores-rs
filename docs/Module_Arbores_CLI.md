# Module Design - Arbores CLI

## 命令格式

### 基本用法

```bash
# 启动交互式 REPL
arbores

# 执行 Scheme 文件
arbores script.scm

# 执行单个表达式
# 执行单个表达式
arbores -e '(+ 1 2 3)'

# 从标准输入读取
echo '(* 4 5)' | arbores --
```

### 命令行参数

| 参数 | 短选项 | 长选项 | 描述 | 示例 |
|------|--------|--------|------|------|
| 文件路径 | - | - | 执行指定的 Scheme 文件 | `arbores examples/test.scm` |
| 表达式求值 | `-e` | `--eval` | 求值表达式并退出 | `arbores -e '(+ 1 2)'` |
| 标准输入 | - | `--` | 从标准输入读取代码 | `echo "(+ 1 2)" \| arbores --` |
| 版本信息 | - | `--version` | 显示版本信息 | `arbores --version` |
| 帮助信息 | `-h` | `--help` | 显示帮助信息 | `arbores --help` |

### REPL 交互模式

#### 启动界面

```text
Arbores Scheme Interpreter v0.1.0
Type :help for help, :exit to quit, or Ctrl+D to exit.

arbores> 
```

#### 特殊命令

```scheme
arbores> :help        # 显示帮助信息
arbores> :symbols     # 列出可用符号
arbores> :keywords    # 显示 Scheme 关键字
arbores> :clear       # 清屏
arbores> :reset       # 重置解释器状态
arbores> :history     # 命令历史说明
arbores> :exit        # 退出
```

#### 多行输入支持

```scheme
arbores> (begin
      ..   (define x 42)
      ..   (+ x 8))
50
```

#### 键盘快捷键

- **↑/↓ 箭头**：浏览命令历史
- **Ctrl+A/E**：移动到行首/行尾
- **Ctrl+L**：清屏
- **Ctrl+C**：中断当前输入
- **Ctrl+D**：退出 REPL

### 输出格式

所有输出格式由 `formatters.rs` 模块统一处理，确保一致性和美观性。

#### 成功执行

```bash
$ arbores -e '(+ 1 2 3)'
6

$ arbores examples/test.scm
Hello, World!
42
```

#### 错误处理

```bash
$ arbores -e '(+ 1 "not a number")'
Error evaluating expression: Type error: Expected number, got string

$ arbores nonexistent.scm
Error reading file 'nonexistent.scm': No such file or directory (os error 2)
```

#### 数据类型格式化

`formatters.rs` 负责将各种 Scheme 数据类型转换为用户友好的输出格式：

- **数字**：整数和浮点数的精确显示
- **字符串**：适当的引号和转义处理
- **列表**：括号结构的美化打印，支持自动换行
- **符号**：清晰的标识符显示
- **错误**：带位置信息的详细错误报告

#### 终端自适应格式化

`formatters.rs` 考虑终端宽度以确保输出美观：

- **动态宽度检测**：自动获取当前终端宽度
- **智能换行**：长列表和复杂结构的自动换行
- **对齐和缩进**：根据终端宽度调整缩进级别
- **截断处理**：过长输出的优雅截断和省略
- **响应式布局**：窄终端下的紧凑显示模式

## 技术选型

### Rustyline (v10.0)

- **用途**：现代化 REPL 实现
- **功能**：
  - 命令历史记录和浏览
  - 行编辑功能（Emacs 风格快捷键）
  - 多行输入支持（自动括号匹配检测）
  - 中断处理（Ctrl+C）
- **优势**：
  - 成熟稳定，广泛使用
  - 类似 GNU Readline 的功能
  - 与 Rust 生态系统完美集成
  - 支持自定义补全和历史管理

### Clap (v4.0)

- **用途**：命令行参数解析
- **功能**：
  - 声明式参数定义
  - 自动生成帮助信息
  - 子命令支持
  - 参数验证
- **优势**：
  - 类型安全的参数处理
  - 优秀的用户体验
  - 强大的错误处理

### atty (v0.2)

- **用途**：终端类型检测与适配
- **功能**：
  - 检测 stdin 是否连接到终端
  - 支持管道输入检测
  - 检测标准输出是否为终端
  - 识别终端类型和功能
- **使用场景**：
  - 自动切换交互模式和批处理模式
  - 处理管道输入
  - 根据终端环境调整输出格式
  - 优化输出宽度和换行策略
  - 智能禁用彩色输出（当重定向到文件时）

## 架构设计

### 模块结构

```text
src/cli/
├── main.rs              # 主入口点
├── repl.rs              # REPL 实现
├── formatters.rs        # 输出格式化器
└── mod.rs               # 模块声明
```

### 工作流程

1. **参数解析**：(main.rs) 使用 Clap 解析命令行参数
2. **模式判断**：(main.rs) 根据参数和 stdin 状态选择执行模式
3. **REPL 启动**：(repl.rs) 使用 Rustyline 提供交互体验
4. **表达式求值**：(repl.rs) 调用核心求值器处理 Scheme 代码
5. **结果输出**：(formatters.rs) 格式化输出结果或错误信息

### 错误处理策略

- **语法错误**：显示错误位置和详细描述
- **运行时错误**：提供调用栈信息
- **文件错误**：显示系统错误信息
- **中断处理**：优雅处理用户中断信号

#### 模块职责

##### main.rs

- 命令行参数解析和验证
- 执行模式选择（REPL/文件执行/表达式求值）
- 程序入口点和错误处理

##### repl.rs

- 交互式 REPL 循环实现
- 命令历史和多行输入管理
- 特殊命令处理（:help, :exit 等）
- Rustyline 集成

##### formatters.rs

- Scheme 数据类型的显示格式化
- 错误信息的友好输出
- 调试信息和调用栈格式化
- 结果值的美化打印
- 终端宽度自适应布局
- 响应式输出格式调整

##### mod.rs

- 模块导出和公共接口定义
- 内部模块组织和依赖管理

### 性能考虑

#### 启动优化

- 延迟初始化非必要组件
- 快速路径处理简单表达式
- 缓存常用的内置函数

#### 内存管理

- 智能历史记录限制
- 及时清理临时对象
- 避免不必要的字符串克隆

#### 响应性

- 异步信号处理
- 非阻塞输入处理
- 渐进式表达式解析

## 参考文档

### Rustyline 相关资源

#### 官方文档

- **Crate 页面**：[https://crates.io/crates/rustyline](https://crates.io/crates/rustyline)
- **API 文档**：[https://docs.rs/rustyline/](https://docs.rs/rustyline/)
- **GitHub 仓库**：[https://github.com/kkawakam/rustyline](https://github.com/kkawakam/rustyline)

#### 核心特性文档

- **Editor 配置**：[https://docs.rs/rustyline/latest/rustyline/struct.Editor.html](https://docs.rs/rustyline/latest/rustyline/struct.Editor.html)
- **历史管理**：[https://docs.rs/rustyline/latest/rustyline/history/index.html](https://docs.rs/rustyline/latest/rustyline/history/index.html)
- **补全系统**：[https://docs.rs/rustyline/latest/rustyline/completion/index.html](https://docs.rs/rustyline/latest/rustyline/completion/index.html)
- **键盘绑定**：[https://docs.rs/rustyline/latest/rustyline/config/index.html](https://docs.rs/rustyline/latest/rustyline/config/index.html)

#### 使用示例

- **基础 REPL**：[https://github.com/kkawakam/rustyline/blob/master/examples/example.rs](https://github.com/kkawakam/rustyline/blob/master/examples/example.rs)
- **自定义补全**：[https://github.com/kkawakam/rustyline/blob/master/examples/custom_helper.rs](https://github.com/kkawakam/rustyline/blob/master/examples/custom_helper.rs)
- **配置示例**：[https://github.com/kkawakam/rustyline/blob/master/examples/config.rs](https://github.com/kkawakam/rustyline/blob/master/examples/config.rs)

### 命令行设计参考

#### 类似项目

- **Racket REPL**：[https://docs.racket-lang.org/guide/intro.html#%28part._repl%29](https://docs.racket-lang.org/guide/intro.html#%28part._repl%29)
- **MIT Scheme**：[https://www.gnu.org/software/mit-scheme/documentation/stable/mit-scheme-user/](https://www.gnu.org/software/mit-scheme/documentation/stable/mit-scheme-user/)
- **Guile Scheme**：[https://www.gnu.org/software/guile/manual/html_node/Using-Guile-Interactively.html](https://www.gnu.org/software/guile/manual/html_node/Using-Guile-Interactively.html)

#### CLI 设计最佳实践

- **GNU 命令行标准**：[https://www.gnu.org/prep/standards/standards.html#Command_002dLine-Interfaces](https://www.gnu.org/prep/standards/standards.html#Command_002dLine-Interfaces)
- **Clap 用户指南**：[https://docs.rs/clap/latest/clap/_tutorial/index.html](https://docs.rs/clap/latest/clap/_tutorial/index.html)
- **Rust CLI 工作组**：[https://rust-cli.github.io/book/](https://rust-cli.github.io/book/)

### 终端技术参考

#### 终端格式化和布局

- **终端宽度检测**：[https://docs.rs/terminal_size/latest/terminal_size/](https://docs.rs/terminal_size/latest/terminal_size/)
- **文本换行算法**：[https://docs.rs/textwrap/latest/textwrap/](https://docs.rs/textwrap/latest/textwrap/)
- **Pretty printing**：[https://docs.rs/pretty/latest/pretty/](https://docs.rs/pretty/latest/pretty/)

#### ANSI 转义序列

- **颜色和样式**：[https://en.wikipedia.org/wiki/ANSI_escape_code](https://en.wikipedia.org/wiki/ANSI_escape_code)
- **光标控制**：[https://invisible-island.net/xterm/ctlseqs/ctlseqs.html](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)

#### 跨平台兼容性

- **Windows 终端支持**：[https://docs.microsoft.com/en-us/windows/terminal/](https://docs.microsoft.com/en-us/windows/terminal/)
- **Unix 终端标准**：[https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap11.html](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap11.html)
