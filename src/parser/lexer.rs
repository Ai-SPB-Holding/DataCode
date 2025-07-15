// Лексический анализатор для DataCode
// Преобразует входной текст в последовательность токенов

use super::tokens::Token;

/// Лексический анализатор
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    /// Создать новый лексер для заданного входного текста
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Self {
            input: chars,
            position: 0,
            current_char,
        }
    }
    
    /// Перейти к следующему символу
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    /// Посмотреть на следующий символ без перехода к нему
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    /// Пропустить пробельные символы (кроме новой строки)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// Прочитать строковый литерал
    fn read_string(&mut self, quote_char: char) -> String {
        let mut result = String::new();
        self.advance(); // Skip opening quote

        while let Some(ch) = self.current_char {
            if ch == quote_char {
                self.advance(); // Skip closing quote
                break;
            }
            result.push(ch);
            self.advance();
        }

        result
    }
    
    /// Прочитать числовой литерал
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
    
    /// Прочитать идентификатор или ключевое слово
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

    /// Пропустить однострочный комментарий (начинается с #)
    fn skip_single_line_comment(&mut self) {
        // Пропускаем символ #
        self.advance();

        // Читаем до конца строки
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Пропустить многострочный комментарий (""" ... """)
    fn skip_multiline_comment(&mut self) {
        // Пропускаем первые """
        self.advance(); // первая "
        self.advance(); // вторая "
        self.advance(); // третья "

        // Ищем закрывающие """
        while let Some(ch) = self.current_char {
            if ch == '"' && self.peek() == Some('"') {
                // Проверяем, есть ли третья кавычка
                if self.input.get(self.position + 2) == Some(&'"') {
                    // Пропускаем закрывающие """
                    self.advance(); // первая "
                    self.advance(); // вторая "
                    self.advance(); // третья "
                    break;
                }
            }
            self.advance();
        }
    }
    
    /// Получить следующий токен
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
                Some('#') => {
                    self.skip_single_line_comment();
                    continue;
                }
                Some('"') => {
                    // Проверяем, начинается ли многострочный комментарий
                    if self.peek() == Some('"') && self.input.get(self.position + 2) == Some(&'"') {
                        self.skip_multiline_comment();
                        continue;
                    } else {
                        // Обычная строка в двойных кавычках
                        let string_val = self.read_string('"');
                        return Token::String(string_val);
                    }
                }
                Some('\'') => {
                    let string_val = self.read_string('\'');
                    return Token::String(string_val);
                }
                Some(ch) if ch.is_ascii_digit() => {
                    let num = self.read_number();
                    return Token::Number(num);
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let ident = self.read_identifier();
                    return self.keyword_or_identifier(ident);
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
                    return Token::Divide;
                }
                Some('%') => {
                    self.advance();
                    return Token::Modulo;
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
                Some('{') => {
                    self.advance();
                    return Token::LeftBrace;
                }
                Some('}') => {
                    self.advance();
                    return Token::RightBrace;
                }
                Some(',') => {
                    self.advance();
                    return Token::Comma;
                }
                Some('.') => {
                    self.advance();
                    return Token::Dot;
                }
                Some(':') => {
                    self.advance();
                    return Token::Colon;
                }
                Some(_ch) => {
                    self.advance();
                    // Пропускаем неизвестные символы
                    continue;
                }
            }
        }
    }
    
    /// Определить, является ли идентификатор ключевым словом
    fn keyword_or_identifier(&self, ident: String) -> Token {
        match ident.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::Null,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "finally" => Token::Finally,
            "endtry" => Token::EndTry,
            "throw" => Token::Throw,
            "function" => Token::Function,
            "global" => Token::Global,
            "local" => Token::Local,
            "do" => Token::Do,
            "endfunction" => Token::EndFunction,
            "return" => Token::Return,
            _ => Token::Identifier(ident),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_numbers() {
        let mut lexer = Lexer::new("42 3.14");

        assert_eq!(lexer.next_token(), Token::Number(42.0));
        assert_eq!(lexer.next_token(), Token::Number(3.14));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_strings() {
        let mut lexer = Lexer::new("'hello' 'world' \"test\" \"double\"");

        assert_eq!(lexer.next_token(), Token::String("hello".to_string()));
        assert_eq!(lexer.next_token(), Token::String("world".to_string()));
        assert_eq!(lexer.next_token(), Token::String("test".to_string()));
        assert_eq!(lexer.next_token(), Token::String("double".to_string()));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("+ - * / == != < > <= >=");

        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Multiply);
        assert_eq!(lexer.next_token(), Token::Divide);
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::NotEqual);
        assert_eq!(lexer.next_token(), Token::Less);
        assert_eq!(lexer.next_token(), Token::Greater);
        assert_eq!(lexer.next_token(), Token::LessEqual);
        assert_eq!(lexer.next_token(), Token::GreaterEqual);
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("true false null and or not");

        assert_eq!(lexer.next_token(), Token::Bool(true));
        assert_eq!(lexer.next_token(), Token::Bool(false));
        assert_eq!(lexer.next_token(), Token::Null);
        assert_eq!(lexer.next_token(), Token::And);
        assert_eq!(lexer.next_token(), Token::Or);
        assert_eq!(lexer.next_token(), Token::Not);
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = Lexer::new("variable_name another_var");

        assert_eq!(lexer.next_token(), Token::Identifier("variable_name".to_string()));
        assert_eq!(lexer.next_token(), Token::Identifier("another_var".to_string()));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_complex_expression() {
        let mut lexer = Lexer::new("x + 42 == 'hello'");

        assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Number(42.0));
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::String("hello".to_string()));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_single_line_comments() {
        let mut lexer = Lexer::new("# Это комментарий\nglobal x = 42");

        assert_eq!(lexer.next_token(), Token::Newline);
        assert_eq!(lexer.next_token(), Token::Global);
        assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Number(42.0));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_multiline_comments() {
        let mut lexer = Lexer::new("\"\"\"Это многострочный комментарий\"\"\"\nglobal x = 42");

        assert_eq!(lexer.next_token(), Token::Newline);
        assert_eq!(lexer.next_token(), Token::Global);
        assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
        assert_eq!(lexer.next_token(), Token::Assign);
        assert_eq!(lexer.next_token(), Token::Number(42.0));
        assert_eq!(lexer.next_token(), Token::EOF);
    }
}
