use std::rc::Rc;
use crate::legacy::lexer::{Lexer, Token, LocatedToken};
use crate::legacy::types::{Value, SchemeError, Result, Position, LocatedValue};

/// 语法分析器
pub struct Parser {
    tokens: Vec<LocatedToken>,
    position: usize,
}

impl Parser {
    /// 创建新的语法分析器
    pub fn new(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_with_positions()
            .map_err(|e| SchemeError::SyntaxError(e, None))?;
        
        Ok(Parser {
            tokens,
            position: 0,
        })
    }

    /// 获取当前 token
    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).map(|lt| &lt.token).unwrap_or(&Token::EOF)
    }

    /// 获取当前位置信息
    fn current_position(&self) -> Option<Position> {
        self.tokens.get(self.position).map(|lt| lt.position.clone())
    }

    /// 移动到下一个 token
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    /// 解析单个表达式
    pub fn parse_expression(&mut self) -> Result<Value> {
        match self.current_token().clone() {
            Token::EOF => Err(SchemeError::SyntaxError("Unexpected end of input".to_string(), self.current_position())),
            
            Token::Integer(n) => {
                self.advance();
                Ok(Value::Integer(n))
            },
            
            Token::Float(f) => {
                self.advance();
                Ok(Value::Float(f))
            },
            
            Token::String(s) => {
                self.advance();
                Ok(Value::String(s))
            },
            
            Token::Symbol(s) => {
                self.advance();
                Ok(Value::Symbol(s))
            },
            
            Token::Boolean(b) => {
                self.advance();
                Ok(Value::Bool(b))
            },
            
            Token::Quote => {
                let quote_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In quoted expression: {}", msg), quote_pos),
                        other => other,
                    })?;
                Ok(Value::from_vec(vec![
                    Value::Symbol("quote".to_string()),
                    expr
                ]))
            },
            
            Token::Quasiquote => {
                let quasiquote_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In quasiquoted expression: {}", msg), quasiquote_pos),
                        other => other,
                    })?;
                Ok(Value::from_vec(vec![
                    Value::Symbol("quasiquote".to_string()),
                    expr
                ]))
            },
            
            Token::Unquote => {
                let unquote_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In unquoted expression: {}", msg), unquote_pos),
                        other => other,
                    })?;
                Ok(Value::from_vec(vec![
                    Value::Symbol("unquote".to_string()),
                    expr
                ]))
            },
            
            Token::UnquoteSplicing => {
                let unquote_splicing_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In unquote-splicing expression: {}", msg), unquote_splicing_pos),
                        other => other,
                    })?;
                Ok(Value::from_vec(vec![
                    Value::Symbol("unquote-splicing".to_string()),
                    expr
                ]))
            },
            
            Token::LeftParen => {
                self.advance();
                self.parse_list()
            },
            
            _ => Err(SchemeError::SyntaxError(
                format!("Unexpected token: {}", self.current_token()), self.current_position()
            )),
        }
    }

    /// 解析列表
    fn parse_list(&mut self) -> Result<Value> {
        let mut elements = Vec::new();

        while !matches!(self.current_token(), Token::RightParen | Token::EOF) {
            // 检查是否为 dotted pair
            if matches!(self.current_token(), Token::Dot) {
                let dot_pos = self.current_position();
                self.advance();
                if elements.is_empty() {
                    return Err(SchemeError::SyntaxError(
                        "Unexpected dot at beginning of list".to_string(), dot_pos
                    ));
                }
                
                let tail = self.parse_expression()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In dotted pair tail: {}", msg), dot_pos),
                        other => other,
                    })?;
                
                if !matches!(self.current_token(), Token::RightParen) {
                    return Err(SchemeError::SyntaxError(
                        "Expected ')' after dot in dotted pair".to_string(), self.current_position()
                    ));
                }
                self.advance(); // 跳过 ')'
                
                // 构造 dotted pair
                let mut result = tail;
                for elem in elements.into_iter().rev() {
                    result = Value::Cons(Rc::new(elem), Rc::new(result));
                }
                return Ok(result);
            }
            
            let element_result = self.parse_expression()
                .map_err(|e| match e {
                    SchemeError::SyntaxError(msg, pos) => 
                        SchemeError::SyntaxError(format!("In list element: {}", msg), pos),
                    other => other,
                })?;
            elements.push(element_result);
        }

        if matches!(self.current_token(), Token::EOF) {
            return Err(SchemeError::SyntaxError("Unclosed list".to_string(), self.current_position()));
        }

        self.advance(); // 跳过 ')'
        Ok(Value::from_vec(elements))
    }

    /// 解析程序（多个表达式）
    pub fn parse_program(&mut self) -> Result<Vec<Value>> {
        let mut expressions = Vec::new();

        while !matches!(self.current_token(), Token::EOF) {
            expressions.push(self.parse_expression()?);
        }

        Ok(expressions)
    }

    /// 解析单个完整的表达式（便利方法）
    pub fn parse(input: &str) -> Result<Value> {
        let mut parser = Parser::new(input)?;
        parser.parse_expression()
    }

    /// 解析多个表达式（便利方法）
    pub fn parse_multiple(input: &str) -> Result<Vec<Value>> {
        let mut parser = Parser::new(input)?;
        parser.parse_program()
    }

    /// 解析单个表达式（返回带位置信息的值）
    pub fn parse_expression_located(&mut self) -> Result<LocatedValue> {
        let current_pos = self.current_position();
        
        match self.current_token().clone() {
            Token::EOF => Err(SchemeError::SyntaxError("Unexpected end of input".to_string(), current_pos)),
            
            Token::Integer(n) => {
                self.advance();
                Ok(LocatedValue::new(Value::Integer(n), current_pos))
            },
            
            Token::Float(f) => {
                self.advance();
                Ok(LocatedValue::new(Value::Float(f), current_pos))
            },
            
            Token::String(s) => {
                self.advance();
                Ok(LocatedValue::new(Value::String(s), current_pos))
            },
            
            Token::Symbol(s) => {
                self.advance();
                Ok(LocatedValue::new(Value::Symbol(s), current_pos))
            },
            
            Token::Boolean(b) => {
                self.advance();
                Ok(LocatedValue::new(Value::Bool(b), current_pos))
            },
            
            Token::Quote => {
                let quote_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression_located()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In quoted expression: {}", msg), quote_pos),
                        other => other,
                    })?;
                
                let quoted_value = Value::from_vec(vec![
                    Value::Symbol("quote".to_string()),
                    expr.value
                ]);
                Ok(LocatedValue::new(quoted_value, quote_pos))
            },
            
            Token::Quasiquote => {
                let quasiquote_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression_located()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In quasiquoted expression: {}", msg), quasiquote_pos),
                        other => other,
                    })?;
                
                let quasiquoted_value = Value::from_vec(vec![
                    Value::Symbol("quasiquote".to_string()),
                    expr.value
                ]);
                Ok(LocatedValue::new(quasiquoted_value, quasiquote_pos))
            },
            
            Token::Unquote => {
                let unquote_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression_located()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In unquoted expression: {}", msg), unquote_pos),
                        other => other,
                    })?;
                
                let unquoted_value = Value::from_vec(vec![
                    Value::Symbol("unquote".to_string()),
                    expr.value
                ]);
                Ok(LocatedValue::new(unquoted_value, unquote_pos))
            },
            
            Token::UnquoteSplicing => {
                let unquote_splicing_pos = self.current_position();
                self.advance();
                let expr = self.parse_expression_located()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In unquote-splicing expression: {}", msg), unquote_splicing_pos),
                        other => other,
                    })?;
                
                let unquote_splicing_value = Value::from_vec(vec![
                    Value::Symbol("unquote-splicing".to_string()),
                    expr.value
                ]);
                Ok(LocatedValue::new(unquote_splicing_value, unquote_splicing_pos))
            },
            
            Token::LeftParen => {
                let paren_pos = self.current_position();
                self.advance();
                let list_value = self.parse_list_located()?;
                Ok(LocatedValue::new(list_value, paren_pos))
            },
            
            _ => Err(SchemeError::SyntaxError(
                format!("Unexpected token: {}", self.current_token()), current_pos
            )),
        }
    }

    /// 解析列表（返回带位置信息的值）
    fn parse_list_located(&mut self) -> Result<Value> {
        let mut elements = Vec::new();

        while !matches!(self.current_token(), Token::RightParen | Token::EOF) {
            // 检查是否为 dotted pair
            if matches!(self.current_token(), Token::Dot) {
                let dot_pos = self.current_position();
                self.advance();
                if elements.is_empty() {
                    return Err(SchemeError::SyntaxError(
                        "Unexpected dot at beginning of list".to_string(), dot_pos
                    ));
                }
                
                let tail = self.parse_expression_located()
                    .map_err(|e| match e {
                        SchemeError::SyntaxError(msg, _) => 
                            SchemeError::SyntaxError(format!("In dotted pair tail: {}", msg), dot_pos),
                        other => other,
                    })?;
                
                if !matches!(self.current_token(), Token::RightParen) {
                    return Err(SchemeError::SyntaxError(
                        "Expected ')' after dot in dotted pair".to_string(), self.current_position()
                    ));
                }
                self.advance(); // 跳过 ')'
                
                // 构造 dotted pair
                let mut result = tail.value;
                for elem in elements.into_iter().rev() {
                    result = Value::Cons(Rc::new(elem), Rc::new(result));
                }
                return Ok(result);
            }
            
            let element_result = self.parse_expression_located()
                .map_err(|e| match e {
                    SchemeError::SyntaxError(msg, pos) => 
                        SchemeError::SyntaxError(format!("In list element: {}", msg), pos),
                    other => other,
                })?;
            elements.push(element_result.value);
        }

        if matches!(self.current_token(), Token::EOF) {
            return Err(SchemeError::SyntaxError("Unclosed list".to_string(), self.current_position()));
        }

        self.advance(); // 跳过 ')'
        Ok(Value::from_vec(elements))
    }

    /// 解析程序（多个表达式，返回带位置信息的值）
    pub fn parse_program_located(&mut self) -> Result<Vec<LocatedValue>> {
        let mut expressions = Vec::new();

        while !matches!(self.current_token(), Token::EOF) {
            expressions.push(self.parse_expression_located()?);
        }

        Ok(expressions)
    }

    /// 解析单个完整的表达式（便利方法，返回带位置信息的值）
    pub fn parse_located(input: &str) -> Result<LocatedValue> {
        let mut parser = Parser::new(input)?;
        parser.parse_expression_located()
    }

    /// 解析多个表达式（便利方法，返回带位置信息的值）
    pub fn parse_multiple_located(input: &str) -> Result<Vec<LocatedValue>> {
        let mut parser = Parser::new(input)?;
        parser.parse_program_located()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atom() {
        assert_eq!(Parser::parse("42").unwrap(), Value::Integer(42));
        assert_eq!(Parser::parse("3.14").unwrap(), Value::Float(3.14));
        assert_eq!(Parser::parse("\"hello\"").unwrap(), Value::String("hello".to_string()));
        assert_eq!(Parser::parse("foo").unwrap(), Value::Symbol("foo".to_string()));
        assert_eq!(Parser::parse("#t").unwrap(), Value::Bool(true));
        assert_eq!(Parser::parse("#f").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_parse_list() {
        let result = Parser::parse("(+ 1 2)").unwrap();
        let expected = Value::from_vec(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(1),
            Value::Integer(2),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_nested_list() {
        let result = Parser::parse("(+ (* 2 3) 4)").unwrap();
        let inner_list = Value::from_vec(vec![
            Value::Symbol("*".to_string()),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let expected = Value::from_vec(vec![
            Value::Symbol("+".to_string()),
            inner_list,
            Value::Integer(4),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_quote() {
        let result = Parser::parse("'foo").unwrap();
        let expected = Value::from_vec(vec![
            Value::Symbol("quote".to_string()),
            Value::Symbol("foo".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_empty_list() {
        let result = Parser::parse("()").unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_parse_multiple() {
        let results = Parser::parse_multiple("1 2 (+ 3 4)").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Value::Integer(1));
        assert_eq!(results[1], Value::Integer(2));
        
        let expected_list = Value::from_vec(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        assert_eq!(results[2], expected_list);
    }

    #[test]
    fn test_position_info_in_errors() {
        // 测试未闭合列表的位置信息
        let result = Parser::parse("(+ 1 2");
        assert!(result.is_err());
        if let Err(SchemeError::SyntaxError(msg, pos)) = result {
            assert!(msg.contains("Unclosed list"));
            assert!(pos.is_some());
        } else {
            panic!("Expected SyntaxError with position");
        }

        // 测试意外的点号
        let result = Parser::parse("(.)");
        assert!(result.is_err());
        if let Err(SchemeError::SyntaxError(msg, pos)) = result {
            assert!(msg.contains("Unexpected dot"));
            assert!(pos.is_some());
            let pos = pos.unwrap();
            assert_eq!(pos.line, 1);
            assert_eq!(pos.column, 2); // 点号在第2列
        } else {
            panic!("Expected SyntaxError with position");
        }

        // 测试 dotted pair 后缺少右括号
        let result = Parser::parse("(a . b");
        assert!(result.is_err());
        if let Err(SchemeError::SyntaxError(msg, pos)) = result {
            assert!(msg.contains("Expected ')'"));
            assert!(pos.is_some());
        } else {
            panic!("Expected SyntaxError with position");
        }

        // 测试引用表达式中的错误
        let result = Parser::parse("'(");
        assert!(result.is_err());
        if let Err(SchemeError::SyntaxError(msg, pos)) = result {
            assert!(msg.contains("In quoted expression"));
            assert!(pos.is_some());
            let pos = pos.unwrap();
            assert_eq!(pos.line, 1);
            assert_eq!(pos.column, 1); // 引用符号在第1列
        } else {
            panic!("Expected SyntaxError with position");
        }
    }

    #[test]
    fn test_multiline_position_info() {
        let input = "1\n(\n+ 2";
        let result = Parser::parse_multiple(input);
        assert!(result.is_err());
        if let Err(SchemeError::SyntaxError(msg, pos)) = result {
            assert!(msg.contains("Unclosed list"));
            assert!(pos.is_some());
            let pos = pos.unwrap();
            assert_eq!(pos.line, 3); // 错误在第3行
        } else {
            panic!("Expected SyntaxError with position");
        }
    }

    #[test]
    fn test_position_info_propagation() {
        // 测试嵌套错误的位置信息传播
        let result = Parser::parse("(list 'broken");
        assert!(result.is_err());
        if let Err(SchemeError::SyntaxError(msg, pos)) = result {
            // 应该包含上下文信息
            assert!(msg.contains("In quoted expression") || msg.contains("In list element") || msg.contains("Unclosed list"));
            assert!(pos.is_some());
        } else {
            panic!("Expected SyntaxError with position");
        }
    }

    #[test]
    fn test_located_value_parsing() {
        // 测试基本原子值的位置信息
        let result = Parser::parse_located("42").unwrap();
        assert_eq!(result.value(), &Value::Integer(42));
        assert!(result.position().is_some());
        let pos = result.position().unwrap();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);

        // 测试符号的位置信息
        let result = Parser::parse_located("foo").unwrap();
        assert_eq!(result.value(), &Value::Symbol("foo".to_string()));
        assert!(result.position().is_some());

        // 测试字符串的位置信息
        let result = Parser::parse_located("\"hello\"").unwrap();
        assert_eq!(result.value(), &Value::String("hello".to_string()));
        assert!(result.position().is_some());
    }

    #[test]
    fn test_located_list_parsing() {
        let result = Parser::parse_located("(+ 1 2)").unwrap();
        let expected = Value::from_vec(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(1),
            Value::Integer(2),
        ]);
        assert_eq!(result.value(), &expected);
        assert!(result.position().is_some());
        let pos = result.position().unwrap();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1); // '(' 在第1列
    }

    #[test]
    fn test_located_quote_parsing() {
        let result = Parser::parse_located("'foo").unwrap();
        let expected = Value::from_vec(vec![
            Value::Symbol("quote".to_string()),
            Value::Symbol("foo".to_string()),
        ]);
        assert_eq!(result.value(), &expected);
        assert!(result.position().is_some());
        let pos = result.position().unwrap();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1); // '\'' 在第1列
    }

    #[test]
    fn test_located_multiple_parsing() {
        let results = Parser::parse_multiple_located("1 2 (+ 3 4)").unwrap();
        assert_eq!(results.len(), 3);
        
        // 检查第一个值
        assert_eq!(results[0].value(), &Value::Integer(1));
        assert!(results[0].position().is_some());
        
        // 检查第二个值
        assert_eq!(results[1].value(), &Value::Integer(2));
        assert!(results[1].position().is_some());
        let pos2 = results[1].position().unwrap();
        assert_eq!(pos2.column, 3); // '2' 在第3列
        
        // 检查第三个值（列表）
        let expected_list = Value::from_vec(vec![
            Value::Symbol("+".to_string()),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        assert_eq!(results[2].value(), &expected_list);
        assert!(results[2].position().is_some());
        let pos3 = results[2].position().unwrap();
        assert_eq!(pos3.column, 5); // '(' 在第5列
    }

    #[test]
    fn test_located_multiline_parsing() {
        let input = "1\n(+ 2 3)\nfoo";
        let results = Parser::parse_multiple_located(input).unwrap();
        assert_eq!(results.len(), 3);
        
        // 第一行的数字
        let pos1 = results[0].position().unwrap();
        assert_eq!(pos1.line, 1);
        assert_eq!(pos1.column, 1);
        
        // 第二行的列表
        let pos2 = results[1].position().unwrap();
        assert_eq!(pos2.line, 2);
        assert_eq!(pos2.column, 1);
        
        // 第三行的符号
        let pos3 = results[2].position().unwrap();
        assert_eq!(pos3.line, 3);
        assert_eq!(pos3.column, 1);
    }
}
