// Модульная структура парсера DataCode
// Этот модуль координирует работу всех компонентов парсера

pub mod tokens;
pub mod lexer;
pub mod expressions;
pub mod statements;

// Реэкспорт основных типов для удобства использования
pub use tokens::{Token, BinaryOp, UnaryOp, Expr};
pub use lexer::Lexer;
pub use expressions::ExpressionParser;


use crate::error::{DataCodeError, Result};

/// Основной парсер DataCode
/// Координирует работу лексера и парсеров выражений/операторов
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    /// Создать новый парсер для заданного входного текста
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }
    
    /// Получить текущий токен
    pub fn current_token(&self) -> &Token {
        &self.current_token
    }
    
    /// Перейти к следующему токену
    pub fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }
    
    /// Ожидать определенный токен и перейти к следующему
    pub fn expect(&mut self, expected: Token) -> Result<()> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(DataCodeError::syntax_error(
                &format!("Expected {:?}, found {:?}", expected, self.current_token),
                1, 0
            ))
        }
    }

    /// Пропустить все токены новой строки
    pub fn skip_newlines(&mut self) {
        while matches!(self.current_token, Token::Newline) {
            self.advance();
        }
    }
    
    /// Парсить выражение
    pub fn parse_expression(&mut self) -> Result<Expr> {
        let mut expr_parser = ExpressionParser::new(self);
        expr_parser.parse_expression()
    }
    
    /// Парсить оператор (будет реализовано в statements.rs)
    pub fn parse_statement(&mut self) -> Result<Expr> {
        // Временная заглушка - просто парсим как выражение
        self.parse_expression()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new("42");
        assert!(matches!(parser.current_token(), Token::Number(_)));
    }

    #[test]
    fn test_parser_advance() {
        let mut parser = Parser::new("42 + 10");
        assert!(matches!(parser.current_token(), Token::Number(_)));
        
        parser.advance();
        assert!(matches!(parser.current_token(), Token::Plus));
        
        parser.advance();
        assert!(matches!(parser.current_token(), Token::Number(_)));
    }

    #[test]
    fn test_simple_expression_parsing() {
        let mut parser = Parser::new("42");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Literal(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_binary_expression_parsing() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Binary { operator, .. } => {
                assert!(matches!(operator, BinaryOp::Add));
            }
            _ => panic!("Expected binary expression"),
        }
    }
}
