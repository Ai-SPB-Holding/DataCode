use data_code::interpreter::Interpreter;

#[test]
fn test_print_single_argument() {
    let mut interpreter = Interpreter::new();
    
    // Тест print с одним аргументом
    let result = interpreter.exec("print('Hello World')");
    assert!(result.is_ok());
}

#[test]
fn test_print_multiple_arguments() {
    let mut interpreter = Interpreter::new();
    
    // Устанавливаем переменную
    interpreter.exec("global a = 42").unwrap();
    
    // Тест print с несколькими аргументами
    let result = interpreter.exec("print('a=', a)");
    assert!(result.is_ok());
}

#[test]
fn test_print_with_variables() {
    let mut interpreter = Interpreter::new();
    
    // Устанавливаем переменные
    interpreter.exec("global name = 'DataCode'").unwrap();
    interpreter.exec("global version = 1.4").unwrap();
    
    // Тест print с переменными
    let result = interpreter.exec("print('Language:', name, 'Version:', version)");
    assert!(result.is_ok());
}

#[test]
fn test_print_with_expressions() {
    let mut interpreter = Interpreter::new();
    
    // Устанавливаем переменные
    interpreter.exec("global x = 10").unwrap();
    interpreter.exec("global y = 5").unwrap();
    
    // Тест print с выражениями
    let result = interpreter.exec("print('x + y =', x + y)");
    assert!(result.is_ok());
    
    let result = interpreter.exec("print('x * y =', x * y)");
    assert!(result.is_ok());
}

#[test]
fn test_print_different_types() {
    let mut interpreter = Interpreter::new();
    
    // Устанавливаем переменные разных типов
    interpreter.exec("global num = 123").unwrap();
    interpreter.exec("global text = 'hello'").unwrap();
    interpreter.exec("global flag = true").unwrap();
    
    // Тест print с разными типами
    let result = interpreter.exec("print('Number:', num, 'Text:', text, 'Flag:', flag)");
    assert!(result.is_ok());
}

#[test]
fn test_print_empty_call() {
    let mut interpreter = Interpreter::new();
    
    // Тест print без аргументов (должен просто вывести пустую строку)
    let result = interpreter.exec("print()");
    assert!(result.is_ok());
}
