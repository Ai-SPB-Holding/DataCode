// Парсер операторов для DataCode
// Реализует парсинг всех операторов согласно BNF грамматике

use super::tokens::{Token, Expr, DeclarationScope};
use super::Parser;
use super::expressions::ExpressionParser;
use crate::error::{DataCodeError, Result};

/// Информация о текущем блоке для проверки вложенности
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum BlockInfo {
    If,
    For { #[allow(dead_code)] var: String },
    Try,
    Function { #[allow(dead_code)] name: String },
}

/// Парсер операторов
#[allow(dead_code)]
pub struct StatementParser<'a> {
    parser: &'a mut Parser,
    block_stack: Vec<BlockInfo>,
}

impl<'a> StatementParser<'a> {
    /// Создать новый парсер операторов
    pub fn new(parser: &'a mut Parser) -> Self {
        Self {
            parser,
            block_stack: Vec::new(),
        }
    }
    
    /// Парсить оператор (точка входа)
    #[allow(dead_code)]
    pub fn parse_statement(&mut self) -> Result<Expr> {
        self.parser.skip_newlines();
        
        match self.parser.current_token() {
            Token::If => self.parse_if_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::Try => self.parse_try_stmt(),
            Token::Function => self.parse_function_def(),
            Token::Return => self.parse_return_stmt(),
            Token::Global | Token::Local => self.parse_declaration(),
            Token::Print => self.parse_print_stmt(),
            Token::EndIf | Token::Next | Token::EndTry | Token::EndFunction => {
                Err(DataCodeError::syntax_error(
                    &format!("Unexpected closing keyword '{:?}' outside of block", self.parser.current_token()),
                    1, 0
                ))
            }
            _ => self.parse_simple_stmt(),
        }
    }
    
    /// Парсить простой оператор (присваивание или выражение)
    fn parse_simple_stmt(&mut self) -> Result<Expr> {
        // Парсим выражение (может быть присваиванием или просто выражением)
        // Для присваивания нужно проверить, является ли выражение Assignment
        // Но проще сначала попробовать распарсить как присваивание, а если не получится - как выражение
        
        // Пробуем парсить как присваивание
        // Присваивание имеет форму: target = expression
        // target может быть: identifier, index, member
        
        // Сначала парсим левую часть как выражение
        let left_expr = {
            let mut expr_parser = ExpressionParser::new(self.parser);
            expr_parser.parse_expression()?
        };
        self.parser.skip_newlines();
        
        // Проверяем, является ли следующее '='
        if matches!(self.parser.current_token(), Token::Assign) {
            // Это присваивание
            self.parser.advance(); // consume '='
            self.parser.skip_newlines();
            let value = self.parse_expression()?;
            return Ok(Expr::Assignment {
                target: Box::new(left_expr),
                value: Box::new(value),
            });
        }
        
        // Это не присваивание, возвращаем как выражение
        Ok(Expr::ExprStmt {
            expr: Box::new(left_expr),
        })
    }
    
    /// Парсить объявление переменных (global/local)
    fn parse_declaration(&mut self) -> Result<Expr> {
        let scope = match self.parser.current_token() {
            Token::Global => DeclarationScope::Global,
            Token::Local => DeclarationScope::Local,
            _ => unreachable!(),
        };
        self.parser.advance();
        self.parser.skip_newlines();
        
        let mut names = Vec::new();
        
        // Парсим список переменных
        if let Token::Identifier(name) = self.parser.current_token().clone() {
            names.push(name.clone());
            self.parser.advance();
            self.parser.skip_newlines();
            
            while matches!(self.parser.current_token(), Token::Comma) {
                self.parser.advance();
                self.parser.skip_newlines();
                
                if let Token::Identifier(name) = self.parser.current_token() {
                    names.push(name.clone());
                    self.parser.advance();
                    self.parser.skip_newlines();
                } else {
                    return Err(DataCodeError::syntax_error(
                        "Expected identifier after comma in declaration",
                        1, 0
                    ));
                }
            }
        } else {
            return Err(DataCodeError::syntax_error(
                "Expected identifier after global/local",
                1, 0
            ));
        }
        
        Ok(Expr::Declaration { scope, names })
    }
    
    /// Парсить оператор return
    fn parse_return_stmt(&mut self) -> Result<Expr> {
        self.parser.advance(); // consume 'return'
        self.parser.skip_newlines();
        
        let value = if matches!(self.parser.current_token(), Token::Newline | Token::EOF) {
            None
        } else {
            let mut expr_parser = ExpressionParser::new(self.parser);
            Some(Box::new(expr_parser.parse_expression()?))
        };
        
        Ok(Expr::ReturnStmt { value })
    }
    
    /// Парсить оператор print
    fn parse_print_stmt(&mut self) -> Result<Expr> {
        self.parser.advance(); // consume 'print'
        self.parser.expect(Token::LeftParen)?;
        self.parser.skip_newlines();
        
        let mut args = Vec::new();
        
        if !matches!(self.parser.current_token(), Token::RightParen) {
            args.push({
                let mut expr_parser = ExpressionParser::new(self.parser);
                expr_parser.parse_expression()?
            });
            self.parser.skip_newlines();
            
            while matches!(self.parser.current_token(), Token::Comma) {
                self.parser.advance();
                self.parser.skip_newlines();
                
                if matches!(self.parser.current_token(), Token::RightParen) {
                    break;
                }
                
                args.push({
                    let mut expr_parser = ExpressionParser::new(self.parser);
                    expr_parser.parse_expression()?
                });
                self.parser.skip_newlines();
            }
        }
        
        self.parser.expect(Token::RightParen)?;
        
        Ok(Expr::PrintStmt { args })
    }
    
    /// Парсить блок операторов до терминатора
    fn parse_block(&mut self, terminators: &[Token]) -> Result<Vec<Expr>> {
        let mut statements = Vec::new();
        
        self.parser.skip_newlines();
        
        loop {
            // Проверяем, не достигли ли мы терминатора
            if terminators.iter().any(|t| {
                std::mem::discriminant(self.parser.current_token()) == std::mem::discriminant(t)
            }) {
                break;
            }
            
            // Проверяем EOF
            if matches!(self.parser.current_token(), Token::EOF) {
                if !terminators.is_empty() {
                    return Err(DataCodeError::syntax_error(
                        &format!("Unexpected end of file, expected one of: {:?}", terminators),
                        1, 0
                    ));
                }
                break;
            }
            
            // Парсим оператор
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            
            // Пропускаем newline после оператора
            self.parser.skip_newlines();
        }
        
        Ok(statements)
    }
    
    /// Парсить оператор if
    fn parse_if_stmt(&mut self) -> Result<Expr> {
        self.parser.advance(); // consume 'if'
        self.parser.skip_newlines();
        
        // Парсим условие
        let mut expr_parser = ExpressionParser::new(self.parser);
        let condition = expr_parser.parse_expression()?;
        self.parser.skip_newlines();
        
        // Ожидаем 'do'
        self.parser.expect(Token::Do)?;
        self.parser.skip_newlines();
        
        // Парсим блок then
        self.block_stack.push(BlockInfo::If);
        let then_block = self.parse_block(&[Token::Else, Token::EndIf])?;
        self.block_stack.pop();
        
        let else_block = if matches!(self.parser.current_token(), Token::Else) {
            self.parser.advance(); // consume 'else'
            self.parser.expect(Token::Do)?;
            self.parser.skip_newlines();
            
            self.block_stack.push(BlockInfo::If);
            let else_statements = self.parse_block(&[Token::EndIf])?;
            self.block_stack.pop();
            
            Some(Box::new(Expr::Block {
                statements: else_statements,
            }))
        } else {
            None
        };
        
        // Ожидаем 'endif'
        self.parser.expect(Token::EndIf)?;
        
        Ok(Expr::IfStmt {
            condition: Box::new(condition),
            then_block: Box::new(Expr::Block {
                statements: then_block,
            }),
            else_block,
        })
    }
    
    /// Парсить оператор for
    fn parse_for_stmt(&mut self) -> Result<Expr> {
        self.parser.advance(); // consume 'for'
        self.parser.skip_newlines();
        
        // Парсим переменные (может быть одна или несколько через запятую)
        let mut vars = Vec::new();
        
        if let Token::Identifier(name) = self.parser.current_token() {
            vars.push(name.clone());
            self.parser.advance();
            self.parser.skip_newlines();
            
            // Проверяем, есть ли еще переменные через запятую
            while matches!(self.parser.current_token(), Token::Comma) {
                self.parser.advance(); // consume ','
                self.parser.skip_newlines();
                
                if let Token::Identifier(name) = self.parser.current_token() {
                    vars.push(name.clone());
                    self.parser.advance();
                    self.parser.skip_newlines();
                } else {
                    return Err(DataCodeError::syntax_error(
                        "Expected identifier after comma in for loop",
                        1, 0
                    ));
                }
            }
        } else {
            return Err(DataCodeError::syntax_error(
                "Expected identifier after 'for'",
                1, 0
            ));
        }
        
        // Ожидаем 'in'
        self.parser.expect(Token::In)?;
        self.parser.skip_newlines();
        
        // Парсим выражение итерации
        let mut expr_parser = ExpressionParser::new(self.parser);
        let iter_expr = expr_parser.parse_expression()?;
        self.parser.skip_newlines();
        
        // Ожидаем 'do'
        self.parser.expect(Token::Do)?;
        self.parser.skip_newlines();
        
        // Парсим тело цикла
        // Используем первую переменную для BlockInfo (для обратной совместимости)
        let first_var = vars[0].clone();
        self.block_stack.push(BlockInfo::For { var: first_var.clone() });
        let body = self.parse_block(&[Token::Next])?;
        self.block_stack.pop();
        
        // Ожидаем 'next'
        self.parser.expect(Token::Next)?;
        self.parser.skip_newlines();
        
        // Проверяем, что имя переменной в next совпадает с первой переменной
        if let Token::Identifier(next_var) = self.parser.current_token() {
            if next_var != &first_var {
                return Err(DataCodeError::syntax_error(
                    &format!("Mismatched next: expected 'next {}' but found 'next {}'", first_var, next_var),
                    1, 0
                ));
            }
            self.parser.advance();
        } else {
            return Err(DataCodeError::syntax_error(
                &format!("Expected identifier after 'next', expected 'next {}'", first_var),
                1, 0
            ));
        }
        
        Ok(Expr::ForStmt {
            vars,
            iter_expr: Box::new(iter_expr),
            body: Box::new(Expr::Block {
                statements: body,
            }),
        })
    }
    
    /// Парсить оператор try
    fn parse_try_stmt(&mut self) -> Result<Expr> {
        self.parser.advance(); // consume 'try'
        self.parser.skip_newlines();
        
        // Парсим блок try
        self.block_stack.push(BlockInfo::Try);
        let try_block = self.parse_block(&[Token::Catch])?;
        self.block_stack.pop();
        
        // Ожидаем 'catch'
        self.parser.expect(Token::Catch)?;
        self.parser.skip_newlines();
        
        // Парсим переменную для исключения
        let catch_var = if let Token::Identifier(name) = self.parser.current_token() {
            let name = name.clone();
            self.parser.advance();
            self.parser.skip_newlines();
            name
        } else {
            return Err(DataCodeError::syntax_error(
                "Expected identifier after 'catch'",
                1, 0
            ));
        };
        
        // Парсим блок catch
        self.block_stack.push(BlockInfo::Try);
        let catch_block = self.parse_block(&[Token::EndTry])?;
        self.block_stack.pop();
        
        // Ожидаем 'endtry'
        self.parser.expect(Token::EndTry)?;
        
        Ok(Expr::TryStmt {
            try_block: Box::new(Expr::Block {
                statements: try_block,
            }),
            catch_var,
            catch_block: Box::new(Expr::Block {
                statements: catch_block,
            }),
        })
    }
    
    /// Парсить определение функции
    fn parse_function_def(&mut self) -> Result<Expr> {
        self.parser.advance(); // consume 'function'
        self.parser.skip_newlines();
        
        // Парсим имя функции
        let name = if let Token::Identifier(name) = self.parser.current_token() {
            let name = name.clone();
            self.parser.advance();
            self.parser.skip_newlines();
            name
        } else {
            return Err(DataCodeError::syntax_error(
                "Expected identifier after 'function'",
                1, 0
            ));
        };
        
        // Ожидаем '('
        self.parser.expect(Token::LeftParen)?;
        self.parser.skip_newlines();
        
        // Парсим параметры
        let mut params = Vec::new();
        
        if !matches!(self.parser.current_token(), Token::RightParen) {
            if let Token::Identifier(param) = self.parser.current_token() {
                params.push(param.clone());
                self.parser.advance();
                self.parser.skip_newlines();
                
                while matches!(self.parser.current_token(), Token::Comma) {
                    self.parser.advance();
                    self.parser.skip_newlines();
                    
                    if let Token::Identifier(param) = self.parser.current_token() {
                        params.push(param.clone());
                        self.parser.advance();
                        self.parser.skip_newlines();
                    } else {
                        return Err(DataCodeError::syntax_error(
                            "Expected identifier after comma in function parameters",
                            1, 0
                        ));
                    }
                }
            } else {
                return Err(DataCodeError::syntax_error(
                    "Expected identifier or ')' in function parameters",
                    1, 0
                ));
            }
        }
        
        // Ожидаем ')'
        self.parser.expect(Token::RightParen)?;
        self.parser.skip_newlines();
        
        // Парсим тело функции
        self.block_stack.push(BlockInfo::Function { name: name.clone() });
        let body = self.parse_block(&[Token::EndFunction])?;
        self.block_stack.pop();
        
        // Ожидаем 'endfunction'
        self.parser.expect(Token::EndFunction)?;
        
        Ok(Expr::FunctionDef {
            name,
            params,
            body: Box::new(Expr::Block {
                statements: body,
            }),
        })
    }
    
    /// Парсить выражение (вспомогательный метод)
    fn parse_expression(&mut self) -> Result<Expr> {
        let mut expr_parser = ExpressionParser::new(self.parser);
        expr_parser.parse_expression()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_parser_creation() {
        let mut parser = Parser::new("42");
        let _stmt_parser = StatementParser::new(&mut parser);
    }
    
    #[test]
    fn test_parse_assignment() {
        let mut parser = Parser::new("x = 42");
        let mut stmt_parser = StatementParser::new(&mut parser);
        let stmt = stmt_parser.parse_statement().unwrap();
        
        match stmt {
            Expr::Assignment { target, value } => {
                assert!(matches!(target.as_ref(), Expr::Variable(_)));
                assert!(matches!(value.as_ref(), Expr::Literal(_)));
            }
            _ => panic!("Expected assignment"),
        }
    }
    
    #[test]
    fn test_parse_declaration() {
        let mut parser = Parser::new("global x, y");
        let mut stmt_parser = StatementParser::new(&mut parser);
        let stmt = stmt_parser.parse_statement().unwrap();
        
        match stmt {
            Expr::Declaration { scope: DeclarationScope::Global, names } => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "x");
                assert_eq!(names[1], "y");
            }
            _ => panic!("Expected global declaration"),
        }
    }
    
    #[test]
    fn test_parse_print() {
        let mut parser = Parser::new("print(42)");
        let mut stmt_parser = StatementParser::new(&mut parser);
        let stmt = stmt_parser.parse_statement().unwrap();
        
        match stmt {
            Expr::PrintStmt { args } => {
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected print statement"),
        }
    }
}
