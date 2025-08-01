//! 简化的递归下降解析器实现
//! 
//! 本模块实现了基于 Scheme 语法特性的简化递归下降解析器。
//! 由于 Scheme 语法简单且无歧义，不需要复杂的规则系统。

use std::rc::Rc;

use crate::interpreter::lexer::types::{LexError, Token, TokenType, Span};
use super::types::{
    SExpr, SExprContent, Value, ParseError, ParseOutput,
    UnexpectedTokenReason, DottedListError, QuoteType
};
use super::utils::{SourceBuilder, token_to_value, is_atom_token, build_list, build_dotted_list, validate_dotted_list_structure};

// ============================================================================
// 简化的递归下降解析器
// ============================================================================

/// 简化的解析器 - 直接递归下降，无需规则抽象
/// Scheme 语法足够简单，不需要复杂的规则匹配系统
pub struct SimpleParser<I> 
where 
    I: Iterator<Item = Result<Token, LexError>>
{
    /// Token 流（支持前瞻）
    tokens: std::iter::Peekable<I>,
    /// 源代码重建器
    source_builder: SourceBuilder,
    /// 当前位置（用于错误报告）
    position: usize,
}

impl<I> SimpleParser<I> 
where 
    I: Iterator<Item = Result<Token, LexError>>
{
    /// 创建新的解析器
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            source_builder: SourceBuilder::new(),
            position: 0,
        }
    }
    
    /// 解析所有顶层表达式 - 主要入口点
    pub fn parse_all(&mut self) -> ParseOutput {
        let mut expressions = Vec::new();
        
        // 跳过开头的空白和注释
        self.skip_trivia();
        
        while !self.is_at_end() {
            match self.parse_expression() {
                Ok(expr) => {
                    expressions.push(expr);
                    self.skip_trivia(); // 跳过表达式之间的空白
                },
                Err(error) => return ParseOutput::error(error, self.source_builder.clone().build()),
            }
        }
        
        ParseOutput::success(expressions, self.source_builder.clone().build())
    }
    
    /// 解析单个表达式 - 核心分发逻辑
    fn parse_expression(&mut self) -> Result<SExpr, ParseError> {
        match self.peek_token_type()? {
            TokenType::LeftParen => self.parse_list(),
            TokenType::Quote => self.parse_quoted(QuoteType::Quote),
            TokenType::Quasiquote => self.parse_quoted(QuoteType::Quasiquote),
            TokenType::Unquote => self.parse_quoted(QuoteType::Unquote),
            TokenType::UnquoteSplicing => self.parse_quoted(QuoteType::UnquoteSplicing),
            token_type if is_atom_token(&token_type) => self.parse_atom(),
            // 暂时不支持向量解析，但预留接口
            // TokenType::LeftBracket => self.parse_vector(),
            _ => {
                let token = self.consume_token()?;
                Err(ParseError::unexpected_token(
                    token,
                    UnexpectedTokenReason::Expected("expression".to_string())
                ))
            }
        }
    }
    
    /// 解析列表 - 处理 (expr1 expr2 ...) 和点对列表 (expr1 . expr2)
    fn parse_list(&mut self) -> Result<SExpr, ParseError> {
        let start_token = self.consume_token()?; // 消费 '('
        let start_span = start_token.span;
        let mut elements = Vec::new();
        
        loop {
            self.skip_trivia();
            
            match self.peek_token_type()? {
                TokenType::RightParen => {
                    let end_token = self.consume_token()?; // 消费 ')'
                    let full_span = Span::new(start_span.start, end_token.span.end);
                    
                    let content = build_list(elements);
                    return Ok(SExpr::with_span(content, full_span));
                },
                TokenType::Dot => {
                    return self.parse_dotted_list(elements, start_span);
                },
                _ => {
                    let expr = self.parse_expression()?;
                    elements.push(expr);
                }
            }
        }
    }
    
    /// 解析点对列表的尾部
    fn parse_dotted_list(&mut self, elements: Vec<SExpr>, start_span: Span) -> Result<SExpr, ParseError> {
        let dot_token = self.consume_token()?; // 消费 '.'
        
        // 验证点对列表结构的合法性
        validate_dotted_list_structure(&elements, elements.len())
            .map_err(|err| ParseError::invalid_dotted_list(dot_token.clone(), err))?;
        
        self.skip_trivia();
        
        // 解析尾部表达式
        let tail = self.parse_expression()?;
        
        self.skip_trivia();
        
        // 验证后面是 ')'
        match self.peek_token_type()? {
            TokenType::RightParen => {
                let end_token = self.consume_token()?;
                let full_span = Span::new(start_span.start, end_token.span.end);
                
                let content = build_dotted_list(elements, tail);
                Ok(SExpr::with_span(content, full_span))
            },
            _ => {
                let _token = self.consume_token()?;
                Err(ParseError::invalid_dotted_list(
                    dot_token,
                    DottedListError::MultipleTailElements
                ))
            }
        }
    }
    
    /// 解析引用表达式 - 将 'expr 转换为 (quote expr)
    fn parse_quoted(&mut self, quote_type: QuoteType) -> Result<SExpr, ParseError> {
        let quote_token = self.consume_token()?; // 消费引用符号
        let quote_span = quote_token.span;
        
        // 解析被引用的表达式
        let expr = self.parse_expression()?;
        let expr_span = *expr.span.as_ref();
        
        // 创建引用符号
        let quote_symbol = SExpr::with_span(
            SExprContent::Atom(Value::Symbol(quote_type.symbol_name().to_string())),
            quote_span
        );
        
        // 创建尾部 nil
        let nil_span = Span::empty(expr_span.end);
        let nil_node = SExpr::with_span(SExprContent::Nil, nil_span);
        
        // 构建内层 cons：(expr . ())
        let inner_cons_span = expr_span;
        let inner_cons = SExpr::with_span(
            SExprContent::Cons {
                car: Rc::new(expr),
                cdr: Rc::new(nil_node),
            },
            inner_cons_span
        );
        
        // 构建外层 cons：(quote . (expr . ()))
        let full_span = Span::new(quote_span.start, expr_span.end);
        Ok(SExpr::with_span(
            SExprContent::Cons {
                car: Rc::new(quote_symbol),
                cdr: Rc::new(inner_cons),
            },
            full_span
        ))
    }
    
    /// 解析原子值
    fn parse_atom(&mut self) -> Result<SExpr, ParseError> {
        let token = self.consume_token()?;
        let span = token.span;
        let token_type = token.token_type.clone();
        
        let value = token_to_value(token_type)
            .ok_or_else(|| {
                ParseError::unexpected_token(
                    token.clone(),
                    UnexpectedTokenReason::Expected("atomic value".to_string())
                )
            })?;
        
        Ok(SExpr::with_span(SExprContent::Atom(value), span))
    }
    
    // 暂时不实现向量解析，等lexer支持后再补充
    // fn parse_vector(&mut self) -> Result<SExpr, ParseError> {
    //     todo!("Vector parsing will be implemented when lexer supports vectors")
    // }
    
    /// Token 流操作辅助方法
    fn peek_token_type(&mut self) -> Result<TokenType, ParseError> {
        match self.tokens.peek() {
            Some(Ok(token)) => Ok(token.token_type.clone()),
            Some(Err(e)) => Err(ParseError::LexError(e.clone())),
            None => Err(ParseError::unexpected_eof()),
        }
    }
    
    fn consume_token(&mut self) -> Result<Token, ParseError> {
        match self.tokens.next() {
            Some(Ok(token)) => {
                self.position += 1;
                self.source_builder.add_token(&token);
                Ok(token)
            },
            Some(Err(e)) => Err(ParseError::LexError(e)),
            None => Err(ParseError::unexpected_eof()),
        }
    }
    
    fn is_at_end(&mut self) -> bool {
        matches!(self.tokens.peek(), 
            None | 
            Some(Ok(Token { token_type: TokenType::Eof, .. }))
        )
    }
    
    /// 跳过空白字符、换行符和注释
    fn skip_trivia(&mut self) {
        while let Some(Ok(token)) = self.tokens.peek() {
            if token.token_type.is_trivia() {
                let token = self.tokens.next().unwrap().unwrap();
                self.source_builder.add_token(&token);
                self.position += 1;
            } else {
                break;
            }
        }
    }
}

// ============================================================================
// 公共API函数
// ============================================================================

/// 解析器主函数 - 对外暴露的唯一入口
/// 
/// 从 token 流解析 S 表达式并重建源代码内容
pub fn parse(tokens: impl Iterator<Item = Result<Token, LexError>>) -> ParseOutput {
    let mut parser = SimpleParser::new(tokens);
    parser.parse_all()
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::lexer::types::Span;

    fn create_test_token(token_type: TokenType, start: usize, text: &str) -> Result<Token, LexError> {
        let span = Span::new(start, start + text.len());
        Ok(Token::new(token_type, span, text.to_string()))
    }

    #[test]
    fn test_parse_simple_atom() {
        let tokens = vec![
            create_test_token(TokenType::Number(42.0), 0, "42"),
        ];
        
        let result = parse(tokens.into_iter());
        
        match result.result {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                match &exprs[0].content {
                    SExprContent::Atom(Value::Number(n)) => assert_eq!(*n, 42.0),
                    _ => panic!("Expected number atom"),
                }
            },
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_parse_simple_list() {
        let tokens = vec![
            create_test_token(TokenType::LeftParen, 0, "("),
            create_test_token(TokenType::Symbol("define".to_string()), 1, "define"),
            create_test_token(TokenType::Symbol("x".to_string()), 8, "x"),
            create_test_token(TokenType::Number(42.0), 10, "42"),
            create_test_token(TokenType::RightParen, 12, ")"),
        ];
        
        let result = parse(tokens.into_iter());
        
        match result.result {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                match &exprs[0].content {
                    SExprContent::Cons { car, cdr: _ } => {
                        // 验证第一个元素是 'define' 符号
                        match &car.content {
                            SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "define"),
                            _ => panic!("Expected define symbol"),
                        }
                    },
                    _ => panic!("Expected list"),
                }
            },
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_parse_quoted_expression() {
        let tokens = vec![
            create_test_token(TokenType::Quote, 0, "'"),
            create_test_token(TokenType::Symbol("x".to_string()), 1, "x"),
        ];
        
        let result = parse(tokens.into_iter());
        
        match result.result {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                match &exprs[0].content {
                    SExprContent::Cons { car, cdr } => {
                        // 验证第一个元素是 'quote' 符号
                        match &car.content {
                            SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "quote"),
                            _ => panic!("Expected quote symbol"),
                        }
                        // 验证第二个元素是被引用的表达式
                        match &cdr.content {
                            SExprContent::Cons { car: quoted_expr, .. } => {
                                match &quoted_expr.content {
                                    SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "x"),
                                    _ => panic!("Expected quoted symbol"),
                                }
                            },
                            _ => panic!("Expected quoted expression"),
                        }
                    },
                    _ => panic!("Expected quoted list"),
                }
            },
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_parse_dotted_list() {
        let tokens = vec![
            create_test_token(TokenType::LeftParen, 0, "("),
            create_test_token(TokenType::Symbol("a".to_string()), 1, "a"),
            create_test_token(TokenType::Dot, 3, "."),
            create_test_token(TokenType::Symbol("b".to_string()), 5, "b"),
            create_test_token(TokenType::RightParen, 6, ")"),
        ];
        
        let result = parse(tokens.into_iter());
        
        match result.result {
            Ok(exprs) => {
                assert_eq!(exprs.len(), 1);
                match &exprs[0].content {
                    SExprContent::Cons { car, cdr } => {
                        // 验证 car 是 'a'
                        match &car.content {
                            SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "a"),
                            _ => panic!("Expected symbol 'a'"),
                        }
                        // 验证 cdr 是 'b'（不是列表）
                        match &cdr.content {
                            SExprContent::Atom(Value::Symbol(s)) => assert_eq!(s, "b"),
                            _ => panic!("Expected symbol 'b' as tail"),
                        }
                    },
                    _ => panic!("Expected dotted pair"),
                }
            },
            Err(e) => panic!("Parse failed: {}", e),
        }
    }
}
