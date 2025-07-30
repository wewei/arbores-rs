# 🌲 增强版 REPL 功能展示

## 新功能概览

我们的 Scheme 解释器现在具备了现代化的 REPL 体验！

### 🚀 启动增强版 REPL

```bash
# 启动默认（增强版）REPL
./target/release/arbores

# 或者明确指定增强模式
./target/release/arbores --repl enhanced

# 使用简单模式（后备选项）
./target/release/arbores --repl simple
```

### ✨ 增强功能

#### 1. 命令历史和行编辑
- **↑/↓ 箭头**：浏览命令历史
- **Ctrl+A/E**：移动到行首/行尾
- **Ctrl+L**：清屏
- **Ctrl+C**：中断当前输入
- **Ctrl+D**：退出 REPL

#### 2. 多行输入支持
```scheme
arbores> (begin
      ..   (define x 42)
      ..   (+ x 8))
50
```

#### 3. 特殊命令
```scheme
arbores> :help        # 显示帮助信息
arbores> :symbols     # 列出可用符号
arbores> :keywords    # 显示 Scheme 关键字
arbores> :clear       # 清屏
arbores> :reset       # 重置解释器状态
arbores> :exit        # 退出
```

#### 4. 文件执行
```bash
# 执行 Scheme 文件
./target/release/arbores examples/enhanced_repl_demo.scm

# 管道输入
echo "(+ 1 2 3)" | ./target/release/arbores
```

### 🎨 用户界面改进

启动时显示：
```
🌲 Arbores Scheme Interpreter v0.1.0 (Enhanced Mode)
Type :help for help, :exit to quit, or Ctrl+D to exit.
Features: History ✓ Line editing ✓ Multi-line ✓

arbores> 
```

### 📚 示例会话

```scheme
🌲 Arbores Scheme Interpreter v0.1.0 (Enhanced Mode)
Type :help for help, :exit to quit, or Ctrl+D to exit.
Features: History ✓ Line editing ✓ Multi-line ✓

arbores> (define factorial 
      ..   (lambda (n)
      ..     (if (= n 0) 1
      ..         (* n (factorial (- n 1))))))
()

arbores> (factorial 5)
120

arbores> :symbols
Available symbols: quote, if, lambda, let, begin, and, or, cond, define, set!, +, -, *, /, =, <, >, <=, >=, abs, max, min, cons, car, cdr, list, null?, pair?, number?, string?, symbol?, #t, #f, true, false, factorial

arbores> :exit
Goodbye!
```

### 🔧 技术实现

- **基础库**：`rustyline` v10.1.1 提供终端功能
- **命令解析**：`clap` v4.0 处理命令行参数  
- **自动回退**：如果增强模式失败，自动切换到简单模式
- **括号匹配**：智能检测不完整的表达式以支持多行输入

### 📈 性能与兼容性

- ✅ 所有 39 个现有测试继续通过
- ✅ 向后兼容简单 REPL 模式
- ✅ 支持批处理和交互模式
- ✅ 管道输入和文件执行正常工作

这一更新显著提升了开发者使用 Arbores 的体验，使其更接近现代编程语言解释器的标准！
