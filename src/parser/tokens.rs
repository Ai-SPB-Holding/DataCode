// Определения токенов и операторов для парсера DataCode

use crate::value::Value;

/// Токены лексического анализа
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
    Modulo,         // %
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
    EndTry,         // endtry
    Throw,          // throw

    // Функции
    Function,       // function
    Global,         // global
    Local,          // local
    Do,             // do
    EndFunction,    // endfunction
    Return,         // return
    
    // Скобки
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    LeftBrace,      // {
    RightBrace,     // }

    // Разделители
    Comma,          // ,
    Dot,            // .
    Colon,          // :
    
    // Специальные
    Assign,         // =
    Newline,
    EOF,
}

/// Узлы абстрактного синтаксического дерева (AST)
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
    ObjectLiteral {
        pairs: Vec<(String, Expr)>,
    },
    Spread {
        expression: Box<Expr>,
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

/// Бинарные операторы
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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

/// Унарные операторы
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Minus,
}

impl Token {
    /// Проверить, является ли токен литералом
    pub fn is_literal(&self) -> bool {
        matches!(self, Token::String(_) | Token::Number(_) | Token::Bool(_) | Token::Null)
    }
    
    /// Проверить, является ли токен оператором
    pub fn is_operator(&self) -> bool {
        matches!(self,
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Modulo |
            Token::Equal | Token::NotEqual | Token::Less | Token::Greater |
            Token::LessEqual | Token::GreaterEqual | Token::And | Token::Or | Token::Not
        )
    }
    
    /// Проверить, является ли токен ключевым словом
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            Token::And | Token::Or | Token::Not | Token::Try |
            Token::Catch | Token::Finally | Token::EndTry | Token::Throw |
            Token::Function | Token::Global | Token::Local | Token::Do |
            Token::EndFunction | Token::Return
        )
    }
}

impl BinaryOp {
    /// Получить приоритет оператора (чем больше число, тем выше приоритет)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Equal | BinaryOp::NotEqual => 3,
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual => 4,
            BinaryOp::Add | BinaryOp::Subtract => 5,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo | BinaryOp::PathJoin => 6,
        }
    }
    
    /// Проверить, является ли оператор левоассоциативным
    pub fn is_left_associative(&self) -> bool {
        // Все наши операторы левоассоциативны
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_is_literal() {
        assert!(Token::Number(42.0).is_literal());
        assert!(Token::String("hello".to_string()).is_literal());
        assert!(Token::Bool(true).is_literal());
        assert!(Token::Null.is_literal());
        assert!(!Token::Plus.is_literal());
    }

    #[test]
    fn test_token_is_operator() {
        assert!(Token::Plus.is_operator());
        assert!(Token::Equal.is_operator());
        assert!(Token::And.is_operator());
        assert!(!Token::Number(42.0).is_operator());
    }

    #[test]
    fn test_binary_op_precedence() {
        assert!(BinaryOp::Multiply.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::Add.precedence() > BinaryOp::Equal.precedence());
        assert!(BinaryOp::Equal.precedence() > BinaryOp::And.precedence());
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
    }

    #[test]
    fn test_binary_op_associativity() {
        assert!(BinaryOp::Add.is_left_associative());
        assert!(BinaryOp::Multiply.is_left_associative());
        assert!(BinaryOp::Equal.is_left_associative());
    }
}
