# Module Design - Arbores Interpreter

## 概述

Arbores 解释器是整个系统的核心，负责解析、求值和执行 Scheme 代码。除了标准的 Scheme Standard Procedure 外，还要支持对 Arbores 代码仓库的访问。

## 设计目标

- **标准兼容**：目标支持 R7RS Scheme 标准（除部分需要被沙盒排除掉的系统 Procedure）
- **性能优化**：高效的解析和执行引擎
- **错误处理**：详细的错误信息和调试支持
- **仓库集成**：通过特殊形式与仓库管理器交互

## 架构设计

### 处理流程

```text
源代码 → (Lexer) → Token流
                    ↓
SExpr<MValue> ← (Parser)
  ↓
(Expander) → SExpr<Value>
                    ↓
      Value ← (Evaluator)
```

### 核心组件

Arbores 解释器由五个核心组件组成，每个组件都有独立的设计文档：

1. **[词法分析器 (Lexer)](./interpreter/Lexer_Design.md)** - 将源代码转换为Token流
2. **[语法分析器 (Parser)](./interpreter/Parser_Design.md)** - 将Token流解析为抽象语法树
3. **[环境管理 (Environment)](./interpreter/Environment_Management.md)** - 管理变量绑定和作用域
4. **[宏展开器 (Expander)](./interpreter/Macro_Expander.md)** - 处理宏定义和展开
5. **[执行器 (Evaluator)](./interpreter/Evaluator_Design.md)** - 求值表达式并执行代码

## 参考文档

### Scheme 标准

- **R7RS 规范**：[https://small.r7rs.org/](https://small.r7rs.org/)
- **R6RS 规范**：[http://www.r6rs.org/](http://www.r6rs.org/)
- **SRFI 库**：[https://srfi.schemers.org/](https://srfi.schemers.org/)

### 实现参考

- **Racket**：[https://racket-lang.org/](https://racket-lang.org/)
- **Guile**：[https://www.gnu.org/software/guile/](https://www.gnu.org/software/guile/)
- **Chicken Scheme**：[https://www.call-cc.org/](https://www.call-cc.org/)

### 技术资源

- **SICP**：[https://mitpress.mit.edu/sites/default/files/sicp/index.html](https://mitpress.mit.edu/sites/default/files/sicp/index.html)
- **Lisp in Small Pieces**：经典 Lisp 实现教程
- **Rust 解析器设计**：[https://rust-unofficial.github.io/too-many-lists/](https://rust-unofficial.github.io/too-many-lists/)
