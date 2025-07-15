use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;

#[cfg(test)]
mod user_functions_tests {
    use super::*;

    #[test]
    fn test_simple_function_definition() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function add(a, b) do
    return a + b
endfunction"#;
        
        let result = interp.exec(function_code);
        assert!(result.is_ok());
        
        // Проверяем, что функция была определена
        assert!(interp.has_user_function("add"));
    }

    #[test]
    fn test_function_call() {
        let mut interp = Interpreter::new();
        
        // Определяем функцию
        let function_code = r#"global function add(a, b) do
    return a + b
endfunction"#;
        
        interp.exec(function_code).unwrap();
        
        // Вызываем функцию
        interp.exec("global result = add(5, 3)").unwrap();
        
        // Проверяем результат
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(8.0)));
    }

    #[test]
    fn test_function_with_no_parameters() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function get_answer() do
    return 42
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global answer = get_answer()").unwrap();
        
        assert_eq!(interp.get_variable("answer"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_function_with_string_operations() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function greet(name) do
    return 'Hello, ' + name + '!'
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global greeting = greet('DataCode')").unwrap();
        
        assert_eq!(interp.get_variable("greeting"), Some(&Value::String("Hello, DataCode!".to_string())));
    }

    #[test]
    fn test_function_with_local_variables() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function calculate(x) do
    local doubled = x * 2
    local result = doubled + 10
    return result
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global final_result = calculate(5)").unwrap();
        
        // Результат должен быть (5 * 2) + 10 = 20
        assert_eq!(interp.get_variable("final_result"), Some(&Value::Number(20.0)));
        
        // Локальные переменные не должны быть видны снаружи
        assert_eq!(interp.get_variable("doubled"), None);
        assert_eq!(interp.get_variable("result"), None);
    }

    #[test]
    //#[ignore = "Not implemented yet"]
    fn test_recursive_function() {
        let mut interp = Interpreter::new();

        let function_code = r#"global function factorial(n) do
    if n <= 1 do
        return 1
    else
        return n * factorial(n - 1)
    endif
endfunction"#;

        interp.exec(function_code).unwrap();
        interp.exec("global fact5 = factorial(5)").unwrap();
        assert_eq!(interp.get_variable("fact5"), Some(&Value::Number(120.0)));

        // Тест других значений
        interp.exec("global fact0 = factorial(0)").unwrap();
        interp.exec("global fact1 = factorial(1)").unwrap();
        interp.exec("global fact3 = factorial(3)").unwrap();

        assert_eq!(interp.get_variable("fact0"), Some(&Value::Number(1.0)));
        assert_eq!(interp.get_variable("fact1"), Some(&Value::Number(1.0)));
        assert_eq!(interp.get_variable("fact3"), Some(&Value::Number(6.0)));
    }

    #[test]
    fn test_function_scope_isolation() {
        let mut interp = Interpreter::new();
        
        // Устанавливаем глобальную переменную
        interp.exec("global x = 100").unwrap();
        
        let function_code = r#"global function test_scope(x) do
    local y = x + 1
    return y
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global result = test_scope(5)").unwrap();
        
        // Результат должен быть 6 (параметр x = 5, y = 5 + 1)
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(6.0)));
        
        // Глобальная переменная x не должна измениться
        assert_eq!(interp.get_variable("x"), Some(&Value::Number(100.0)));
        
        // Локальная переменная y не должна быть видна
        assert_eq!(interp.get_variable("y"), None);
    }

    #[test]
    fn test_function_wrong_argument_count() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function add(a, b) do
    return a + b
endfunction"#;
        
        interp.exec(function_code).unwrap();
        
        // Вызов с неправильным количеством аргументов
        let result = interp.exec("global result = add(5)");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, .. } => {
                assert_eq!(name, "add");
            }
            _ => panic!("Expected FunctionError"),
        }
    }

    #[test]
    fn test_local_function() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"local function helper(x) do
    return x * 2
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global result = helper(10)").unwrap();
        
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(20.0)));
    }

    #[test]
    fn test_function_with_complex_expressions() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function complex_calc(a, b, c) do
    local temp1 = (a + b) * c
    local temp2 = temp1 / 2
    return temp2 - a
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global result = complex_calc(2, 3, 4)").unwrap();
        
        // (2 + 3) * 4 = 20, 20 / 2 = 10, 10 - 2 = 8
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(8.0)));
    }

    #[test]
    fn test_function_calling_builtin() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function get_current_time() do
    local time = now()
    return time
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global current_time = get_current_time()").unwrap();
        
        // Проверяем, что функция вернула строку (время)
        match interp.get_variable("current_time") {
            Some(Value::String(_)) => {}, // OK
            _ => panic!("Expected string from time function"),
        }
    }

    #[test]
    fn test_function_return_without_value() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function void_func() do
    local x = 10
    return
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global result = void_func()").unwrap();
        
        assert_eq!(interp.get_variable("result"), Some(&Value::Null));
    }

    #[test]
    fn test_function_no_explicit_return() {
        let mut interp = Interpreter::new();
        
        let function_code = r#"global function no_return() do
    local x = 10
    local y = x + 5
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global result = no_return()").unwrap();
        
        // Функция без явного return должна возвращать Null
        assert_eq!(interp.get_variable("result"), Some(&Value::Null));
    }

    #[test]
    fn test_function_with_early_return() {
        let _interp = Interpreter::new();

        let _function_code = r#"global function early_return(x) do
    if x > 10 then
        return 'big'
    endif
    return 'small'
endfunction"#;
        
        // Пока что пропускаем этот тест, так как if/else еще не реализованы
        // interp.exec(function_code).unwrap();
        // interp.exec("global result1 = early_return(15)").unwrap();
        // interp.exec("global result2 = early_return(5)").unwrap();
        
        // assert_eq!(interp.get_variable("result1"), Some(&Value::String("big".to_string())));
        // assert_eq!(interp.get_variable("result2"), Some(&Value::String("small".to_string())));
    }

    #[test]
    fn test_function_parameter_shadowing() {
        let mut interp = Interpreter::new();
        
        // Устанавливаем глобальную переменную
        interp.exec("global value = 100").unwrap();
        
        let function_code = r#"global function shadow_test(value) do
    return value + 1
endfunction"#;
        
        interp.exec(function_code).unwrap();
        interp.exec("global result = shadow_test(5)").unwrap();
        
        // Параметр должен затенить глобальную переменную
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(6.0)));
        
        // Глобальная переменная не должна измениться
        assert_eq!(interp.get_variable("value"), Some(&Value::Number(100.0)));
    }

    #[test]
    #[ignore = "Not implemented yet"]
    fn test_function_definition_errors() {
        let mut interp = Interpreter::new();
        
        // Отсутствует endfunction
        let bad_function1 = r#"global function bad() do
    return 42"#;
        
        let result = interp.exec(bad_function1);
        assert!(result.is_err());
        
        // Неправильный синтаксис параметров
        let bad_function2 = r#"global function bad( do
    return 42
endfunction"#;
        
        let result = interp.exec(bad_function2);
        assert!(result.is_err());
        
        // Отсутствует 'do'
        let bad_function3 = r#"global function bad()
    return 42
endfunction"#;
        
        let result = interp.exec(bad_function3);
        assert!(result.is_err());
    }
}
