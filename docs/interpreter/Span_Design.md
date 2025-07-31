# Span 位置设计

状态：Reviewed

## 概述

Span 是解释器中用于表示源代码位置范围的核心数据类型。本文档详细描述了 Span 的数学逻辑、设计原则和实现策略。

## 设计原则

### 1. 数学一致性

**字符偏移量作为基础单位**：
- 使用 `usize` 类型的字符偏移量作为位置的唯一标识
- 对于位置 `n`，其下一个位置总是 `n + 1`，数学上明确且唯一
- 避免了 `(row, column)` 坐标系统中的歧义性问题

**左闭右开区间表示**：
- Span 表示范围 `[start, end)`，包含 `start` 位置，不包含 `end` 位置
- 这种表示法在计算机科学中广泛使用，具有良好的数学性质
- 空区间表示为 `{start: x, end: x}`，表示位置 `x` 处的零长度范围

### 2. 简化计算逻辑

**位置推进的确定性**：
```rust
// 简单且确定的位置推进
let next_position = current_position + 1;

// 区间长度计算
let length = span.end - span.start;

// 区间合并
let merged_span = Span::new(span1.start, span2.end);
```

**避免编码相关的复杂性**：
- 字符偏移量与具体的字节编码（UTF-8、UTF-16等）无关
- 不需要考虑多字节字符对位置计算的影响
- 位置计算与字符的显示宽度无关

## 数据结构设计

### Span 核心定义

```rust
/// 源代码位置范围 - 基于字符偏移量的左闭右开区间
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// 起始位置（字符偏移量，包含）
    pub start: usize,
    /// 结束位置（字符偏移量，不包含）
    pub end: usize,
}
```

### 核心方法实现

```rust
impl Span {
    /// 创建新的 Span
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "Span start must be <= end");
        Self { start, end }
    }
    
    /// 创建单个字符位置的 Span
    pub fn single_char(position: usize) -> Self {
        Self::new(position, position + 1)
    }
    
    /// 创建空 Span（零长度区间）
    pub fn empty(position: usize) -> Self {
        Self::new(position, position)
    }
    
    /// 获取 Span 的长度（字符数）
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// 判断 Span 是否为空
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    
    /// 判断 Span 是否包含指定位置
    pub fn contains(&self, position: usize) -> bool {
        self.start <= position && position < self.end
    }
    
    /// 判断 Span 是否包含另一个 Span
    pub fn contains_span(&self, other: &Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }
    
    /// 判断两个 Span 是否有重叠
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }
    
    /// 合并两个 Span，返回包含两者的最小 Span
    pub fn merge(&self, other: &Span) -> Span {
        Span::new(
            self.start.min(other.start),
            self.end.max(other.end),
        )
    }
    
    /// 扩展 Span 到指定位置
    pub fn extend_to(&self, position: usize) -> Span {
        Span::new(self.start, self.end.max(position))
    }
    
    /// 将 Span 向右偏移指定距离
    pub fn offset(&self, delta: usize) -> Span {
        Span::new(self.start + delta, self.end + delta)
    }
}
```

## 数学性质

### 1. 区间运算

**长度计算**：
```rust
assert_eq!(Span::new(5, 10).len(), 5);  // [5, 10) 包含 5 个字符
assert_eq!(Span::empty(5).len(), 0);    // [5, 5) 包含 0 个字符
```

**包含关系**：
```rust
let span = Span::new(10, 20);
assert!(span.contains(10));    // 包含起始位置
assert!(span.contains(15));    // 包含中间位置
assert!(!span.contains(20));   // 不包含结束位置
```

**重叠检测**：
```rust
let span1 = Span::new(10, 15);
let span2 = Span::new(12, 18);
assert!(span1.overlaps(&span2));  // [10,15) 与 [12,18) 重叠
```

### 2. 空区间的特殊性质

**空区间表示**：
```rust
let empty_at_5 = Span::empty(5);  // [5, 5)
assert_eq!(empty_at_5.start, 5);
assert_eq!(empty_at_5.end, 5);
assert!(empty_at_5.is_empty());
```

**空区间的应用场景**：
- 表示插入点位置（如光标位置）
- 表示语法糖展开时的虚拟元素位置
- 表示列表尾部的 `nil` 位置

## 位置转换

由于某些场景下仍需要行列信息（如错误报告），提供转换功能：

### PositionConverter 设计

```rust
/// 位置转换器 - 将字符偏移量转换为行列信息
pub struct PositionConverter {
    /// 每行的起始字符偏移量
    line_starts: Vec<usize>,
}

impl PositionConverter {
    /// 从源代码文本创建转换器
    pub fn from_source(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (offset, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(offset + ch.len_utf8());
            }
        }
        Self { line_starts }
    }
    
    /// 将字符偏移量转换为行列位置
    pub fn to_line_column(&self, char_offset: usize) -> (usize, usize) {
        match self.line_starts.binary_search(&char_offset) {
            Ok(line) => (line + 1, 1),  // 直接在行首
            Err(line) => {
                let line_start = self.line_starts[line - 1];
                (line, char_offset - line_start + 1)
            }
        }
    }
    
    /// 将 Span 转换为行列范围
    pub fn span_to_line_column(&self, span: &Span) -> ((usize, usize), (usize, usize)) {
        (
            self.to_line_column(span.start),
            self.to_line_column(span.end),
        )
    }
}
```

## 使用示例

### 基本操作

```rust
// 创建 Span
let span1 = Span::new(10, 15);          // [10, 15)
let span2 = Span::single_char(20);      // [20, 21)
let span3 = Span::empty(25);            // [25, 25)

// 计算长度
assert_eq!(span1.len(), 5);
assert_eq!(span2.len(), 1);
assert_eq!(span3.len(), 0);

// 合并 Span
let merged = span1.merge(&span2);       // [10, 21)
assert_eq!(merged, Span::new(10, 21));
```

### 在词法分析中的应用

```rust
// Token 中的 Span
struct Token {
    pub token_type: TokenType,
    pub span: Span,
    pub raw_text: String,
}

// 创建 Token
let token = Token {
    token_type: TokenType::Number(42.0),
    span: Span::new(10, 12),  // "42" 占据字符 10-11
    raw_text: "42".to_string(),
};
```

### 在语法分析中的应用

```rust
// S表达式中的 Span
struct SExpr {
    pub content: SExprContent,
    pub span: Option<Rc<Span>>,
}

// 引用表达式的 Span 分配
fn parse_quoted_expression(quote_pos: usize, expr: SExpr) -> SExpr {
    let quote_span = Span::single_char(quote_pos);  // "'" 位置
    let expr_span = expr.span.unwrap();
    
    // 整个表达式的 Span
    let full_span = quote_span.merge(&expr_span);
    
    // 尾部 nil 的空 Span
    let nil_span = Span::empty(expr_span.end);
    
    // ...构建表达式
}
```

## 错误报告中的应用

```rust
// 错误报告使用行列信息
pub fn report_error(span: &Span, source: &str, message: &str) {
    let converter = PositionConverter::from_source(source);
    let (start_line, start_col) = converter.to_line_column(span.start);
    let (end_line, end_col) = converter.to_line_column(span.end);
    
    if start_line == end_line {
        println!("Error at line {}, columns {}-{}: {}", 
                 start_line, start_col, end_col, message);
    } else {
        println!("Error from line {}:{} to line {}:{}: {}", 
                 start_line, start_col, end_line, end_col, message);
    }
}
```

## 设计优势

### 1. 数学简洁性
- 位置计算简单直观：`next = current + 1`
- 区间运算符合数学直觉：长度 = `end - start`
- 避免了坐标系统的歧义性

### 2. 实现简单性
- 无需考虑换行符对位置计算的影响
- 无需处理不同编码格式的复杂性
- 区间操作（合并、包含、重叠）实现简单

### 3. 性能优势
- 位置比较和计算都是 O(1) 操作
- 无需维护复杂的行列映射状态
- 内存占用最小（两个 usize 值）

### 4. 扩展性
- 可以按需转换为行列信息
- 支持各种区间操作和查询
- 便于实现复杂的源码分析功能

## 总结

基于字符偏移量的 Span 设计提供了数学上简洁、实现上高效的位置表示方案。通过左闭右开区间的表示法，我们获得了清晰的区间语义和便利的操作接口。同时，通过 PositionConverter 可以在需要时转换为用户友好的行列信息，实现了内部简洁性和外部易用性的平衡。
