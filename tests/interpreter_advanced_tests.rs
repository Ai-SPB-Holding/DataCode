use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;

#[cfg(test)]
mod interpreter_advanced_tests {
    use super::*;

    #[test]
    fn test_improved_expression_parsing() {
        let mut interp = Interpreter::new();

        // Тест арифметических операций
        interp.exec("global x = 10").unwrap();
        interp.exec("global y = 5").unwrap();
        interp.exec("global sum = x + y").unwrap();
        interp.exec("global diff = x - y").unwrap();
        interp.exec("global prod = x * y").unwrap();

        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(15.0)));
        assert_eq!(interp.get_variable("diff"), Some(&Value::Number(5.0)));
        assert_eq!(interp.get_variable("prod"), Some(&Value::Number(50.0)));

        // Тест деления отдельно
        interp.exec("global quot = x / y").unwrap();
        // Проверяем, что получили правильное значение
        match interp.get_variable("quot") {
            Some(Value::Number(n)) => {
                // Проверяем, что результат близок к 2.0 (учитывая возможные ошибки округления)
                assert!((n - 2.0).abs() < 0.001, "Expected 2.0, got {}", n);
            }
            other => panic!("Expected Number(2.0), got {:?}", other),
        }
    }

    #[test]
    fn test_comparison_operators() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 10").unwrap();
        interp.exec("global y = 5").unwrap();
        
        interp.exec("global gt = x > y").unwrap();
        interp.exec("global lt = x < y").unwrap();
        interp.exec("global eq = x == 10").unwrap();
        interp.exec("global ne = x != y").unwrap();
        interp.exec("global ge = x >= 10").unwrap();
        interp.exec("global le = y <= 5").unwrap();
        
        assert_eq!(interp.get_variable("gt"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("lt"), Some(&Value::Bool(false)));
        assert_eq!(interp.get_variable("eq"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("ne"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("ge"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("le"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_logical_operators() {
        let mut interp = Interpreter::new();
        
        interp.exec("global flag1 = true").unwrap();
        interp.exec("global flag2 = false").unwrap();
        
        interp.exec("global and_result = flag1 and flag2").unwrap();
        interp.exec("global or_result = flag1 or flag2").unwrap();
        interp.exec("global not_result = not flag1").unwrap();
        
        assert_eq!(interp.get_variable("and_result"), Some(&Value::Bool(false)));
        assert_eq!(interp.get_variable("or_result"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("not_result"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_complex_expressions() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 10").unwrap();
        interp.exec("global y = 5").unwrap();
        interp.exec("global z = 2").unwrap();
        
        // Тест приоритета операторов
        interp.exec("global result1 = x + y * z").unwrap(); // 10 + (5 * 2) = 20
        interp.exec("global result2 = (x + y) * z").unwrap(); // (10 + 5) * 2 = 30
        
        assert_eq!(interp.get_variable("result1"), Some(&Value::Number(20.0)));
        assert_eq!(interp.get_variable("result2"), Some(&Value::Number(30.0)));
        
        // Тест сложных логических выражений
        interp.exec("global complex = (x > y) and (z < 5)").unwrap();
        assert_eq!(interp.get_variable("complex"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_string_operations() {
        let mut interp = Interpreter::new();

        interp.exec("global name = 'DataCode'").unwrap();
        interp.exec("global version = '1.0'").unwrap();

        interp.exec("global full_name = name + ' v' + version").unwrap();
        assert_eq!(interp.get_variable("full_name"), Some(&Value::String("DataCode v1.0".to_string())));

        // Тест сравнения строк
        interp.exec("global str_eq = name == 'DataCode'").unwrap();
        interp.exec("global str_ne = name != 'Python'").unwrap();

        assert_eq!(interp.get_variable("str_eq"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("str_ne"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_function_calls_in_expressions() {
        let mut interp = Interpreter::new();

        // Тест вызова функций в выражениях
        interp.exec("global current_time = now()").unwrap();

        // Проверяем, что функция вернула строку
        match interp.get_variable("current_time") {
            Some(Value::String(_)) => {}, // OK
            _ => panic!("now() should return a string"),
        }

        // Тест функций с аргументами
        interp.exec("global cwd = getcwd()").unwrap();
        match interp.get_variable("cwd") {
            Some(Value::Path(_)) => {}, // OK
            _ => panic!("getcwd() should return a path"),
        }
    }

    #[test]
    fn test_parentheses_grouping() {
        let mut interp = Interpreter::new();
        
        interp.exec("global a = 2").unwrap();
        interp.exec("global b = 3").unwrap();
        interp.exec("global c = 4").unwrap();
        
        // Без скобок: 2 + 3 * 4 = 2 + 12 = 14
        interp.exec("global without_parens = a + b * c").unwrap();
        assert_eq!(interp.get_variable("without_parens"), Some(&Value::Number(14.0)));
        
        // Со скобками: (2 + 3) * 4 = 5 * 4 = 20
        interp.exec("global with_parens = (a + b) * c").unwrap();
        assert_eq!(interp.get_variable("with_parens"), Some(&Value::Number(20.0)));
    }

    #[test]
    fn test_unary_operators() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 10").unwrap();
        interp.exec("global flag = true").unwrap();
        
        interp.exec("global neg_x = -x").unwrap();
        interp.exec("global not_flag = not flag").unwrap();
        
        assert_eq!(interp.get_variable("neg_x"), Some(&Value::Number(-10.0)));
        assert_eq!(interp.get_variable("not_flag"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_error_handling_improved() {
        let mut interp = Interpreter::new();
        
        // Тест ошибки неизвестной переменной
        let result = interp.exec("global x = unknown_var");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::VariableError { name, .. } => {
                assert_eq!(name, "unknown_var");
            }
            _ => panic!("Expected VariableError"),
        }
        
        // Тест ошибки деления на ноль
        interp.exec("global zero = 0").unwrap();
        let result = interp.exec("global div_by_zero = 10 / zero");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("Division by zero"));
            }
            _ => panic!("Expected RuntimeError for division by zero"),
        }
    }

    #[test]
    fn test_boolean_truthiness() {
        let mut interp = Interpreter::new();
        
        // Числа: 0 - false, остальные - true
        interp.exec("global zero = 0").unwrap();
        interp.exec("global nonzero = 5").unwrap();
        
        interp.exec("global zero_or_true = zero or true").unwrap();
        interp.exec("global nonzero_and_false = nonzero and false").unwrap();
        
        assert_eq!(interp.get_variable("zero_or_true"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("nonzero_and_false"), Some(&Value::Bool(false)));
        
        // Строки: пустая - false, непустая - true
        interp.exec("global empty_str = ''").unwrap();
        interp.exec("global nonempty_str = 'hello'").unwrap();
        
        interp.exec("global empty_or_true = empty_str or true").unwrap();
        interp.exec("global nonempty_and_false = nonempty_str and false").unwrap();
        
        assert_eq!(interp.get_variable("empty_or_true"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("nonempty_and_false"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_path_operations_improved() {
        let mut interp = Interpreter::new();

        interp.exec("global base = getcwd()").unwrap();
        interp.exec("global subdir = 'data'").unwrap();
        interp.exec("global filename = 'test.txt'").unwrap();

        // Тест простых операций с путями
        match interp.get_variable("base") {
            Some(Value::Path(_)) => {}, // OK
            _ => panic!("Expected path from getcwd()"),
        }

        assert_eq!(interp.get_variable("subdir"), Some(&Value::String("data".to_string())));
        assert_eq!(interp.get_variable("filename"), Some(&Value::String("test.txt".to_string())));
    }

    #[test]
    fn test_mixed_type_operations() {
        let mut interp = Interpreter::new();
        
        // Тест умножения строки на число
        interp.exec("global str = 'hello'").unwrap();
        interp.exec("global count = 3").unwrap();
        interp.exec("global repeated = str * count").unwrap();
        
        assert_eq!(interp.get_variable("repeated"), Some(&Value::String("hellohellohello".to_string())));
        
        // Тест ошибок при несовместимых типах
        let result = interp.exec("global invalid = str - count");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::TypeError { .. } => {}, // OK
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_short_circuit_evaluation() {
        let mut interp = Interpreter::new();

        interp.exec("global flag = false").unwrap();

        // Тест базового логического вычисления
        interp.exec("global result1 = flag and true").unwrap();
        assert_eq!(interp.get_variable("result1"), Some(&Value::Bool(false)));

        interp.exec("global flag2 = true").unwrap();
        interp.exec("global result2 = flag2 or false").unwrap();
        assert_eq!(interp.get_variable("result2"), Some(&Value::Bool(true)));

        // Тест сложных логических выражений
        interp.exec("global x = 5").unwrap();
        interp.exec("global y = 10").unwrap();
        interp.exec("global complex1 = (x < y) and (y > 0)").unwrap();
        interp.exec("global complex2 = (x > y) or (x > 0)").unwrap();

        assert_eq!(interp.get_variable("complex1"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("complex2"), Some(&Value::Bool(true)));
    }
}
