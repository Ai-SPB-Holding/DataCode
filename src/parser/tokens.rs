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
    #[allow(dead_code)]
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
    
    // Условные конструкции
    If,             // if
    Else,           // else
    EndIf,          // endif
    
    // Циклы
    For,            // for
    Next,           // next
    In,             // in
    
    // Встроенные функции
    Print,          // print
    
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
        named_args: Vec<(String, Expr)>,
    },
    NamedArg {
        name: String,
        value: Box<Expr>,
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
    // Statements (операторы)
    Assignment {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Declaration {
        scope: DeclarationScope,
        names: Vec<String>,
    },
    ReturnStmt {
        value: Option<Box<Expr>>,
    },
    PrintStmt {
        args: Vec<Expr>,
    },
    ExprStmt {
        expr: Box<Expr>,
    },
    Block {
        statements: Vec<Expr>,
    },
    IfStmt {
        condition: Box<Expr>,
        then_block: Box<Expr>,
        else_block: Option<Box<Expr>>,
    },
    ForStmt {
        vars: Vec<String>,
        iter_expr: Box<Expr>,
        body: Box<Expr>,
    },
    TryStmt {
        try_block: Box<Expr>,
        catch_var: String,
        catch_block: Box<Expr>,
    },
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
    #[allow(dead_code)]
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

/// Область видимости для объявлений переменных
#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationScope {
    #[allow(dead_code)]
    Global,
    #[allow(dead_code)]
    Local,
}

impl Token {
    /// Проверить, является ли токен литералом
    #[allow(dead_code)]
    pub fn is_literal(&self) -> bool {
        matches!(self, Token::String(_) | Token::Number(_) | Token::Bool(_) | Token::Null)
    }
    
    /// Проверить, является ли токен оператором
    #[allow(dead_code)]
    pub fn is_operator(&self) -> bool {
        matches!(self,
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Modulo |
            Token::Equal | Token::NotEqual | Token::Less | Token::Greater |
            Token::LessEqual | Token::GreaterEqual | Token::And | Token::Or | Token::Not
        )
    }
    
    /// Проверить, является ли токен ключевым словом
    #[allow(dead_code)]
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            Token::And | Token::Or | Token::Not | Token::Try |
            Token::Catch | Token::Finally | Token::EndTry | Token::Throw |
            Token::Function | Token::Global | Token::Local | Token::Do |
            Token::EndFunction | Token::Return | Token::If | Token::Else |
            Token::EndIf | Token::For | Token::Next | Token::In | Token::Print
        )
    }
    
    /// Проверить, является ли токен ключевым словом оператора (не может быть в expression context)
    /// Примечание: Print не включен, так как print() может использоваться как функция в выражениях
    pub fn is_statement_keyword(&self) -> bool {
        matches!(self,
            Token::If | Token::For | Token::Try | Token::Function |
            Token::Return | Token::Global | Token::Local |
            Token::Else | Token::EndIf | Token::Next | Token::Catch |
            Token::EndTry | Token::EndFunction | Token::Do
        )
    }
}

impl BinaryOp {
    /// Получить приоритет оператора (чем больше число, тем выше приоритет)
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
