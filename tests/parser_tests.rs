use data_code::parser::{Parser, Lexer, Token, Expr, BinaryOp, UnaryOp};
use data_code::value::Value;
use data_code::evaluator::parse_and_evaluate;
use std::collections::HashMap;

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let mut lexer = Lexer::new("123 + 'hello'");
        
        assert_eq!(lexer.next_token(), Token::Number(123.0));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::String("hello".to_string()));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_tokenize_identifiers() {
        let mut lexer = Lexer::new("variable_name function_call()");
        
        assert_eq!(lexer.next_token(), Token::Identifier("variable_name".to_string()));
        assert_eq!(lexer.next_token(), Token::Identifier("function_call".to_string()));
        assert_eq!(lexer.next_token(), Token::LeftParen);
        assert_eq!(lexer.next_token(), Token::RightParen);
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_tokenize_operators() {
        let mut lexer = Lexer::new("== != < > <= >= and or not");
        
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::NotEqual);
        assert_eq!(lexer.next_token(), Token::Less);
        assert_eq!(lexer.next_token(), Token::Greater);
        assert_eq!(lexer.next_token(), Token::LessEqual);
        assert_eq!(lexer.next_token(), Token::GreaterEqual);
        assert_eq!(lexer.next_token(), Token::And);
        assert_eq!(lexer.next_token(), Token::Or);
        assert_eq!(lexer.next_token(), Token::Not);
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_tokenize_booleans() {
        let mut lexer = Lexer::new("true false");
        
        assert_eq!(lexer.next_token(), Token::Bool(true));
        assert_eq!(lexer.next_token(), Token::Bool(false));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_tokenize_path_join() {
        let mut lexer = Lexer::new("path / 'subdir'");

        assert_eq!(lexer.next_token(), Token::Identifier("path".to_string()));
        assert_eq!(lexer.next_token(), Token::Divide); // Теперь лексер генерирует Divide
        assert_eq!(lexer.next_token(), Token::String("subdir".to_string()));
        assert_eq!(lexer.next_token(), Token::EOF);
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_literal() {
        let mut parser = Parser::new("42");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Literal(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let mut parser = Parser::new("'hello world'");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Literal(Value::String(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_variable() {
        let mut parser = Parser::new("my_var");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Variable(name) => assert_eq!(name, "my_var"),
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_parse_binary_addition() {
        let mut parser = Parser::new("a + b");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Binary { left, operator, right } => {
                assert_eq!(operator, BinaryOp::Add);
                match (*left, *right) {
                    (Expr::Variable(l), Expr::Variable(r)) => {
                        assert_eq!(l, "a");
                        assert_eq!(r, "b");
                    }
                    _ => panic!("Expected variables in binary expression"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_path_join() {
        let mut parser = Parser::new("base / 'subdir'");
        let expr = parser.parse_expression().unwrap();

        match expr {
            Expr::Binary { left, operator, right } => {
                assert_eq!(operator, BinaryOp::Divide); // Теперь парсер генерирует Divide
                match (*left, *right) {
                    (Expr::Variable(l), Expr::Literal(Value::String(r))) => {
                        assert_eq!(l, "base");
                        assert_eq!(r, "subdir");
                    }
                    _ => panic!("Expected variable and string in path join"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let mut parser = Parser::new("func(a, 'hello', 42)");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "func");
                assert_eq!(args.len(), 3);
                
                match &args[0] {
                    Expr::Variable(v) => assert_eq!(v, "a"),
                    _ => panic!("Expected variable as first argument"),
                }
                
                match &args[1] {
                    Expr::Literal(Value::String(s)) => assert_eq!(s, "hello"),
                    _ => panic!("Expected string as second argument"),
                }
                
                match &args[2] {
                    Expr::Literal(Value::Number(n)) => assert_eq!(*n, 42.0),
                    _ => panic!("Expected number as third argument"),
                }
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let mut parser = Parser::new("x > 10");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Binary { left, operator, right } => {
                assert_eq!(operator, BinaryOp::Greater);
                match (*left, *right) {
                    (Expr::Variable(l), Expr::Literal(Value::Number(r))) => {
                        assert_eq!(l, "x");
                        assert_eq!(r, 10.0);
                    }
                    _ => panic!("Expected variable and number in comparison"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_logical_and() {
        let mut parser = Parser::new("a and b");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Binary { left, operator, right } => {
                assert_eq!(operator, BinaryOp::And);
                match (*left, *right) {
                    (Expr::Variable(l), Expr::Variable(r)) => {
                        assert_eq!(l, "a");
                        assert_eq!(r, "b");
                    }
                    _ => panic!("Expected variables in logical expression"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_unary_not() {
        let mut parser = Parser::new("not x");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Unary { operator, operand } => {
                assert_eq!(operator, UnaryOp::Not);
                match *operand {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected variable in unary expression"),
                }
            }
            _ => panic!("Expected unary expression"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let mut parser = Parser::new("(a + b) * c");
        let expr = parser.parse_expression().unwrap();
        
        match expr {
            Expr::Binary { left, operator, right } => {
                assert_eq!(operator, BinaryOp::Multiply);
                match (*left, *right) {
                    (Expr::Binary { operator: BinaryOp::Add, .. }, Expr::Variable(r)) => {
                        assert_eq!(r, "c");
                    }
                    _ => panic!("Expected addition in parentheses and variable"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }
}

#[cfg(test)]
mod evaluator_tests {
    use super::*;

    fn create_test_variables() -> HashMap<String, Value> {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), Value::Number(10.0));
        vars.insert("y".to_string(), Value::Number(5.0));
        vars.insert("name".to_string(), Value::String("test".to_string()));
        vars.insert("flag".to_string(), Value::Bool(true));
        vars.insert("arr".to_string(), Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]));
        vars
    }

    #[test]
    fn test_evaluate_arithmetic() {
        let vars = create_test_variables();
        
        let result = parse_and_evaluate("x + y", &vars, 1).unwrap();
        assert_eq!(result, Value::Number(15.0));
        
        let result = parse_and_evaluate("x - y", &vars, 1).unwrap();
        assert_eq!(result, Value::Number(5.0));
        
        let result = parse_and_evaluate("x * y", &vars, 1).unwrap();
        assert_eq!(result, Value::Number(50.0));
        
        let result = parse_and_evaluate("x / y", &vars, 1).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_evaluate_comparison() {
        let vars = create_test_variables();
        
        let result = parse_and_evaluate("x > y", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = parse_and_evaluate("x < y", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(false));
        
        let result = parse_and_evaluate("x == 10", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = parse_and_evaluate("x != y", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_logical() {
        let vars = create_test_variables();
        
        let result = parse_and_evaluate("flag and true", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = parse_and_evaluate("flag or false", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = parse_and_evaluate("not flag", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_evaluate_string_operations() {
        let vars = create_test_variables();
        
        let result = parse_and_evaluate("name + '_suffix'", &vars, 1).unwrap();
        assert_eq!(result, Value::String("test_suffix".to_string()));
        
        let result = parse_and_evaluate("'prefix_' + name", &vars, 1).unwrap();
        assert_eq!(result, Value::String("prefix_test".to_string()));
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let vars = create_test_variables();
        
        let result = parse_and_evaluate("(x + y) * 2 > 20", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = parse_and_evaluate("flag and (x > y)", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_function_call() {
        let vars = HashMap::new();
        
        let result = parse_and_evaluate("now()", &vars, 1).unwrap();
        match result {
            Value::String(_) => {}, // now() возвращает строку с временем
            _ => panic!("Expected string from now()"),
        }
    }

    #[test]
    fn test_evaluate_error_cases() {
        let vars = HashMap::new();
        
        // Неизвестная переменная
        let result = parse_and_evaluate("unknown_var", &vars, 1);
        assert!(result.is_err());
        
        // Деление на ноль
        let result = parse_and_evaluate("10 / 0", &vars, 1);
        assert!(result.is_err());
        
        // Неправильные типы для операции
        let mut vars = HashMap::new();
        vars.insert("str".to_string(), Value::String("hello".to_string()));
        let result = parse_and_evaluate("str - 5", &vars, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_operator_precedence() {
        let vars = create_test_variables();
        
        // 2 + 3 * 4 должно быть 14, не 20
        let result = parse_and_evaluate("2 + 3 * 4", &vars, 1).unwrap();
        assert_eq!(result, Value::Number(14.0));
        
        // (2 + 3) * 4 должно быть 20
        let result = parse_and_evaluate("(2 + 3) * 4", &vars, 1).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_boolean_conversion() {
        let mut vars = HashMap::new();
        vars.insert("zero".to_string(), Value::Number(0.0));
        vars.insert("empty_str".to_string(), Value::String("".to_string()));
        vars.insert("empty_arr".to_string(), Value::Array(vec![]));
        
        let result = parse_and_evaluate("zero or 1", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = parse_and_evaluate("empty_str and 'hello'", &vars, 1).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_intelligent_divide_operator() {
        use std::path::PathBuf;
        let mut vars = HashMap::new();

        // Тест математического деления
        let result = parse_and_evaluate("10 / 2", &vars, 1).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Тест деления на ноль
        let result = parse_and_evaluate("10 / 0", &vars, 1);
        assert!(result.is_err());

        // Тест объединения путей с переменной Path
        vars.insert("base_path".to_string(), Value::Path(PathBuf::from("/home/user")));
        let result = parse_and_evaluate("base_path / 'documents'", &vars, 1).unwrap();
        match result {
            Value::Path(p) => {
                assert_eq!(p.to_string_lossy(), "/home/user/documents");
            }
            _ => panic!("Expected Path value"),
        }
    }
}
