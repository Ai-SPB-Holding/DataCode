// Парсер выражений для DataCode
// Реализует рекурсивный спуск для парсинга выражений с учетом приоритета операторов

use super::tokens::{Token, Expr, BinaryOp, UnaryOp};
use super::Parser;
use crate::value::Value;
use crate::error::{DataCodeError, Result};

/// Парсер выражений
/// Использует алгоритм рекурсивного спуска с учетом приоритета операторов
pub struct ExpressionParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> ExpressionParser<'a> {
    /// Создать новый парсер выражений
    pub fn new(parser: &'a mut Parser) -> Self {
        Self { parser }
    }
    
    /// Парсить выражение (точка входа)
    pub fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }
    
    /// Парсить логическое ИЛИ (самый низкий приоритет)
    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        
        while matches!(self.parser.current_token(), Token::Or) {
            self.parser.advance();
            let right = self.parse_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить логическое И
    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        
        while matches!(self.parser.current_token(), Token::And) {
            self.parser.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить операторы равенства
    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        
        while matches!(self.parser.current_token(), Token::Equal | Token::NotEqual) {
            let op = match self.parser.current_token() {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            self.parser.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить операторы сравнения
    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_addition()?;
        
        while matches!(self.parser.current_token(), 
            Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual) {
            let op = match self.parser.current_token() {
                Token::Less => BinaryOp::Less,
                Token::Greater => BinaryOp::Greater,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            self.parser.advance();
            let right = self.parse_addition()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить сложение и вычитание
    fn parse_addition(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplication()?;
        
        while matches!(self.parser.current_token(), Token::Plus | Token::Minus) {
            let op = match self.parser.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            self.parser.advance();
            let right = self.parse_multiplication()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить умножение, деление и остаток от деления
    fn parse_multiplication(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        while matches!(self.parser.current_token(), Token::Multiply | Token::Divide | Token::Modulo) {
            let op = match self.parser.current_token() {
                Token::Multiply => BinaryOp::Multiply,
                Token::Divide => BinaryOp::Divide,
                Token::Modulo => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            self.parser.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }
    
    /// Парсить унарные операторы
    fn parse_unary(&mut self) -> Result<Expr> {
        match self.parser.current_token() {
            Token::Not => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            Token::Minus => {
                self.parser.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Minus,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_postfix(),
        }
    }
    
    /// Парсить постфиксные операторы (вызовы функций, индексация)
    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        
        loop {
            match self.parser.current_token() {
                Token::LeftParen => {
                    // Вызов функции
                    if let Expr::Variable(name) = expr {
                        self.parser.advance(); // consume '('
                        let args = self.parse_function_args()?;
                        self.parser.expect(Token::RightParen)?;
                        expr = Expr::FunctionCall { name, args };
                    } else {
                        break;
                    }
                }
                Token::LeftBracket => {
                    // Индексация
                    self.parser.advance(); // consume '['
                    let index = self.parse_expression()?;
                    self.parser.expect(Token::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Dot => {
                    // Доступ к члену
                    self.parser.advance(); // consume '.'
                    if let Token::Identifier(member) = self.parser.current_token() {
                        let member = member.clone();
                        self.parser.advance();
                        expr = Expr::Member {
                            object: Box::new(expr),
                            member,
                        };
                    } else {
                        return Err(DataCodeError::syntax_error(
                            "Expected identifier after '.'",
                            1, 0
                        ));
                    }
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    /// Парсить аргументы функции
    fn parse_function_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        
        if !matches!(self.parser.current_token(), Token::RightParen) {
            args.push(self.parse_expression()?);
            
            while matches!(self.parser.current_token(), Token::Comma) {
                self.parser.advance(); // consume ','
                args.push(self.parse_expression()?);
            }
        }
        
        Ok(args)
    }

    /// Парсить первичные выражения (литералы, переменные, скобки)
    fn parse_primary(&mut self) -> Result<Expr> {
        match self.parser.current_token() {
            Token::String(s) => {
                let value = s.clone();
                self.parser.advance();
                Ok(Expr::Literal(Value::String(value)))
            }
            Token::Number(n) => {
                let value = *n;
                self.parser.advance();
                Ok(Expr::Literal(Value::Number(value)))
            }
            Token::Bool(b) => {
                let value = *b;
                self.parser.advance();
                Ok(Expr::Literal(Value::Bool(value)))
            }
            Token::Null => {
                self.parser.advance();
                Ok(Expr::Literal(Value::Null))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.parser.advance();
                Ok(Expr::Variable(name))
            }
            Token::LeftParen => {
                self.parser.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.parser.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                // Массив
                self.parser.advance(); // consume '['
                let mut elements = Vec::new();

                if !matches!(self.parser.current_token(), Token::RightBracket) {
                    elements.push(self.parse_expression()?);

                    while matches!(self.parser.current_token(), Token::Comma) {
                        self.parser.advance(); // consume ','
                        elements.push(self.parse_expression()?);
                    }
                }

                self.parser.expect(Token::RightBracket)?;
                Ok(Expr::ArrayLiteral { elements })
            }
            Token::LeftBrace => {
                // Объект
                self.parser.advance(); // consume '{'
                let mut pairs = Vec::new();

                if !matches!(self.parser.current_token(), Token::RightBrace) {
                    // Парсим первую пару ключ: значение
                    let key = match self.parser.current_token() {
                        Token::Identifier(name) => {
                            let key = name.clone();
                            self.parser.advance();
                            key
                        }
                        Token::String(s) => {
                            let key = s.clone();
                            self.parser.advance();
                            key
                        }
                        _ => return Err(DataCodeError::syntax_error(
                            "Expected identifier or string as object key",
                            1, 0
                        )),
                    };

                    self.parser.expect(Token::Colon)?;
                    let value = self.parse_expression()?;
                    pairs.push((key, value));

                    // Парсим остальные пары
                    while matches!(self.parser.current_token(), Token::Comma) {
                        self.parser.advance(); // consume ','

                        // Проверяем, не закрывающая ли это скобка (trailing comma)
                        if matches!(self.parser.current_token(), Token::RightBrace) {
                            break;
                        }

                        let key = match self.parser.current_token() {
                            Token::Identifier(name) => {
                                let key = name.clone();
                                self.parser.advance();
                                key
                            }
                            Token::String(s) => {
                                let key = s.clone();
                                self.parser.advance();
                                key
                            }
                            _ => return Err(DataCodeError::syntax_error(
                                "Expected identifier or string as object key",
                                1, 0
                            )),
                        };

                        self.parser.expect(Token::Colon)?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));
                    }
                }

                self.parser.expect(Token::RightBrace)?;
                Ok(Expr::ObjectLiteral { pairs })
            }
            _ => Err(DataCodeError::syntax_error(
                &format!("Unexpected token: {:?}", self.parser.current_token()),
                1, 0
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let mut parser = Parser::new("42");
        let mut expr_parser = ExpressionParser::new(&mut parser);
        let expr = expr_parser.parse_expression().unwrap();

        match expr {
            Expr::Literal(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let mut parser = Parser::new("2 + 3");
        let mut expr_parser = ExpressionParser::new(&mut parser);
        let expr = expr_parser.parse_expression().unwrap();

        match expr {
            Expr::Binary { operator, .. } => {
                assert!(matches!(operator, BinaryOp::Add));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_precedence() {
        let mut parser = Parser::new("2 + 3 * 4");
        let mut expr_parser = ExpressionParser::new(&mut parser);
        let expr = expr_parser.parse_expression().unwrap();

        // Должно быть: 2 + (3 * 4)
        match expr {
            Expr::Binary { left, operator: BinaryOp::Add, right } => {
                assert!(matches!(left.as_ref(), Expr::Literal(Value::Number(2.0))));
                assert!(matches!(right.as_ref(), Expr::Binary { operator: BinaryOp::Multiply, .. }));
            }
            _ => panic!("Expected addition with multiplication on right"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let mut parser = Parser::new("func(1, 2)");
        let mut expr_parser = ExpressionParser::new(&mut parser);
        let expr = expr_parser.parse_expression().unwrap();

        match expr {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "func");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_array_literal() {
        let mut parser = Parser::new("[1, 2, 3]");
        let mut expr_parser = ExpressionParser::new(&mut parser);
        let expr = expr_parser.parse_expression().unwrap();

        match expr {
            Expr::ArrayLiteral { elements } => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected array literal"),
        }
    }
}
