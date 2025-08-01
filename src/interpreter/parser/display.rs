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
        self.fmt_with_indent(f, 0)
    }
}

impl SExpr {
    /// 带缩进的格式化输出
    pub fn fmt_with_indent(&self, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
        match &self.content {
            SExprContent::Atom(value) => {
                write!(f, "{}", value)?;
                self.write_position_comment(f)
            },
            SExprContent::Nil => {
                write!(f, "()")?;
                self.write_position_comment(f)
            },
            SExprContent::Cons { car, cdr } => {
                write!(f, "(")?;
                self.fmt_cons_content(f, car, cdr, indent)?;
                write!(f, ")")?;
                self.write_position_comment(f)
            },
            SExprContent::Vector(elements) => {
                write!(f, "#(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    element.fmt_with_indent(f, indent + 1)?;
                }
                write!(f, ")")?;
                self.write_position_comment(f)
            },
        }
    }

    /// 格式化 cons 结构的内容
    fn fmt_cons_content(&self, f: &mut Formatter<'_>, car: &SExpr, cdr: &SExpr, indent: usize) -> fmt::Result {
        car.fmt_with_indent(f, indent + 1)?;
        
        match &cdr.content {
            SExprContent::Nil => {
                // 正常列表的结尾，不需要额外输出
                Ok(())
            },
            SExprContent::Cons { car: next_car, cdr: next_cdr } => {
                // 继续列表
                write!(f, " ")?;
                self.fmt_cons_content(f, next_car, next_cdr, indent)
            },
            _ => {
                // 真正的点对
                write!(f, " . ")?;
                cdr.fmt_with_indent(f, indent + 1)
            }
        }
    }

    /// 写入位置信息注释
    fn write_position_comment(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, ";{{{},{}}}", self.span.start, self.span.end)
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
        assert_eq!(format!("{}", number), "42;{0,5}");

        let symbol = SExpr::with_span(
            SExprContent::Atom(Value::Symbol("hello".to_string())),
            span,
        );
        assert_eq!(format!("{}", symbol), "hello;{0,5}");
    }

    #[test]
    fn test_nil_display() {
        let span = Span::new(0, 2);
        let nil = SExpr::with_span(SExprContent::Nil, span);
        assert_eq!(format!("{}", nil), "();{0,2}");
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
        
        assert_eq!(format!("{}", list), "(1;{1,2} 2;{3,4});{0,5}");
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
        
        assert_eq!(format!("{}", pair), "(1;{1,2} . 2;{5,6});{0,7}");
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
        
        assert_eq!(format!("{}", vector), "#(1;{2,3} 2;{4,5});{0,6}");
    }

    #[test]
    fn test_string_escaping() {
        let span = Span::new(0, 10);
        let string_with_quotes = SExpr::with_span(
            SExprContent::Atom(Value::String("hello \"world\"".to_string())),
            span,
        );
        assert_eq!(format!("{}", string_with_quotes), "\"hello \\\"world\\\"\";{0,10}");
    }

    #[test]
    fn test_character_display() {
        let span = Span::new(0, 3);
        
        let space_char = SExpr::with_span(
            SExprContent::Atom(Value::Character(' ')),
            span,
        );
        assert_eq!(format!("{}", space_char), "#\\space;{0,3}");
        
        let newline_char = SExpr::with_span(
            SExprContent::Atom(Value::Character('\n')),
            span,
        );
        assert_eq!(format!("{}", newline_char), "#\\newline;{0,3}");
        
        let regular_char = SExpr::with_span(
            SExprContent::Atom(Value::Character('a')),
            span,
        );
        assert_eq!(format!("{}", regular_char), "#\\a;{0,3}");
    }

    #[test]
    fn test_boolean_display() {
        let span = Span::new(0, 2);
        
        let true_val = SExpr::with_span(
            SExprContent::Atom(Value::Boolean(true)),
            span,
        );
        assert_eq!(format!("{}", true_val), "#t;{0,2}");
        
        let false_val = SExpr::with_span(
            SExprContent::Atom(Value::Boolean(false)),
            span,
        );
        assert_eq!(format!("{}", false_val), "#f;{0,2}");
    }

    #[test]
    fn test_number_formatting() {
        let span = Span::new(0, 5);
        
        // 整数应该不显示小数点
        let integer = SExpr::with_span(
            SExprContent::Atom(Value::Number(42.0)),
            span,
        );
        assert_eq!(format!("{}", integer), "42;{0,5}");
        
        // 小数应该显示小数点
        let float = SExpr::with_span(
            SExprContent::Atom(Value::Number(3.14)),
            span,
        );
        assert_eq!(format!("{}", float), "3.14;{0,5}");
    }
}
