use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;

#[cfg(test)]
mod loop_tests {
    use super::*;

    #[test]
    fn test_simple_for_loop() {
        let mut interp = Interpreter::new();
        
        // Создаем массив для итерации
        interp.set_variable("numbers".to_string(), Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]), true);
        
        interp.exec("global sum = 0").unwrap();
        
        let loop_code = r#"for num in numbers do
    global sum = sum + num
forend"#;
        
        interp.exec(loop_code).unwrap();
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(6.0)));
    }

    #[test]
    fn test_for_loop_with_strings() {
        let mut interp = Interpreter::new();
        
        // Создаем массив строк
        interp.set_variable("words".to_string(), Value::Array(vec![
            Value::String("Hello".to_string()),
            Value::String(" ".to_string()),
            Value::String("World".to_string()),
        ]), true);
        
        interp.exec("global result = ''").unwrap();
        
        let loop_code = r#"for word in words do
    global result = result + word
forend"#;
        
        interp.exec(loop_code).unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::String("Hello World".to_string())));
    }

    #[test]
    fn test_for_loop_variable_scope() {
        let mut interp = Interpreter::new();
        
        // Устанавливаем переменную с тем же именем, что и переменная цикла
        interp.exec("global item = 'original'").unwrap();
        
        interp.set_variable("items".to_string(), Value::Array(vec![
            Value::String("first".to_string()),
            Value::String("second".to_string()),
        ]), true);
        
        let loop_code = r#"for item in items do
    global last_item = item
forend"#;
        
        interp.exec(loop_code).unwrap();
        
        // Переменная цикла должна быть локальной, поэтому глобальная не должна измениться
        assert_eq!(interp.get_variable("item"), Some(&Value::String("original".to_string())));
        assert_eq!(interp.get_variable("last_item"), Some(&Value::String("second".to_string())));
    }

    #[test]
    fn test_for_loop_empty_array() {
        let mut interp = Interpreter::new();
        
        // Создаем пустой массив
        interp.set_variable("empty".to_string(), Value::Array(vec![]), true);
        
        interp.exec("global counter = 0").unwrap();
        
        let loop_code = r#"for item in empty do
    global counter = counter + 1
forend"#;
        
        interp.exec(loop_code).unwrap();
        
        // Счетчик должен остаться 0, так как массив пустой
        assert_eq!(interp.get_variable("counter"), Some(&Value::Number(0.0)));
    }

    #[test]
    fn test_for_loop_with_conditions() {
        let mut interp = Interpreter::new();
        
        interp.set_variable("numbers".to_string(), Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ]), true);
        
        interp.exec("global even_sum = 0").unwrap();
        
        // Пока что модуль (%) может не быть реализован, поэтому используем другой подход
        let simple_loop_code = r#"for num in numbers do
    if num == 2 do
        global even_sum = even_sum + num
    endif
    if num == 4 do
        global even_sum = even_sum + num
    endif
forend"#;
        
        interp.exec(simple_loop_code).unwrap();
        assert_eq!(interp.get_variable("even_sum"), Some(&Value::Number(6.0))); // 2 + 4
    }

    #[test]
    fn test_for_loop_with_user_functions() {
        let mut interp = Interpreter::new();
        
        // Определяем функцию
        let function_code = r#"global function double(x) do
    return x * 2
endfunction"#;
        
        interp.exec(function_code).unwrap();
        
        interp.set_variable("numbers".to_string(), Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]), true);
        
        interp.exec("global doubled_sum = 0").unwrap();
        
        let loop_code = r#"for num in numbers do
    global doubled_sum = doubled_sum + double(num)
forend"#;
        
        interp.exec(loop_code).unwrap();
        assert_eq!(interp.get_variable("doubled_sum"), Some(&Value::Number(12.0))); // (1*2) + (2*2) + (3*2)
    }

    #[test]
    fn test_nested_for_loops() {
        let mut interp = Interpreter::new();
        
        interp.set_variable("outer".to_string(), Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
        ]), true);
        
        interp.set_variable("inner".to_string(), Value::Array(vec![
            Value::Number(10.0),
            Value::Number(20.0),
        ]), true);
        
        interp.exec("global product_sum = 0").unwrap();
        
        let nested_loop_code = r#"for i in outer do
    for j in inner do
        global product_sum = product_sum + (i * j)
    forend
forend"#;
        
        interp.exec(nested_loop_code).unwrap();
        // (1*10) + (1*20) + (2*10) + (2*20) = 10 + 20 + 20 + 40 = 90
        assert_eq!(interp.get_variable("product_sum"), Some(&Value::Number(90.0)));
    }

    #[test]
    fn test_for_loop_syntax_errors() {
        let mut interp = Interpreter::new();
        
        // Отсутствует forend
        let bad_loop1 = r#"for item in items do
    print(item)"#;
        let result = interp.exec(bad_loop1);
        assert!(result.is_err());
        
        // Неправильный синтаксис for
        let bad_loop2 = r#"for item items do
    print(item)
forend"#;
        let result = interp.exec(bad_loop2);
        assert!(result.is_err());
        
        // Отсутствует 'in'
        let bad_loop3 = r#"for item do
    print(item)
forend"#;
        let result = interp.exec(bad_loop3);
        assert!(result.is_err());
    }

    #[test]
    fn test_for_loop_non_array_error() {
        let mut interp = Interpreter::new();
        
        interp.exec("global not_array = 'string'").unwrap();
        
        let loop_code = r#"for item in not_array do
    print(item)
forend"#;
        
        let result = interp.exec(loop_code);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::TypeError { expected, .. } => {
                assert_eq!(expected, "Array");
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_for_loop_undefined_collection() {
        let mut interp = Interpreter::new();
        
        let loop_code = r#"for item in undefined_array do
    print(item)
forend"#;
        
        let result = interp.exec(loop_code);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            DataCodeError::VariableError { name, .. } => {
                assert_eq!(name, "undefined_array");
            }
            _ => panic!("Expected VariableError"),
        }
    }

    #[test]
    fn test_for_loop_with_return_in_function() {
        let mut interp = Interpreter::new();
        
        // Определяем функцию с циклом и return
        let function_code = r#"global function find_first_even(numbers) do
    for num in numbers do
        if num == 2 do
            return num
        endif
        if num == 4 do
            return num
        endif
    forend
    return -1
endfunction"#;
        
        interp.exec(function_code).unwrap();
        
        interp.set_variable("test_numbers".to_string(), Value::Array(vec![
            Value::Number(1.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ]), true);
        
        interp.exec("global result = find_first_even(test_numbers)").unwrap();
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(4.0)));
    }

    #[test]
    fn test_for_loop_modifying_external_variables() {
        let mut interp = Interpreter::new();
        
        interp.set_variable("items".to_string(), Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ]), true);
        
        interp.exec("global collected = ''").unwrap();
        interp.exec("global count = 0").unwrap();
        
        let loop_code = r#"for item in items do
    global collected = collected + item
    global count = count + 1
forend"#;
        
        interp.exec(loop_code).unwrap();
        
        assert_eq!(interp.get_variable("collected"), Some(&Value::String("abc".to_string())));
        assert_eq!(interp.get_variable("count"), Some(&Value::Number(3.0)));
    }

    #[test]
    fn test_for_loop_with_complex_expressions() {
        let mut interp = Interpreter::new();
        
        interp.set_variable("values".to_string(), Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]), true);
        
        interp.exec("global result = 0").unwrap();
        
        let loop_code = r#"for val in values do
    global result = result + (val * val + 1)
forend"#;
        
        interp.exec(loop_code).unwrap();
        // (1*1+1) + (2*2+1) + (3*3+1) = 2 + 5 + 10 = 17
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(17.0)));
    }
}
