use data_code::error::{DataCodeError, VariableErrorType, FunctionErrorType};
use data_code::interpreter::Interpreter;
use data_code::evaluator::parse_and_evaluate;
use std::collections::HashMap;

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_variable_not_found_error() {
        let mut interp = Interpreter::new();

        let result = interp.exec("global x = unknown_variable");
        assert!(result.is_err());

        match result.unwrap_err() {
            DataCodeError::VariableError { name, error_type, line } => {
                assert_eq!(name, "unknown_variable");
                assert_eq!(error_type, VariableErrorType::NotFound);
                assert!(line > 0); // Проверяем, что номер строки больше 0
            }
            _ => panic!("Expected VariableError"),
        }
    }

    #[test]
    fn test_function_not_found_error() {
        let vars = HashMap::new();
        
        let result = parse_and_evaluate("unknown_function()", &vars, 5);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, error_type, line } => {
                assert_eq!(name, "unknown_function");
                assert_eq!(error_type, FunctionErrorType::NotFound);
                assert_eq!(line, 5);
            }
            _ => panic!("Expected FunctionError"),
        }
    }

    #[test]
    fn test_wrong_argument_count_error() {
        let vars = HashMap::new();
        
        // now() не принимает аргументы
        let result = parse_and_evaluate("now(123)", &vars, 3);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, error_type, line } => {
                assert_eq!(name, "now");
                match error_type {
                    FunctionErrorType::WrongArgumentCount { expected, found } => {
                        assert_eq!(expected, 0);
                        assert_eq!(found, 1);
                    }
                    _ => panic!("Expected WrongArgumentCount"),
                }
                assert_eq!(line, 3);
            }
            _ => panic!("Expected FunctionError"),
        }
    }

    #[test]
    fn test_type_error() {
        let mut vars = HashMap::new();
        vars.insert("text".to_string(), data_code::value::Value::String("hello".to_string()));
        vars.insert("number".to_string(), data_code::value::Value::Number(42.0));
        
        // Попытка вычесть число из строки
        let result = parse_and_evaluate("text - number", &vars, 7);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::TypeError { expected, found, line } => {
                assert_eq!(expected, "Number");
                assert_eq!(found, "other");
                assert_eq!(line, 7);
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_division_by_zero_error() {
        let mut vars = HashMap::new();
        vars.insert("zero".to_string(), data_code::value::Value::Number(0.0));

        let result = parse_and_evaluate("10 / zero", &vars, 2);
        assert!(result.is_err());

        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, line } => {
                assert!(message.contains("Division by zero"));
                assert_eq!(line, 2);
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_syntax_error() {
        let mut interp = Interpreter::new();

        // Неправильный синтаксис присваивания - отсутствует знак равенства
        let result = interp.exec("global x y");
        assert!(result.is_err());

        match result.unwrap_err() {
            DataCodeError::SyntaxError { message, line, column } => {
                assert!(message.contains("Invalid assignment"));
                assert!(line > 0); // Проверяем, что номер строки больше 0
                assert_eq!(column, 0);
            }
            _ => panic!("Expected SyntaxError"),
        }
    }

    #[test]
    fn test_expression_error() {
        let vars = HashMap::new();
        
        // Неправильный синтаксис выражения
        let result = parse_and_evaluate("(((", &vars, 4);
        assert!(result.is_err());
        
        // Проверяем, что получили какую-то ошибку парсинга
        match result.unwrap_err() {
            DataCodeError::SyntaxError { .. } => {}, // OK
            DataCodeError::ExpressionError { .. } => {}, // OK
            _ => panic!("Expected syntax or expression error"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = DataCodeError::variable_not_found("my_var", 10);
        let error_string = format!("{}", error);
        
        assert!(error_string.contains("Variable Error"));
        assert!(error_string.contains("line 10"));
        assert!(error_string.contains("my_var"));
        assert!(error_string.contains("not found"));
    }

    #[test]
    fn test_error_display_function() {
        let error = DataCodeError::wrong_argument_count("test_func", 2, 3, 15);
        let error_string = format!("{}", error);
        
        assert!(error_string.contains("Function Error"));
        assert!(error_string.contains("line 15"));
        assert!(error_string.contains("test_func"));
        assert!(error_string.contains("expects 2"));
        assert!(error_string.contains("found 3"));
    }

    #[test]
    fn test_error_display_type() {
        let error = DataCodeError::type_error("String", "Number", 8);
        let error_string = format!("{}", error);
        
        assert!(error_string.contains("Type Error"));
        assert!(error_string.contains("line 8"));
        assert!(error_string.contains("expected String"));
        assert!(error_string.contains("found Number"));
    }

    #[test]
    fn test_error_display_runtime() {
        let error = DataCodeError::runtime_error("Something went wrong", 12);
        let error_string = format!("{}", error);
        
        assert!(error_string.contains("Runtime Error"));
        assert!(error_string.contains("line 12"));
        assert!(error_string.contains("Something went wrong"));
    }

    #[test]
    fn test_error_display_syntax() {
        let error = DataCodeError::syntax_error("Unexpected token", 5, 10);
        let error_string = format!("{}", error);
        
        assert!(error_string.contains("Syntax Error"));
        assert!(error_string.contains("line 5"));
        assert!(error_string.contains("column 10"));
        assert!(error_string.contains("Unexpected token"));
    }

    #[test]
    fn test_nested_expression_errors() {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), data_code::value::Value::Number(10.0));
        
        // Ошибка во вложенном выражении
        let result = parse_and_evaluate("x + unknown_var * 2", &vars, 6);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::VariableError { name, line, .. } => {
                assert_eq!(name, "unknown_var");
                assert_eq!(line, 6);
            }
            _ => panic!("Expected VariableError"),
        }
    }

    #[test]
    fn test_function_call_errors() {
        let vars = HashMap::new();
        
        // Функция с неправильным типом аргумента
        let result = parse_and_evaluate("path(123)", &vars, 9);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::TypeError { expected, found, line } => {
                assert_eq!(expected, "String");
                assert_eq!(found, "other");
                assert_eq!(line, 9);
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_for_loop_errors() {
        let mut interp = Interpreter::new();
        
        // Неправильный синтаксис for loop
        let result = interp.exec("for x y do\nprint(x)\nnext x");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::SyntaxError { message, .. } => {
                assert!(message.contains("Invalid for syntax"));
            }
            _ => panic!("Expected SyntaxError"),
        }
        
        // Отсутствующий next
        let result = interp.exec("for x in arr do\nprint(x)");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::SyntaxError { message, .. } => {
                assert!(message.contains("Missing next") || message.contains("Missing 'next"), 
                    "Expected message containing 'Missing next', got: '{}'", message);
            }
            e => panic!("Expected SyntaxError, got: {:?}", e),
        }
    }

    #[test]
    fn test_error_line_tracking() {
        let mut interp = Interpreter::new();

        let result = interp.exec("global x = nonexistent");
        assert!(result.is_err());

        match result.unwrap_err() {
            DataCodeError::VariableError { line, .. } => {
                assert!(line > 0); // Проверяем, что номер строки больше 0
            }
            _ => panic!("Expected VariableError with correct line number"),
        }
    }
}
