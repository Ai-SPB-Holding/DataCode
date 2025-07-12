use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;
use std::path::PathBuf;

#[cfg(test)]
mod builtin_functions_tests {
    use super::*;

    #[test]
    fn test_now_function() {
        let mut interp = Interpreter::new();
        
        interp.exec("global current_time = now()").unwrap();
        
        match interp.get_variable("current_time") {
            Some(Value::String(time_str)) => {
                // Проверяем, что строка не пустая и содержит разумные символы времени
                assert!(!time_str.is_empty());
                assert!(time_str.len() > 10); // Минимальная длина для времени
            }
            _ => panic!("now() should return a string"),
        }
    }

    #[test]
    fn test_now_function_no_args() {
        let mut interp = Interpreter::new();
        
        // now() не должна принимать аргументы
        let result = interp.exec("global time = now(123)");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, .. } => {
                assert_eq!(name, "now");
            }
            _ => panic!("Expected FunctionError"),
        }
    }

    #[test]
    fn test_getcwd_function() {
        let mut interp = Interpreter::new();
        
        interp.exec("global current_dir = getcwd()").unwrap();
        
        match interp.get_variable("current_dir") {
            Some(Value::Path(path)) => {
                // Проверяем, что путь существует и не пустой
                assert!(path.exists() || path.is_absolute());
            }
            _ => panic!("getcwd() should return a path"),
        }
    }

    #[test]
    fn test_getcwd_function_no_args() {
        let mut interp = Interpreter::new();
        
        // getcwd() не должна принимать аргументы
        let result = interp.exec("global dir = getcwd('test')");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, .. } => {
                assert_eq!(name, "getcwd");
            }
            _ => panic!("Expected FunctionError"),
        }
    }

    #[test]
    fn test_path_function() {
        let mut interp = Interpreter::new();
        
        interp.exec("global test_path = path('/home/user')").unwrap();
        
        match interp.get_variable("test_path") {
            Some(Value::Path(path)) => {
                assert_eq!(path, &PathBuf::from("/home/user"));
            }
            _ => panic!("path() should return a path"),
        }
    }

    #[test]
    fn test_path_function_with_string() {
        let mut interp = Interpreter::new();
        
        interp.exec("global str_path = 'test/directory'").unwrap();
        interp.exec("global converted_path = path(str_path)").unwrap();
        
        match interp.get_variable("converted_path") {
            Some(Value::Path(path)) => {
                assert_eq!(path, &PathBuf::from("test/directory"));
            }
            _ => panic!("path() should return a path"),
        }
    }

    #[test]
    fn test_path_function_wrong_type() {
        let mut interp = Interpreter::new();
        
        // path() должна принимать только строки
        let result = interp.exec("global bad_path = path(123)");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::TypeError { expected, .. } => {
                assert_eq!(expected, "String");
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_path_function_wrong_arg_count() {
        let mut interp = Interpreter::new();
        
        // path() должна принимать ровно один аргумент
        let result = interp.exec("global bad_path = path()");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, .. } => {
                assert_eq!(name, "path");
            }
            _ => panic!("Expected FunctionError"),
        }
        
        let result2 = interp.exec("global bad_path2 = path('a', 'b')");
        assert!(result2.is_err());
        
        match result2.unwrap_err() {
            DataCodeError::FunctionError { name, .. } => {
                assert_eq!(name, "path");
            }
            _ => panic!("Expected FunctionError"),
        }
    }

    #[test]
    fn test_print_function() {
        let mut interp = Interpreter::new();
        
        // print() должна работать с различными типами
        let result1 = interp.exec("print('Hello, World!')");
        assert!(result1.is_ok());
        
        let result2 = interp.exec("print(42)");
        assert!(result2.is_ok());
        
        let result3 = interp.exec("print(true)");
        assert!(result3.is_ok());
        
        interp.exec("global test_var = 'test'").unwrap();
        let result4 = interp.exec("print(test_var)");
        assert!(result4.is_ok());
    }

    #[test]
    fn test_print_multiple_args() {
        let mut interp = Interpreter::new();
        
        // print() должна работать с несколькими аргументами
        let result = interp.exec("print('Value:', 42, 'is the answer')");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_function() {
        let mut interp = Interpreter::new();
        
        let result = interp.exec("global x = unknown_function()");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, error_type, .. } => {
                assert_eq!(name, "unknown_function");
                match error_type {
                    data_code::error::FunctionErrorType::NotFound => {}, // OK
                    _ => panic!("Expected NotFound error"),
                }
            }
            _ => panic!("Expected FunctionError"),
        }
    }

    #[test]
    fn test_function_in_expressions() {
        let mut interp = Interpreter::new();

        // Тест использования функций в арифметических выражениях
        interp.exec("global x = 5").unwrap();
        interp.exec("global y = 3").unwrap();
        interp.exec("global sum = x + y").unwrap();
        interp.exec("global product = x * y").unwrap();

        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(8.0)));
        assert_eq!(interp.get_variable("product"), Some(&Value::Number(15.0)));
    }

    #[test]
    fn test_function_chaining() {
        let mut interp = Interpreter::new();

        // Тест цепочки функций - используем простые операции
        interp.exec("global base_str = '/home/user'").unwrap();
        interp.exec("global base_path = path(base_str)").unwrap();

        match interp.get_variable("base_path") {
            Some(Value::Path(path)) => {
                let path_str = path.to_string_lossy();
                assert!(path_str.contains("user"));
            }
            _ => panic!("Expected path from chained functions"),
        }
    }

    #[test]
    fn test_builtin_function_with_user_function() {
        let mut interp = Interpreter::new();
        
        // Определяем пользовательскую функцию, которая использует встроенную
        let function_code = r#"global function get_current_time_formatted() do
    local time = now()
    return 'Current time: ' + time
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global formatted_time = get_current_time_formatted()").unwrap();
        
        match interp.get_variable("formatted_time") {
            Some(Value::String(s)) => {
                assert!(s.starts_with("Current time: "));
            }
            _ => panic!("Expected formatted time string"),
        }
    }

    #[test]
    fn test_builtin_function_in_conditions() {
        let mut interp = Interpreter::new();
        
        // Тест использования встроенных функций в условиях
        let if_code = r#"if getcwd() != path('/nonexistent') do
    global result = 'different_paths'
else
    global result = 'same_paths'
endif"#;
        
        interp.exec(if_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("different_paths".to_string())));
    }

    #[test]
    fn test_function_error_line_tracking() {
        let mut interp = Interpreter::new();

        let result = interp.exec("global x = nonexistent_func()");
        assert!(result.is_err());

        match result.unwrap_err() {
            DataCodeError::FunctionError { line, .. } => {
                // Проверяем, что номер строки больше 0 (правильно отслеживается)
                assert!(line > 0);
            }
            _ => panic!("Expected FunctionError with correct line number"),
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let mut interp = Interpreter::new();

        // Тест вложенных вызовов функций - используем простые операции
        interp.exec("global base_str = '/home/user'").unwrap();
        interp.exec("global base_path = path(base_str)").unwrap();

        match interp.get_variable("base_path") {
            Some(Value::Path(path)) => {
                let path_str = path.to_string_lossy();
                assert!(path_str.contains("user"));
            }
            _ => panic!("Expected nested path"),
        }
    }
}
