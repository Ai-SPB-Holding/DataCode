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
            self.parser.skip_newlines(); // skip newlines after operator
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
            self.parser.skip_newlines(); // skip newlines after operator
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
            self.parser.skip_newlines(); // skip newlines after operator
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
                        let (args, named_args) = self.parse_function_args()?;
                        self.parser.expect(Token::RightParen)?;
                        expr = Expr::FunctionCall { name, args, named_args };
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
    
    /// Парсить аргументы функции (возвращает позиционные и именованные аргументы)
    fn parse_function_args(&mut self) -> Result<(Vec<Expr>, Vec<(String, Expr)>)> {
        let mut args = Vec::new();
        let mut named_args = Vec::new();

        self.parser.skip_newlines(); // skip newlines after '('

        if !matches!(self.parser.current_token(), Token::RightParen) {
            match self.parse_function_arg()? {
                Expr::NamedArg { name, value } => {
                    named_args.push((name, *value));
                }
                arg => {
                    args.push(arg);
                }
            }
            self.parser.skip_newlines(); // skip newlines after first argument

            while matches!(self.parser.current_token(), Token::Comma) {
                self.parser.advance(); // consume ','
                self.parser.skip_newlines(); // skip newlines after ','

                // Check for trailing comma (comma followed by closing paren)
                if matches!(self.parser.current_token(), Token::RightParen) {
                    break;
                }

                match self.parse_function_arg()? {
                    Expr::NamedArg { name, value } => {
                        named_args.push((name, *value));
                    }
                    arg => {
                        // После первого именованного аргумента все последующие должны быть именованными
                        if !named_args.is_empty() {
                            return Err(DataCodeError::syntax_error(
                                "Positional arguments cannot follow named arguments",
                                1, 0
                            ));
                        }
                        args.push(arg);
                    }
                }
                self.parser.skip_newlines(); // skip newlines after argument
            }
        }

        Ok((args, named_args))
    }

    /// Парсить один аргумент функции (может быть обычным выражением, spread или именованным аргументом)
    fn parse_function_arg(&mut self) -> Result<Expr> {
        if matches!(self.parser.current_token(), Token::Multiply) {
            // Spread operator
            self.parser.advance(); // consume '*'
            let expression = self.parse_expression_until_comma()?;
            Ok(Expr::Spread {
                expression: Box::new(expression),
            })
        } else {
            // Парсим выражение, но проверяем, является ли оно именованным аргументом
            // Именованный аргумент имеет форму: identifier = expression
            // Проверяем это в parse_primary_until_comma_with_named_arg
            self.parse_expression_until_comma_with_named_arg()
        }
    }
    
    /// Парсить выражение до запятой или закрывающей скобки с поддержкой именованных аргументов
    fn parse_expression_until_comma_with_named_arg(&mut self) -> Result<Expr> {
        // Проверяем, является ли это именованным аргументом (identifier = expression)
        if let Token::Identifier(name) = self.parser.current_token() {
            let name = name.clone();
            self.parser.advance(); // consume identifier
            self.parser.skip_newlines();
            
            // Проверяем, есть ли после идентификатора оператор присваивания
            if matches!(self.parser.current_token(), Token::Assign) {
                // Это именованный аргумент
                self.parser.advance(); // consume '='
                self.parser.skip_newlines();
                let value = self.parse_expression_until_comma()?;
                return Ok(Expr::NamedArg {
                    name,
                    value: Box::new(value),
                });
            } else {
                // Это не именованный аргумент, парсим как обычное выражение
                // Но мы уже продвинулись, поэтому нужно парсить оставшуюся часть
                // Начинаем с переменной и продолжаем парсинг
                let mut expr = Expr::Variable(name);
                
                // Продолжаем парсинг постфиксных операторов и бинарных выражений
                // Используем parse_postfix_until_comma, но начинаем с уже созданной переменной
                // Для этого нужно парсить с текущей позиции
                return self.parse_expression_continuation_until_comma(expr);
            }
        }
        
        // Для всех остальных случаев парсим как обычное выражение
        self.parse_expression_until_comma()
    }
    
    /// Продолжить парсинг выражения, начиная с уже созданного выражения
    /// Используется когда мы уже распарсили часть выражения (например, переменную)
    fn parse_expression_continuation_until_comma(&mut self, mut expr: Expr) -> Result<Expr> {
        // Парсим постфиксные операторы (вызовы функций, индексация, доступ к членам)
        loop {
            match self.parser.current_token() {
                Token::LeftParen => {
                    if let Expr::Variable(name) = expr {
                        self.parser.advance(); // consume '('
                        let (args, named_args) = self.parse_function_args()?;
                        self.parser.expect(Token::RightParen)?;
                        expr = Expr::FunctionCall { name, args, named_args };
                    } else {
                        break;
                    }
                }
                Token::LeftBracket => {
                    self.parser.advance(); // consume '['
                    let index = self.parse_expression_until_comma()?;
                    self.parser.expect(Token::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Dot => {
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
            
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                break;
            }
        }
        
        // Теперь парсим бинарные операторы, начиная с текущего выражения
        // Парсим умножение/деление
        while matches!(self.parser.current_token(), Token::Multiply | Token::Divide | Token::Modulo) {
            let op = match self.parser.current_token() {
                Token::Multiply => BinaryOp::Multiply,
                Token::Divide => BinaryOp::Divide,
                Token::Modulo => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            self.parser.advance();
            self.parser.skip_newlines();
            let right = self.parse_unary_until_comma()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                return Ok(expr);
            }
        }
        
        // Парсим сложение/вычитание
        while matches!(self.parser.current_token(), Token::Plus | Token::Minus) {
            let op = match self.parser.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            self.parser.advance();
            self.parser.skip_newlines();
            let right = self.parse_multiplication_until_comma()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                return Ok(expr);
            }
        }
        
        Ok(expr)
    }
    
    /// Парсить выражение до запятой или закрывающей скобки
    /// Это нужно для правильного парсинга аргументов функций
    fn parse_expression_until_comma(&mut self) -> Result<Expr> {
        self.parse_or_until_comma()
    }
    
    /// Парсить логическое ИЛИ до запятой или закрывающей скобки
    fn parse_or_until_comma(&mut self) -> Result<Expr> {
        let mut left = self.parse_and_until_comma()?;
        
        while matches!(self.parser.current_token(), Token::Or) {
            self.parser.advance();
            // Проверяем, не является ли следующий токен запятой или закрывающей скобкой
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                // Откатываемся назад, так как мы уже продвинулись
                return Ok(left);
            }
            let right = self.parse_and_until_comma()?;
            // Проверяем после парсинга правой части
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                left = Expr::Binary {
                    left: Box::new(left),
                    operator: BinaryOp::Or,
                    right: Box::new(right),
                };
                return Ok(left);
            }
            left = Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить логическое И до запятой или закрывающей скобки
    fn parse_and_until_comma(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality_until_comma()?;
        
        while matches!(self.parser.current_token(), Token::And) {
            self.parser.advance();
            // Проверяем, не является ли следующий токен запятой или закрывающей скобкой
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                return Ok(left);
            }
            let right = self.parse_equality_until_comma()?;
            // Проверяем после парсинга правой части
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                left = Expr::Binary {
                    left: Box::new(left),
                    operator: BinaryOp::And,
                    right: Box::new(right),
                };
                return Ok(left);
            }
            left = Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить операторы равенства до запятой или закрывающей скобки
    fn parse_equality_until_comma(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison_until_comma()?;
        
        while matches!(self.parser.current_token(), Token::Equal | Token::NotEqual) {
            let op = match self.parser.current_token() {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            self.parser.advance();
            // Проверяем, не является ли следующий токен запятой или закрывающей скобкой
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                return Ok(left);
            }
            let right = self.parse_comparison_until_comma()?;
            // Проверяем после парсинга правой части
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                left = Expr::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
                return Ok(left);
            }
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить операторы сравнения до запятой или закрывающей скобки
    fn parse_comparison_until_comma(&mut self) -> Result<Expr> {
        let mut left = self.parse_addition_until_comma()?;
        
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
            self.parser.skip_newlines();
            // Проверяем, не является ли следующий токен запятой или закрывающей скобкой
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                return Ok(left);
            }
            let right = self.parse_addition_until_comma()?;
            // Проверяем после парсинга правой части
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                left = Expr::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
                return Ok(left);
            }
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить сложение и вычитание до запятой или закрывающей скобки
    fn parse_addition_until_comma(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplication_until_comma()?;
        
        while matches!(self.parser.current_token(), Token::Plus | Token::Minus) {
            let op = match self.parser.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            self.parser.advance();
            self.parser.skip_newlines();
            // Проверяем, не является ли следующий токен запятой или закрывающей скобкой
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                return Ok(left);
            }
            let right = self.parse_multiplication_until_comma()?;
            // Проверяем после парсинга правой части
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                left = Expr::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
                return Ok(left);
            }
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Парсить умножение, деление и остаток от деления до запятой или закрывающей скобки
    fn parse_multiplication_until_comma(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary_until_comma()?;
        
        // Проверяем сразу после парсинга левой части
        if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
            return Ok(left);
        }

        while matches!(self.parser.current_token(), Token::Multiply | Token::Divide | Token::Modulo) {
            let op = match self.parser.current_token() {
                Token::Multiply => BinaryOp::Multiply,
                Token::Divide => BinaryOp::Divide,
                Token::Modulo => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            self.parser.advance();
            self.parser.skip_newlines();
            // После оператора должна быть правая часть - проверка на Comma/RightParen здесь не нужна
            // Если правая часть отсутствует, parse_unary_until_comma выдаст ошибку
            let right = self.parse_unary_until_comma()?;
            // Проверяем после парсинга правой части
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                // Если следующий токен - запятая или закрывающая скобка, возвращаем бинарное выражение
                left = Expr::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
                return Ok(left);
            }
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }
    
    /// Парсить унарные операторы до запятой или закрывающей скобки
    fn parse_unary_until_comma(&mut self) -> Result<Expr> {
        match self.parser.current_token() {
            Token::Not => {
                self.parser.advance();
                let operand = self.parse_unary_until_comma()?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            Token::Minus => {
                self.parser.advance();
                let operand = self.parse_unary_until_comma()?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Minus,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_postfix_until_comma(),
        }
    }
    
    /// Парсить постфиксные операторы до запятой или закрывающей скобки
    fn parse_postfix_until_comma(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        
        // Проверяем сразу после парсинга первичного выражения
        // Если следующий токен - запятая или закрывающая скобка, возвращаем выражение
        if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
            return Ok(expr);
        }
        
        loop {
            match self.parser.current_token() {
                Token::LeftParen => {
                    // Вызов функции
                    if let Expr::Variable(name) = expr {
                        self.parser.advance(); // consume '('
                        let (args, named_args) = self.parse_function_args()?;
                        self.parser.expect(Token::RightParen)?;
                        expr = Expr::FunctionCall { name, args, named_args };
                    } else {
                        break;
                    }
                }
                Token::LeftBracket => {
                    // Индексация
                    self.parser.advance(); // consume '['
                    let index = self.parse_expression_until_comma()?;
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
            
            // Проверяем, не является ли следующий токен запятой или закрывающей скобкой
            if matches!(self.parser.current_token(), Token::Comma | Token::RightParen) {
                break;
            }
        }
        
        Ok(expr)
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
                self.parser.skip_newlines(); // skip newlines after '('
                let expr = self.parse_expression()?;
                self.parser.skip_newlines(); // skip newlines before ')'
                self.parser.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                // Массив
                self.parser.advance(); // consume '['
                self.parser.skip_newlines(); // skip newlines after '['
                let mut elements = Vec::new();

                if !matches!(self.parser.current_token(), Token::RightBracket) {
                    elements.push(self.parse_expression()?);
                    self.parser.skip_newlines(); // skip newlines after first element

                    while matches!(self.parser.current_token(), Token::Comma) {
                        self.parser.advance(); // consume ','
                        self.parser.skip_newlines(); // skip newlines after ','

                        // Check for trailing comma (comma followed by closing bracket)
                        if matches!(self.parser.current_token(), Token::RightBracket) {
                            break;
                        }

                        elements.push(self.parse_expression()?);
                        self.parser.skip_newlines(); // skip newlines after element
                    }
                }

                self.parser.expect(Token::RightBracket)?;
                Ok(Expr::ArrayLiteral { elements })
            }
            Token::LeftBrace => {
                // Объект
                self.parser.advance(); // consume '{'
                self.parser.skip_newlines(); // skip newlines after '{'
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
                    self.parser.skip_newlines(); // skip newlines after first pair

                    // Парсим остальные пары
                    while matches!(self.parser.current_token(), Token::Comma) {
                        self.parser.advance(); // consume ','
                        self.parser.skip_newlines(); // skip newlines after ','

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
                        self.parser.skip_newlines(); // skip newlines after pair
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
            Expr::FunctionCall { name, args, named_args } => {
                assert_eq!(name, "func");
                assert_eq!(args.len(), 2);
                assert_eq!(named_args.len(), 0);
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
