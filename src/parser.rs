use crate::value::Value;
use crate::error::{DataCodeError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Литералы
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Identifier(String),
    
    // Операторы
    Plus,           // +
    Minus,          // -
    Multiply,       // *
    Divide,         // /
    PathJoin,       // / (для путей)
    
    // Операторы сравнения
    Equal,          // ==
    NotEqual,       // !=
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    
    // Логические операторы
    And,            // and
    Or,             // or
    Not,            // not

    // Обработка исключений
    Try,            // try
    Catch,          // catch
    Finally,        // finally
    Throw,          // throw
    
    // Скобки
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    
    // Разделители
    Comma,          // ,
    Dot,            // .
    
    // Специальные
    Assign,         // =
    Newline,
    EOF,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Member {
        object: Box<Expr>,
        member: String,
    },
    ArrayLiteral {
        elements: Vec<Expr>,
    },
    TryBlock {
        try_body: Vec<String>,
        catch_var: Option<String>,
        catch_body: Vec<String>,
        finally_body: Option<Vec<String>>,
    },
    ThrowStatement {
        message: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    PathJoin,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Minus,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Self {
            input: chars,
            position: 0,
            current_char,
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> String {
        let mut result = String::new();
        self.advance(); // Skip opening quote
        
        while let Some(ch) = self.current_char {
            if ch == '\'' {
                self.advance(); // Skip closing quote
                break;
            }
            result.push(ch);
            self.advance();
        }
        
        result
    }
    
    fn read_number(&mut self) -> f64 {
        let mut result = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '.' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result.parse().unwrap_or(0.0)
    }
    
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result
    }
    
    pub fn next_token(&mut self) -> Token {
        loop {
            match self.current_char {
                None => return Token::EOF,
                Some(' ') | Some('\t') | Some('\r') => {
                    self.skip_whitespace();
                    continue;
                }
                Some('\n') => {
                    self.advance();
                    return Token::Newline;
                }
                Some('\'') => {
                    let string_val = self.read_string();
                    return Token::String(string_val);
                }
                Some(ch) if ch.is_ascii_digit() => {
                    let num = self.read_number();
                    return Token::Number(num);
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let ident = self.read_identifier();
                    return match ident.as_str() {
                        "true" => Token::Bool(true),
                        "false" => Token::Bool(false),
                        "null" => Token::Null,
                        "and" => Token::And,
                        "or" => Token::Or,
                        "not" => Token::Not,
                        "try" => Token::Try,
                        "catch" => Token::Catch,
                        "finally" => Token::Finally,
                        "throw" => Token::Throw,
                        _ => Token::Identifier(ident),
                    };
                }
                Some('+') => {
                    self.advance();
                    return Token::Plus;
                }
                Some('-') => {
                    self.advance();
                    return Token::Minus;
                }
                Some('*') => {
                    self.advance();
                    return Token::Multiply;
                }
                Some('/') => {
                    self.advance();
                    // Контекстно определяем: если предыдущий токен - число, то это деление
                    // Иначе - соединение путей
                    return Token::Divide; // По умолчанию деление, логика будет в парсере
                }
                Some('=') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Token::Equal;
                    } else {
                        self.advance();
                        return Token::Assign;
                    }
                }
                Some('!') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Token::NotEqual;
                    } else {
                        self.advance();
                        return Token::Not;
                    }
                }
                Some('<') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Token::LessEqual;
                    } else {
                        self.advance();
                        return Token::Less;
                    }
                }
                Some('>') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Token::GreaterEqual;
                    } else {
                        self.advance();
                        return Token::Greater;
                    }
                }
                Some('(') => {
                    self.advance();
                    return Token::LeftParen;
                }
                Some(')') => {
                    self.advance();
                    return Token::RightParen;
                }
                Some('[') => {
                    self.advance();
                    return Token::LeftBracket;
                }
                Some(']') => {
                    self.advance();
                    return Token::RightBracket;
                }
                Some(',') => {
                    self.advance();
                    return Token::Comma;
                }
                Some('.') => {
                    self.advance();
                    return Token::Dot;
                }
                Some(_ch) => {
                    self.advance();
                    // Пропускаем неизвестные символы
                    continue;
                }
            }
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }
    
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }
    
    fn expect(&mut self, expected: Token) -> Result<()> {
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
    
    pub fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        
        while matches!(self.current_token, Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        
        while matches!(self.current_token, Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        
        while matches!(self.current_token, Token::Equal | Token::NotEqual) {
            let op = match self.current_token {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_addition()?;
        
        while matches!(self.current_token, Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual) {
            let op = match self.current_token {
                Token::Less => BinaryOp::Less,
                Token::Greater => BinaryOp::Greater,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_addition(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplication()?;
        
        while matches!(self.current_token, Token::Plus | Token::Minus | Token::Divide) {
            let op = match self.current_token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                Token::Divide => {
                    // Оператор / будет интеллектуально обработан в интерпретаторе
                    // PathJoin для путей, математическое деление для чисел
                    BinaryOp::Divide
                }
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_multiplication(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        while matches!(self.current_token, Token::Multiply | Token::Divide) {
            let op = match self.current_token {
                Token::Multiply => BinaryOp::Multiply,
                Token::Divide => BinaryOp::Divide, // Интеллектуальная обработка в интерпретаторе
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<Expr> {
        match self.current_token {
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Minus,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }
    
    fn parse_primary(&mut self) -> Result<Expr> {
        match &self.current_token {
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::Literal(Value::String(value)))
            }
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expr::Literal(Value::Number(value)))
            }
            Token::Bool(b) => {
                let value = *b;
                self.advance();
                Ok(Expr::Literal(Value::Bool(value)))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Literal(Value::Null))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Проверяем, является ли это вызовом функции
                let mut expr = if matches!(self.current_token, Token::LeftParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();

                    if !matches!(self.current_token, Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if matches!(self.current_token, Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect(Token::RightParen)?;
                    Expr::FunctionCall { name, args }
                } else {
                    Expr::Variable(name)
                };

                // Проверяем индексирование
                while matches!(self.current_token, Token::LeftBracket) {
                    self.advance(); // consume '['
                    let index = self.parse_expression()?;
                    self.expect(Token::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }

                Ok(expr)
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                self.advance(); // consume '['
                let mut elements = Vec::new();

                // Проверяем пустой массив
                if matches!(self.current_token, Token::RightBracket) {
                    self.advance(); // consume ']'
                    return Ok(Expr::ArrayLiteral { elements });
                }

                // Парсим элементы массива
                loop {
                    elements.push(self.parse_expression()?);

                    if matches!(self.current_token, Token::Comma) {
                        self.advance(); // consume ','
                        // Проверяем trailing comma
                        if matches!(self.current_token, Token::RightBracket) {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                self.expect(Token::RightBracket)?;
                Ok(Expr::ArrayLiteral { elements })
            }
            Token::Throw => {
                self.advance(); // consume 'throw'
                let message = Box::new(self.parse_expression()?);
                Ok(Expr::ThrowStatement { message })
            }
            _ => Err(DataCodeError::syntax_error(
                &format!("Unexpected token: {:?}", self.current_token),
                1, 0
            )),
        }
    }
}
