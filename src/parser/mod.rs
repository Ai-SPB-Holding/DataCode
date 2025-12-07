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
pub use statements::StatementParser;


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
        // Диагностика: выводим входной текст для отладки
        if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
            eprintln!("🔍 DEBUG Parser::new: Input text (length: {}): '{}'", input.len(), input);
            // Проверяем, нет ли в тексте многострочного содержимого
            if input.contains('\n') {
                eprintln!("⚠️  DEBUG Parser::new: WARNING - Input contains newlines! This might cause issues.");
                let lines: Vec<&str> = input.lines().collect();
                eprintln!("   First line: '{}'", lines[0]);
                if lines.len() > 1 {
                    eprintln!("   Second line: '{}'", lines[1]);
                }
            }
            // Проверяем, начинается ли текст с 'local' или 'global' - это не должно быть выражением!
            if input.trim().starts_with("local ") || input.trim().starts_with("global ") {
                eprintln!("⚠️  DEBUG Parser::new: CRITICAL - Parser created with 'local' or 'global' statement! This should be handled as a statement, not an expression!");
                eprintln!("   This indicates a bug where a statement is being parsed as an expression.");
                eprintln!("   Input: '{}'", input);
                // НЕ создаем парсер для такого случая - это ошибка
                // Вместо этого вернем парсер с ошибкой, которая будет обработана позже
            }
        }
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
        let old_token = format!("{:?}", self.current_token);
        self.current_token = self.lexer.next_token();
        if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
            if matches!(self.current_token, Token::Local) {
                eprintln!("🔍 DEBUG parser::advance: Advanced from '{}' to Local token", old_token);
                eprintln!("   This might indicate that lexer is reading beyond expression boundaries!");
            }
        }
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
    
    /// Парсить оператор
    #[allow(dead_code)]
    pub fn parse_statement(&mut self) -> Result<Expr> {
        let mut stmt_parser = StatementParser::new(self);
        stmt_parser.parse_statement()
    }
    
    /// Парсить программу (список операторов)
    #[allow(dead_code)]
    pub fn parse_program(&mut self) -> Result<Vec<Expr>> {
        let mut statements = Vec::new();
        
        self.skip_newlines();
        
        while !matches!(self.current_token(), Token::EOF) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }
        
        Ok(statements)
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
