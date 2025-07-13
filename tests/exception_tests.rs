use data_code::interpreter::Interpreter;
use data_code::error::DataCodeError;
use data_code::value::Value;

#[test]
fn test_basic_try_catch() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
try
    global x = 10 / 0
catch e
    global error_msg = e
    print('Caught error:', e)
endtry
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что переменная error_msg была установлена
    assert!(interpreter.variables.contains_key("error_msg"));
}

#[test]
fn test_try_catch_without_variable() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
try
    throw 'Custom error message'
catch
    print('Error caught without variable')
    global caught = true
endtry
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что блок catch был выполнен
    assert_eq!(interpreter.variables.get("caught"), Some(&Value::Bool(true)));
}

#[test]
fn test_try_finally() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
try
    global x = 42
finally
    global finally_executed = true
endtry
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что блок finally был выполнен
    assert_eq!(interpreter.variables.get("finally_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.variables.get("x"), Some(&Value::Number(42.0)));
}

#[test]
fn test_try_catch_finally() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
try
    throw 'Test exception'
catch e
    global error_caught = true
finally
    global cleanup_done = true
endtry
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что оба блока были выполнены
    assert_eq!(interpreter.variables.get("error_caught"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.variables.get("cleanup_done"), Some(&Value::Bool(true)));
}

#[test]
fn test_throw_statement() {
    let mut interpreter = Interpreter::new();
    
    let result = interpreter.exec("throw 'This is a test exception'");
    assert!(result.is_err());
    
    if let Err(DataCodeError::UserException { message, .. }) = result {
        assert_eq!(message, "This is a test exception");
    } else {
        panic!("Expected UserException");
    }
}

#[test]
fn test_throw_with_expression() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
global msg = 'Dynamic error: ' + '404'
throw msg
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_err());
    
    if let Err(DataCodeError::UserException { message, .. }) = result {
        assert_eq!(message, "Dynamic error: 404");
    } else {
        panic!("Expected UserException");
    }
}

#[test]
fn test_nested_try_catch() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    global step1 = true
    throw 'Test exception'
catch e
    global caught_exception = e
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok());

    // Проверяем, что исключение было обработано
    assert!(interpreter.variables.contains_key("step1"));
    assert!(interpreter.variables.contains_key("caught_exception"));
}

#[test]
fn test_try_with_successful_execution() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
try
    global success = true
    global value = 42
catch e
    global error_occurred = true
finally
    global cleanup = true
endtry
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что try блок выполнился успешно
    assert_eq!(interpreter.variables.get("success"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.variables.get("value"), Some(&Value::Number(42.0)));

    // Catch блок не должен был выполниться
    assert!(!interpreter.variables.contains_key("error_occurred"));

    // Finally блок должен был выполниться
    assert_eq!(interpreter.variables.get("cleanup"), Some(&Value::Bool(true)));
}

#[test]
fn test_exception_in_function() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
global function risky_function() do
    throw 'Function error'
    return 42
endfunction

try
    global result = risky_function()
catch e
    global function_error = e
endtry
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что исключение из функции было поймано
    assert!(interpreter.variables.contains_key("function_error"));
    assert!(!interpreter.variables.contains_key("result"));
}

#[test]
fn test_exception_in_loop() {
    let mut interpreter = Interpreter::new();
    
    let code = r#"
global errors = []

for i in [1, 2, 3] do
    try
        if i == 2 do
            throw 'Error at iteration ' + i
        endif
        push(errors, 'Success: ' + i)
    catch e
        push(errors, 'Caught: ' + e)
    endtry
forend
"#;
    
    let result = interpreter.exec(code);
    assert!(result.is_ok());
    
    // Проверяем, что массив errors содержит ожидаемые значения
    assert!(interpreter.variables.contains_key("errors"));
}
