//! SExpr 的 Display trait 实现
//! 
//! 本模块负责将 SExpr AST 转换为带位置信息的 Scheme 代码字符串。
//! 位置信息以 inline comment 形式 ;{start,end} 附加在每个表达式后。

use std::fmt::{self, Display, Formatter};
use crate::interpreter::parser::types::{SExpr, SExprContent, Value};

// ============================================================================
// SExpr Display 实现
// ============================================================================

impl Display for SExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_compact(f)
    }
}

impl SExpr {
    /// 带缩进的格式化输出
    /// - indent: 缩进层级
    /// - inline: 是否内联输出（不换行）
    pub fn fmt_with_indent(&self, f: &mut Formatter<'_>, indent: usize, inline: bool) -> fmt::Result {
        match &self.content {
            SExprContent::Atom(value) => {
                write!(f, "{}", value)?;
                self.write_position_comment(f, indent, inline)
            },
            SExprContent::Nil => {
                write!(f, "()")?;
                self.write_position_comment(f, indent, inline)
            },
            SExprContent::Cons { car, cdr } => {
                write!(f, "(")?;
                self.fmt_cons_content(f, car, cdr, indent, inline)?;
                write!(f, ")")?;
                self.write_position_comment(f, indent, inline)
            },
            SExprContent::Vector(elements) => {
                write!(f, "#(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    element.fmt_with_indent(f, indent + 1, true)?; // vector 元素内联
                }
                write!(f, ")")?;
                self.write_position_comment(f, indent, inline)
            },
        }
    }

    /// 美化输出 - 带换行和缩进
    pub fn fmt_pretty(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0, false)
    }

    /// 紧凑输出 - 单行输出
    pub fn fmt_compact(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0, true)
    }

    /// 生成美化输出字符串
    pub fn to_pretty_string(&self) -> String {
        let mut lines = Vec::new();
        self.fmt_pretty_to_lines(&mut lines, 0);
        
        // 计算最大内容长度用于对齐
        let max_content_len = lines.iter()
            .map(|(content, _)| content.len())
            .max()
            .unwrap_or(0);
        
        // 生成最终输出，span 信息右对齐
        lines.into_iter()
            .map(|(content, span_info)| {
                if span_info.is_empty() {
                    content
                } else {
                    format!("{:<width$} {}", content, span_info, width = max_content_len)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 递归生成内容和 span 信息行的辅助方法
    fn fmt_pretty_to_lines(&self, lines: &mut Vec<(String, String)>, indent: usize) {
        let indent_str = "  ".repeat(indent); // 每层缩进 2 个空格
        
        match &self.content {
            SExprContent::Atom(value) => {
                let content = format!("{}{}", indent_str, value);
                let span_info = format!("#; ({} {})", self.span.start, self.span.end);
                lines.push((content, span_info));
            },
            SExprContent::Nil => {
                let content = format!("{}()", indent_str);
                let span_info = format!("#; ({} {})", self.span.start, self.span.end);
                lines.push((content, span_info));
            },
            SExprContent::Cons { car, cdr } => {
                // 开始括号
                let open_content = format!("{}(", indent_str);
                lines.push((open_content, String::new()));
                
                // 递归处理 cons 内容
                self.fmt_cons_pretty_to_lines(lines, car, cdr, indent + 1);
                
                // 结束括号和 span 信息
                let close_content = format!("{})", indent_str);
                let span_info = format!("#; ({} {})", self.span.start, self.span.end);
                lines.push((close_content, span_info));
            },
            SExprContent::Vector(elements) => {
                if elements.is_empty() {
                    let content = format!("{}#()", indent_str);
                    let span_info = format!("#; ({} {})", self.span.start, self.span.end);
                    lines.push((content, span_info));
                } else {
                    // 所有非空向量都使用多行格式，以便为每个元素添加 span 信息
                    let open_content = format!("{}#(", indent_str);
                    lines.push((open_content, String::new()));
                    
                    // 递归处理每个元素
                    for element in elements {
                        element.fmt_pretty_to_lines(lines, indent + 1);
                    }
                    
                    // 结束括号和 span 信息
                    let close_content = format!("{})", indent_str);
                    let span_info = format!("#; ({} {})", self.span.start, self.span.end);
                    lines.push((close_content, span_info));
                }
            },
        }
    }

    /// 递归格式化 cons 结构的内容（美化版本）
    fn fmt_cons_pretty_to_lines(&self, lines: &mut Vec<(String, String)>, car: &SExpr, cdr: &SExpr, indent: usize) {
        // 处理第一个元素
        car.fmt_pretty_to_lines(lines, indent);
        
        match &cdr.content {
            SExprContent::Nil => {
                // 正常列表的结尾，不需要额外处理
            },
            SExprContent::Cons { car: next_car, cdr: next_cdr } => {
                // 继续列表的下一个元素
                self.fmt_cons_pretty_to_lines(lines, next_car, next_cdr, indent);
            },
            _ => {
                // 真正的点对：添加点和 cdr
                let indent_str = "  ".repeat(indent);
                let dot_content = format!("{}.", indent_str);
                lines.push((dot_content, String::new()));
                cdr.fmt_pretty_to_lines(lines, indent);
            }
        }
    }

    /// 格式化 cons 结构的内容
    fn fmt_cons_content(&self, f: &mut Formatter<'_>, car: &SExpr, cdr: &SExpr, indent: usize, inline: bool) -> fmt::Result {
        car.fmt_with_indent(f, indent + 1, true)?; // cons 内部元素内联
        
        match &cdr.content {
            SExprContent::Nil => {
                // 正常列表的结尾，不需要额外输出
                Ok(())
            },
            SExprContent::Cons { car: next_car, cdr: next_cdr } => {
                // 继续列表
                write!(f, " ")?;
                self.fmt_cons_content(f, next_car, next_cdr, indent, inline)
            },
            _ => {
                // 真正的点对
                write!(f, " . ")?;
                cdr.fmt_with_indent(f, indent + 1, true) // 点对的 cdr 内联
            }
        }
    }

    /// 写入位置信息注释
    fn write_position_comment(&self, f: &mut Formatter<'_>, indent: usize, inline: bool) -> fmt::Result {
        write!(f, " #; ({} {})", self.span.start, self.span.end)?;
        
        if !inline {
            // 换行并添加缩进
            writeln!(f)?;
            if indent > 0 {
                for _ in 0..indent {
                    write!(f, "  ")?; // 每层缩进 2 个空格
                }
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// Value Display 实现
// ============================================================================

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                // 如果是整数，不显示小数点
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            },
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Character(c) => write!(f, "#\\{}", escape_character(*c)),
            Value::Boolean(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Value::Symbol(s) => write!(f, "{}", s),
        }
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 转义字符串中的特殊字符
fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// 转义字符字面量
fn escape_character(c: char) -> String {
    match c {
        ' ' => "space".to_string(),
        '\n' => "newline".to_string(),
        '\r' => "return".to_string(),
        '\t' => "tab".to_string(),
        '(' => "(".to_string(),
        ')' => ")".to_string(),
        _ => c.to_string(),
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::lexer::types::Span;
    use std::rc::Rc;

    #[test]
    fn test_atom_display() {
        let span = Span::new(0, 5);
        let number = SExpr::with_span(
            SExprContent::Atom(Value::Number(42.0)),
            span,
        );
        assert_eq!(format!("{}", number), "42 #; (0 5)");

        let symbol = SExpr::with_span(
            SExprContent::Atom(Value::Symbol("hello".to_string())),
            span,
        );
        assert_eq!(format!("{}", symbol), "hello #; (0 5)");
    }

    #[test]
    fn test_nil_display() {
        let span = Span::new(0, 2);
        let nil = SExpr::with_span(SExprContent::Nil, span);
        assert_eq!(format!("{}", nil), "() #; (0 2)");
    }

    #[test]
    fn test_simple_list_display() {
        let span1 = Span::new(1, 2);
        let span2 = Span::new(3, 4);
        let span_list = Span::new(0, 5);
        
        let car = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Number(1.0)),
            span1,
        ));
        let cdr = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Number(2.0)),
            span2,
        ));
        let nil = Rc::new(SExpr::with_span(SExprContent::Nil, Span::new(4, 5)));
        
        // 创建 (1 2)
        let cdr_cons = Rc::new(SExpr::with_span(
            SExprContent::Cons { car: cdr, cdr: nil },
            Span::new(3, 5),
        ));
        let list = SExpr::with_span(
            SExprContent::Cons { car, cdr: cdr_cons },
            span_list,
        );
        
        assert_eq!(format!("{}", list), "(1 #; (1 2) 2 #; (3 4)) #; (0 5)");
    }

    #[test]
    fn test_dotted_pair_display() {
        let span1 = Span::new(1, 2);
        let span2 = Span::new(5, 6);
        let span_pair = Span::new(0, 7);
        
        let car = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Number(1.0)),
            span1,
        ));
        let cdr = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Number(2.0)),
            span2,
        ));
        
        let pair = SExpr::with_span(
            SExprContent::Cons { car, cdr },
            span_pair,
        );
        
        assert_eq!(format!("{}", pair), "(1 #; (1 2) . 2 #; (5 6)) #; (0 7)");
    }

    #[test]
    fn test_vector_display() {
        let span1 = Span::new(2, 3);
        let span2 = Span::new(4, 5);
        let span_vector = Span::new(0, 6);
        
        let elem1 = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Number(1.0)),
            span1,
        ));
        let elem2 = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Number(2.0)),
            span2,
        ));
        
        let vector = SExpr::with_span(
            SExprContent::Vector(vec![elem1, elem2]),
            span_vector,
        );
        
        assert_eq!(format!("{}", vector), "#(1 #; (2 3) 2 #; (4 5)) #; (0 6)");
    }

    #[test]
    fn test_string_escaping() {
        let span = Span::new(0, 10);
        let string_with_quotes = SExpr::with_span(
            SExprContent::Atom(Value::String("hello \"world\"".to_string())),
            span,
        );
        assert_eq!(format!("{}", string_with_quotes), "\"hello \\\"world\\\"\" #; (0 10)");
    }

    #[test]
    fn test_character_display() {
        let span = Span::new(0, 3);
        
        let space_char = SExpr::with_span(
            SExprContent::Atom(Value::Character(' ')),
            span,
        );
        assert_eq!(format!("{}", space_char), "#\\space #; (0 3)");
        
        let newline_char = SExpr::with_span(
            SExprContent::Atom(Value::Character('\n')),
            span,
        );
        assert_eq!(format!("{}", newline_char), "#\\newline #; (0 3)");
        
        let regular_char = SExpr::with_span(
            SExprContent::Atom(Value::Character('a')),
            span,
        );
        assert_eq!(format!("{}", regular_char), "#\\a #; (0 3)");
    }

    #[test]
    fn test_boolean_display() {
        let span = Span::new(0, 2);
        
        let true_val = SExpr::with_span(
            SExprContent::Atom(Value::Boolean(true)),
            span,
        );
        assert_eq!(format!("{}", true_val), "#t #; (0 2)");
        
        let false_val = SExpr::with_span(
            SExprContent::Atom(Value::Boolean(false)),
            span,
        );
        assert_eq!(format!("{}", false_val), "#f #; (0 2)");
    }

    #[test]
    fn test_number_formatting() {
        let span = Span::new(0, 5);
        
        // 整数应该不显示小数点
        let integer = SExpr::with_span(
            SExprContent::Atom(Value::Number(42.0)),
            span,
        );
        assert_eq!(format!("{}", integer), "42 #; (0 5)");
        
        // 小数应该显示小数点
        let float = SExpr::with_span(
            SExprContent::Atom(Value::Number(3.14)),
            span,
        );
        assert_eq!(format!("{}", float), "3.14 #; (0 5)");
    }

    #[test]
    fn test_pretty_string_simple() {
        let span = Span::new(0, 2);
        let atom = SExpr::with_span(
            SExprContent::Atom(Value::Number(42.0)),
            span,
        );
        
        // 简单原子的美化输出应该与紧凑输出相同
        assert_eq!(atom.to_pretty_string(), "42 #; (0 2)");
    }

    #[test]
    fn test_pretty_string_list() {
        // 构造一个简单列表 (a b c)
        let span_a = Span::new(1, 2);
        let span_b = Span::new(3, 4);
        let span_c = Span::new(5, 6);
        let span_list = Span::new(0, 7);
        
        let a = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Symbol("a".to_string())),
            span_a,
        ));
        let b = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Symbol("b".to_string())),
            span_b,
        ));
        let c = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Symbol("c".to_string())),
            span_c,
        ));
        
        let nil = Rc::new(SExpr::with_span(SExprContent::Nil, span_c));
        let list_c = Rc::new(SExpr::with_span(
            SExprContent::Cons { car: c, cdr: nil },
            span_list,
        ));
        let list_bc = Rc::new(SExpr::with_span(
            SExprContent::Cons { car: b, cdr: list_c },
            span_list,
        ));
        let list_abc = SExpr::with_span(
            SExprContent::Cons { car: a, cdr: list_bc },
            span_list,
        );
        
        let pretty = list_abc.to_pretty_string();
        // 美化输出应该包含换行和缩进
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  a")); // 缩进的 a
        assert!(pretty.contains("  b")); // 缩进的 b
        assert!(pretty.contains("  c")); // 缩进的 c
        assert!(pretty.contains("#; (1 2)")); // span 注释
        assert!(pretty.contains("#; (3 4)")); // span 注释
        assert!(pretty.contains("#; (5 6)")); // span 注释
        assert!(pretty.contains("#; (0 7)")); // 外层 span 注释
    }

    #[test]
    fn test_pretty_string_nested() {
        // 构造嵌套列表 (a (b c))
        let span_a = Span::new(1, 2);
        let span_b = Span::new(4, 5);
        let span_c = Span::new(6, 7);
        let span_inner = Span::new(3, 8);
        let span_outer = Span::new(0, 9);
        
        let a = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Symbol("a".to_string())),
            span_a,
        ));
        let b = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Symbol("b".to_string())),
            span_b,
        ));
        let c = Rc::new(SExpr::with_span(
            SExprContent::Atom(Value::Symbol("c".to_string())),
            span_c,
        ));
        
        let nil = Rc::new(SExpr::with_span(SExprContent::Nil, span_c));
        let inner_list = Rc::new(SExpr::with_span(
            SExprContent::Cons { 
                car: b, 
                cdr: Rc::new(SExpr::with_span(
                    SExprContent::Cons { car: c, cdr: nil.clone() },
                    span_inner,
                ))
            },
            span_inner,
        ));
        
        let outer_list = SExpr::with_span(
            SExprContent::Cons { 
                car: a, 
                cdr: Rc::new(SExpr::with_span(
                    SExprContent::Cons { car: inner_list, cdr: nil },
                    span_outer,
                ))
            },
            span_outer,
        );
        
        let pretty = outer_list.to_pretty_string();
        // 嵌套结构应该有多层缩进
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  a")); // 第一层缩进的 a
        assert!(pretty.contains("    b")); // 第二层缩进的 b
        assert!(pretty.contains("    c")); // 第二层缩进的 c
        assert!(pretty.contains("#; (1 2)")); // a 的 span 注释
        assert!(pretty.contains("#; (4 5)")); // b 的 span 注释
        assert!(pretty.contains("#; (6 7)")); // c 的 span 注释
        assert!(pretty.contains("#; (3 8)")); // 内层列表的 span 注释
        assert!(pretty.contains("#; (0 9)")); // 外层列表的 span 注释
    }
}
