// Recursive Descent Parser

use crate::lexer::{Token, TokenKind};
use crate::parser::ast::{Expr, Stmt, Param, Arg};
use crate::common::error::LangError;
use crate::common::value::Value;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, LangError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, LangError> {
        if self.match_token(TokenKind::Global) {
            // global a = 5
            let global_line = self.previous().line;
            let name = self.consume(TokenKind::Identifier, "Expect variable name after 'global'")?.lexeme.clone();
            self.consume(TokenKind::Equal, "Expect '=' after variable name")?;
            let value = self.expression()?;
            Ok(Stmt::Let { name, value, is_global: true, line: global_line })
        } else if self.match_token(TokenKind::Let) {
            self.variable_declaration()
        } else if self.check(TokenKind::At) || self.check(TokenKind::Fn) {
            self.function_declaration()
        } else {
            self.statement()
        }
    }

    fn variable_declaration(&mut self) -> Result<Stmt, LangError> {
        let let_line = self.previous().line;
        let name = self.consume(TokenKind::Identifier, "Expect variable name")?.lexeme.clone();
        self.consume(TokenKind::Equal, "Expect '=' after variable name")?;
        let value = self.expression()?;
        Ok(Stmt::Let { name, value, is_global: false, line: let_line })
    }

    fn function_declaration(&mut self) -> Result<Stmt, LangError> {
        // Проверяем наличие аннотации @cache
        let is_cached = if self.match_token(TokenKind::At) {
            self.consume(TokenKind::Cache, "Expect 'cache' after '@'")?;
            true
        } else {
            false
        };
        
        // Парсим fn
        self.consume(TokenKind::Fn, "Expect 'fn'")?;
        let fn_line = self.previous().line;
        let name = self.consume(TokenKind::Identifier, "Expect function name")?.lexeme.clone();
        self.consume(TokenKind::LParen, "Expect '(' after function name")?;

        let mut params = Vec::new();
        let mut has_default = false;
        if !self.check(TokenKind::RParen) {
            loop {
                if params.len() >= 255 {
                    return Err(LangError::ParseError {
                        message: "Cannot have more than 255 parameters".to_string(),
                        line: self.previous().line,
                    });
                }
                
                let param_name = self.consume(TokenKind::Identifier, "Expect parameter name")?.lexeme.clone();
                let param_line = self.previous().line;
                
                // Проверяем, есть ли значение по умолчанию
                let default_value = if self.match_token(TokenKind::Equal) {
                    has_default = true;
                    Some(self.expression()?)
                } else {
                    // Проверяем порядок: обязательный параметр не может идти после параметра с default
                    if has_default {
                        return Err(LangError::ParseError {
                            message: "Non-default argument follows default argument".to_string(),
                            line: param_line,
                        });
                    }
                    None
                };
                
                params.push(Param {
                    name: param_name,
                    default_value,
                });
                
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen, "Expect ')' after parameters")?;
        self.consume(TokenKind::LBrace, "Expect '{' before function body")?;

        let body = self.block()?;

        Ok(Stmt::Function { name, params, body, is_cached, line: fn_line })
    }

    fn statement(&mut self) -> Result<Stmt, LangError> {
        if self.match_token(TokenKind::If) {
            self.if_statement()
        } else if self.match_token(TokenKind::While) {
            self.while_statement()
        } else if self.match_token(TokenKind::For) {
            self.for_statement()
        } else if self.match_token(TokenKind::Return) {
            self.return_statement()
        } else if self.match_token(TokenKind::Break) {
            self.break_statement()
        } else if self.match_token(TokenKind::Continue) {
            self.continue_statement()
        } else if self.match_token(TokenKind::Throw) {
            self.throw_statement()
        } else if self.match_token(TokenKind::Try) {
            self.try_statement()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, LangError> {
        let if_line = self.previous().line;
        // Скобки опциональны - парсим условие как выражение
        // Если есть скобки, они будут частью выражения, а не синтаксиса if
        let condition = self.expression()?;
        // Проверяем, есть ли скобки вокруг условия (опциональные)
        // Если следующая лексема - это '{', значит скобок не было
        // Если следующая лексема - это ')', значит были скобки, пропускаем их
        if self.match_token(TokenKind::RParen) {
            // Были скобки, пропустили закрывающую
        }
        self.consume(TokenKind::LBrace, "Expect '{' after condition")?;
        let then_branch = self.block()?;
        
        let else_branch = if self.match_token(TokenKind::Else) {
            // Проверяем, является ли следующий токен 'if' (else if)
            if self.check(TokenKind::If) {
                // Потребляем 'if' и рекурсивно парсим if_statement для else if
                self.advance(); // Потребляем токен 'if'
                Some(vec![self.if_statement()?])
            } else {
                // Обычный else блок
                self.consume(TokenKind::LBrace, "Expect '{' after 'else'")?;
                Some(self.block()?)
            }
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            line: if_line,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, LangError> {
        let while_line = self.previous().line;
        // Скобки опциональны - парсим условие как выражение
        let condition = self.expression()?;
        // Проверяем, есть ли скобки вокруг условия (опциональные)
        if self.match_token(TokenKind::RParen) {
            // Были скобки, пропустили закрывающую
        }
        self.consume(TokenKind::LBrace, "Expect '{' after condition")?;
        let body = self.block()?;
        Ok(Stmt::While { condition, body, line: while_line })
    }

    fn for_statement(&mut self) -> Result<Stmt, LangError> {
        let for_line = self.previous().line;
        
        // Парсим: for variable in iterable { body }
        let variable = self.consume(TokenKind::Identifier, "Expect variable name after 'for'")?.lexeme.clone();
        self.consume(TokenKind::In, "Expect 'in' after variable name")?;
        let iterable = self.expression()?;
        self.consume(TokenKind::LBrace, "Expect '{' before loop body")?;
        let body = self.block()?;

        Ok(Stmt::For {
            variable,
            iterable,
            body,
            line: for_line,
        })
    }

    fn return_statement(&mut self) -> Result<Stmt, LangError> {
        let return_line = self.previous().line;
        let value = if !self.check(TokenKind::Semicolon) && !self.check(TokenKind::RBrace) {
            Some(self.expression()?)
        } else {
            None
        };
        // Семиколон опционален для return
        self.match_token(TokenKind::Semicolon);
        Ok(Stmt::Return { value, line: return_line })
    }

    fn break_statement(&mut self) -> Result<Stmt, LangError> {
        let break_line = self.previous().line;
        // Семиколон опционален для break
        self.match_token(TokenKind::Semicolon);
        Ok(Stmt::Break { line: break_line })
    }

    fn continue_statement(&mut self) -> Result<Stmt, LangError> {
        let continue_line = self.previous().line;
        // Семиколон опционален для continue
        self.match_token(TokenKind::Semicolon);
        Ok(Stmt::Continue { line: continue_line })
    }

    fn throw_statement(&mut self) -> Result<Stmt, LangError> {
        let throw_line = self.previous().line;
        // Парсим выражение (значение ошибки)
        let value = self.expression()?;
        // Семиколон опционален для throw
        self.match_token(TokenKind::Semicolon);
        Ok(Stmt::Throw { value, line: throw_line })
    }

    fn try_statement(&mut self) -> Result<Stmt, LangError> {
        use crate::parser::ast::CatchBlock;
        
        let try_line = self.previous().line;
        
        // Парсим try блок
        self.consume(TokenKind::LBrace, "Expect '{' after 'try'")?;
        let try_block = self.block()?;
        
        // Парсим catch блоки (должен быть хотя бы один)
        let mut catch_blocks = Vec::new();
        
        while self.match_token(TokenKind::Catch) {
            let catch_line = self.previous().line;
            
            // Парсим тип ошибки (опционально)
            let error_type = if self.check(TokenKind::Identifier) {
                let error_type_name = self.peek().lexeme.clone();
                // Проверяем, является ли это типом ошибки
                if crate::common::error::ErrorType::from_name(&error_type_name).is_some() {
                    self.advance();
                    Some(error_type_name)
                } else {
                    None
                }
            } else {
                None
            };
            
            // Парсим переменную ошибки (опционально)
            let error_var = if self.match_token(TokenKind::Identifier) {
                Some(self.previous().lexeme.clone())
            } else {
                None
            };
            
            // Парсим тело catch блока
            self.consume(TokenKind::LBrace, "Expect '{' after 'catch'")?;
            let catch_body = self.block()?;
            
            catch_blocks.push(CatchBlock {
                error_type,
                error_var,
                body: catch_body,
                line: catch_line,
            });
        }
        
        // Проверяем, что есть хотя бы один catch блок
        if catch_blocks.is_empty() {
            return Err(LangError::ParseError {
                message: "try statement must have at least one catch block".to_string(),
                line: try_line,
            });
        }
        
        // Парсим else блок (опционально)
        let else_block = if self.match_token(TokenKind::Else) {
            self.consume(TokenKind::LBrace, "Expect '{' after 'else'")?;
            Some(self.block()?)
        } else {
            None
        };
        
        Ok(Stmt::Try {
            try_block,
            catch_blocks,
            else_block,
            line: try_line,
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, LangError> {
        let expr = self.expression()?;
        let line = expr.line();
        // Семиколон опционален для выражений
        self.match_token(TokenKind::Semicolon);
        Ok(Stmt::Expr { expr, line })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, LangError> {
        let mut statements = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenKind::RBrace, "Expect '}' after block")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, LangError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LangError> {
        let expr = self.or_expression()?;
        
        // Проверяем операторы присваивания (+=, -=, *=, /=, //=, %=, **=)
        if self.match_token(TokenKind::PlusEqual)
            || self.match_token(TokenKind::MinusEqual)
            || self.match_token(TokenKind::StarEqual)
            || self.match_token(TokenKind::StarStarEqual)
            || self.match_token(TokenKind::SlashEqual)
            || self.match_token(TokenKind::SlashSlashEqual)
            || self.match_token(TokenKind::PercentEqual)
        {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            if let Expr::Variable { name, .. } = expr {
                let value = self.assignment()?;
                return Ok(Expr::AssignOp {
                    name,
                    op: op_kind,
                    value: Box::new(value),
                    line: op_line,
                });
            }
            return Err(LangError::ParseError {
                message: "Invalid assignment target".to_string(),
                line: op_line,
            });
        }
        
        // Обычное присваивание (=)
        if self.match_token(TokenKind::Equal) {
            let equal_line = self.previous().line;
            if let Expr::Variable { name, .. } = expr {
                let value = self.assignment()?;
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                    line: equal_line,
                });
            }
            return Err(LangError::ParseError {
                message: "Invalid assignment target".to_string(),
                line: equal_line,
            });
        }
        Ok(expr)
    }

    fn or_expression(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.and_expression()?;
        while self.match_token(TokenKind::Or) {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            let right = self.and_expression()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op_kind,
                right: Box::new(right),
                line: op_line,
            };
        }
        Ok(expr)
    }

    fn and_expression(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.equality()?;
        while self.match_token(TokenKind::And) {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op_kind,
                right: Box::new(right),
                line: op_line,
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.comparison()?;
        while self.match_token(TokenKind::BangEqual) || self.match_token(TokenKind::EqualEqual) {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op_kind,
                right: Box::new(right),
                line: op_line,
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.term()?;
        while self.match_token(TokenKind::Greater)
            || self.match_token(TokenKind::GreaterEqual)
            || self.match_token(TokenKind::Less)
            || self.match_token(TokenKind::LessEqual)
            || self.match_token(TokenKind::In)
        {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op_kind,
                right: Box::new(right),
                line: op_line,
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.factor()?;
        while self.match_token(TokenKind::Minus) || self.match_token(TokenKind::Plus) {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op_kind,
                right: Box::new(right),
                line: op_line,
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.exponent()?;
        while self.match_token(TokenKind::Slash) || self.match_token(TokenKind::SlashSlash) || self.match_token(TokenKind::Star) || self.match_token(TokenKind::Percent) {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            let right = self.exponent()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op_kind,
                right: Box::new(right),
                line: op_line,
            };
        }
        Ok(expr)
    }

    fn exponent(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.unary()?;
        // Exponentiation is right-associative: 2 ** 3 ** 2 = 2 ** (3 ** 2)
        // Don't match ** if the next token is **= (to avoid consuming **=)
        while self.check(TokenKind::StarStar) && !self.check_next(TokenKind::StarStarEqual) {
            self.advance(); // Consume StarStar
            let op_line = self.previous().line;
            let right = self.exponent()?; // Recursive call for right-associativity
            expr = Expr::Binary {
                left: Box::new(expr),
                op: TokenKind::StarStar,
                right: Box::new(right),
                line: op_line,
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LangError> {
        if self.match_token(TokenKind::Bang) || self.match_token(TokenKind::Minus) {
            let op_line = self.previous().line;
            let op_kind = self.previous().kind.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op: op_kind,
                right: Box::new(right),
                line: op_line,
            });
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, LangError> {
        let mut expr = self.primary()?;
        loop {
            // Обрабатываем вызовы функций (круглые скобки)
            // Может быть вызовом переменной или метода (Property)
            // Но не группировкой выражений - проверяем тип выражения перед обработкой
            if self.check(TokenKind::LParen) {
                // Проверяем, что это действительно вызов функции/метода
                // (переменная или Property), а не просто группировка
                match &expr {
                    Expr::Variable { .. } | Expr::Property { .. } => {
                        self.advance(); // Съедаем LParen
                        expr = self.finish_call(expr)?;
                        continue;
                    }
                    _ => {
                        // Это не вызов функции - не обрабатываем скобки здесь
                        // Позволим более высокому уровню обработать это
                        break;
                    }
                }
            }
            
            // Обрабатываем индексацию массивов (квадратные скобки)
            // Массивом может быть любое выражение, не только переменная
            if self.match_token(TokenKind::LBracket) {
                expr = self.finish_array_index(expr)?;
                continue;
            }
            
            // Обрабатываем доступ к свойствам (точка)
            if self.match_token(TokenKind::Dot) {
                let name = self.consume(TokenKind::Identifier, "Expect property name after '.'")?.lexeme.clone();
                let line = self.previous().line;
                expr = Expr::Property {
                    object: Box::new(expr),
                    name,
                    line,
                };
                continue;
            }
            
            // Если ни вызов функции, ни индексация, ни свойство - выходим из цикла
            break;
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LangError> {
        let call_line = self.previous().line; // Номер строки открывающей скобки (LParen)
        let mut args = Vec::new();
        let mut has_named = false;
        if !self.check(TokenKind::RParen) {
            loop {
                if args.len() >= 255 {
                    return Err(LangError::ParseError {
                        message: "Cannot have more than 255 arguments".to_string(),
                        line: self.peek().line,
                    });
                }
                
                // Проверяем, является ли это именованным аргументом (identifier = expression)
                // Сохраняем текущую позицию для проверки
                let arg = if self.check(TokenKind::Identifier) {
                    // Проверяем, является ли следующий токен '='
                    // Сохраняем текущую позицию
                    let saved_current = self.current;
                    // Временно продвигаемся вперед для проверки
                    let is_named = if saved_current + 1 < self.tokens.len() {
                        self.tokens[saved_current + 1].kind == TokenKind::Equal
                    } else {
                        false
                    };
                    
                    if is_named {
                        // Именованный аргумент: name = value
                        let name_token = self.advance();
                        let name = name_token.lexeme.clone();
                        self.consume(TokenKind::Equal, "Expect '=' after parameter name in named argument")?;
                        has_named = true;
                        Arg::Named {
                            name,
                            value: self.expression()?,
                        }
                    } else {
                        // Позиционный аргумент
                        // Проверяем, что после именованного аргумента не идет позиционный
                        if has_named {
                            return Err(LangError::ParseError {
                                message: "Positional argument follows named argument".to_string(),
                                line: self.peek().line,
                            });
                        }
                        Arg::Positional(self.expression()?)
                    }
                } else {
                    // Позиционный аргумент (не идентификатор)
                    // Проверяем, что после именованного аргумента не идет позиционный
                    if has_named {
                        return Err(LangError::ParseError {
                            message: "Positional argument follows named argument".to_string(),
                            line: self.peek().line,
                        });
                    }
                    Arg::Positional(self.expression()?)
                };
                
                args.push(arg);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenKind::RParen, "Expect ')' after arguments")?;
        
        // Извлекаем имя функции из callee
        // Может быть переменной или методом (Property)
        match callee {
            Expr::Variable { name, .. } => {
                Ok(Expr::Call { name, args, line: call_line })
            }
            Expr::Property { object, name, .. } => {
                // Это вызов метода - создаем MethodCall
                Ok(Expr::MethodCall {
                    object,
                    method: name,
                    args,
                    line: call_line,
                })
            }
            _ => {
                // Для сложных выражений пока не поддерживаем вызовы
                Err(LangError::ParseError {
                    message: "Can only call functions, variables, and methods".to_string(),
                    line: paren.line,
                })
            }
        }
    }

    fn finish_array_index(&mut self, array: Expr) -> Result<Expr, LangError> {
        let index_line = self.previous().line; // Номер строки открывающей скобки (LBracket)
        let index = self.expression()?;
        self.consume(TokenKind::RBracket, "Expect ']' after array index")?;
        
        Ok(Expr::ArrayIndex {
            array: Box::new(array),
            index: Box::new(index),
            line: index_line,
        })
    }

    fn primary(&mut self) -> Result<Expr, LangError> {
        if self.match_token(TokenKind::False) {
            let line = self.previous().line;
            return Ok(Expr::Literal { value: Value::Bool(false), line });
        }
        if self.match_token(TokenKind::True) {
            let line = self.previous().line;
            return Ok(Expr::Literal { value: Value::Bool(true), line });
        }
        if self.match_token(TokenKind::Null) {
            let line = self.previous().line;
            return Ok(Expr::Literal { value: Value::Null, line });
        }
        if self.match_token(TokenKind::Number) {
            let line = self.previous().line;
            let lexeme = self.previous().lexeme.clone();
            let value = lexeme.parse::<f64>()
                .map_err(|_| LangError::ParseError {
                    message: "Invalid number".to_string(),
                    line,
                })?;
            return Ok(Expr::Literal { value: Value::Number(value), line });
        }
        if self.match_token(TokenKind::String) {
            let line = self.previous().line;
            let lexeme = self.previous().lexeme.clone();
            let value = lexeme[1..lexeme.len() - 1].to_string(); // Убираем кавычки
            return Ok(Expr::Literal { value: Value::String(value), line });
        }
        if self.match_token(TokenKind::Identifier) {
            let line = self.previous().line;
            let name = self.previous().lexeme.clone();
            return Ok(Expr::Variable { name, line });
        }
        if self.match_token(TokenKind::LParen) {
            let expr = self.expression()?;
            self.consume(TokenKind::RParen, "Expect ')' after expression")?;
            return Ok(expr);
        }
        if self.match_token(TokenKind::LBracket) {
            return self.array_literal();
        }

        let token = self.peek();
        Err(LangError::ParseError {
            message: format!("Expect expression, found {:?} '{}' at line {}", token.kind, token.lexeme, token.line),
            line: token.line,
        })
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().kind == kind
        }
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        if self.is_at_end() || self.current + 1 >= self.tokens.len() {
            false
        } else {
            self.tokens[self.current + 1].kind == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn array_literal(&mut self) -> Result<Expr, LangError> {
        let line = self.previous().line;
        let mut elements = Vec::new();

        if !self.check(TokenKind::RBracket) {
            loop {
                elements.push(self.expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RBracket, "Expect ']' after array elements")?;

        // Если все элементы - литералы, создаем Value::Array напрямую
        // Иначе создаем ArrayLiteral для компиляции во время выполнения
        let mut all_literals = true;
        let mut values = Vec::new();
        
        for expr in &elements {
            match expr {
                Expr::Literal { value, .. } => {
                    values.push(value.clone());
                }
                _ => {
                    all_literals = false;
                    break;
                }
            }
        }

        if all_literals {
            Ok(Expr::Literal {
                value: Value::Array(Rc::new(RefCell::new(values))),
                line,
            })
        } else {
            Ok(Expr::ArrayLiteral {
                elements,
                line,
            })
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, LangError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(LangError::ParseError {
                message: message.to_string(),
                line: self.peek().line,
            })
        }
    }
}

