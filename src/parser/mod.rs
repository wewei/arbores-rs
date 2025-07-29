use std::rc::Rc;
use crate::lexer::{Lexer, Token, LocatedToken};
use crate::types::{Value, SchemeError, Result, Position};

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
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Value::from_vec(vec![
                    Value::Symbol("quote".to_string()),
                    expr
                ]))
            },
            
            Token::Quasiquote => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Value::from_vec(vec![
                    Value::Symbol("quasiquote".to_string()),
                    expr
                ]))
            },
            
            Token::Unquote => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Value::from_vec(vec![
                    Value::Symbol("unquote".to_string()),
                    expr
                ]))
            },
            
            Token::UnquoteSplicing => {
                self.advance();
                let expr = self.parse_expression()?;
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
                self.advance();
                if elements.is_empty() {
                    return Err(SchemeError::SyntaxError(
                        "Unexpected dot at beginning of list".to_string(), self.current_position()
                    ));
                }
                
                let tail = self.parse_expression()?;
                
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
            
            elements.push(self.parse_expression()?);
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
}
