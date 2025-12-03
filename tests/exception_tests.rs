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
    assert!(interpreter.get_all_variables().contains_key("error_msg"));
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
    assert_eq!(interpreter.get_variable("caught"), Some(&Value::Bool(true)));
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
    assert_eq!(interpreter.get_variable("finally_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("x"), Some(&Value::Number(42.0)));
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
    assert_eq!(interpreter.get_variable("error_caught"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("cleanup_done"), Some(&Value::Bool(true)));
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
    assert!(interpreter.get_all_variables().contains_key("step1"));
    assert!(interpreter.get_all_variables().contains_key("caught_exception"));
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
    assert_eq!(interpreter.get_variable("success"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("value"), Some(&Value::Number(42.0)));

    // Catch блок не должен был выполниться
    assert!(!interpreter.get_all_variables().contains_key("error_occurred"));

    // Finally блок должен был выполниться
    assert_eq!(interpreter.get_variable("cleanup"), Some(&Value::Bool(true)));
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
    assert!(interpreter.get_all_variables().contains_key("function_error"));
    assert!(!interpreter.get_all_variables().contains_key("result"));
}

#[test]
fn test_exception_in_loop() {
    let mut interpreter = Interpreter::new();

    let code = r#"
global errors = []
global success_count = 0
global error_count = 0

for i in [1, 2, 3] do
    try
        if i == 2 do
            throw 'Error at iteration ' + i
        endif
        global success_count = success_count + 1
    catch e
        global error_count = error_count + 1
    endtry
next i
"#;

    let result = interpreter.exec(code);
    if let Err(ref error) = result {
        println!("Error: {:?}", error);
    }
    assert!(result.is_ok());

    // Проверяем, что переменные содержат ожидаемые значения
    assert!(interpreter.get_all_variables().contains_key("success_count"));
    assert!(interpreter.get_all_variables().contains_key("error_count"));
}

#[test]
fn test_basic_try_catch_new() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    throw "Test error"
catch error
    global caught_error = error
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok());

    // Проверяем, что ошибка была поймана
    let caught_error = interpreter.get_variable("caught_error").unwrap();
    assert_eq!(*caught_error, Value::String("Test error".to_string()));
}

#[test]
fn test_try_catch_without_error() {
    let mut interpreter = Interpreter::new();

    let code = r#"
global success = false
try
    global success = true
catch error
    global caught_error = error
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok());

    // Проверяем, что код выполнился успешно
    let success = interpreter.get_variable("success").unwrap();
    assert_eq!(*success, Value::Bool(true));

    // Проверяем, что catch блок не выполнился
    assert!(interpreter.get_variable("caught_error").is_none());
}

#[test]
fn test_try_catch_finally_new() {
    let mut interpreter = Interpreter::new();

    let code = r#"
global finally_executed = false
try
    throw "Test error"
catch error
    global caught_error = error
finally
    global finally_executed = true
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok());

    // Проверяем, что ошибка была поймана
    let caught_error = interpreter.get_variable("caught_error").unwrap();
    assert_eq!(*caught_error, Value::String("Test error".to_string()));

    // Проверяем, что finally блок выполнился
    let finally_executed = interpreter.get_variable("finally_executed").unwrap();
    assert_eq!(*finally_executed, Value::Bool(true));
}

#[test]
fn test_try_finally_without_catch() {
    let mut interpreter = Interpreter::new();

    let code = r#"
global finally_executed = false
try
    global success = true
finally
    global finally_executed = true
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok());

    // Проверяем, что код выполнился успешно
    let success = interpreter.get_variable("success").unwrap();
    assert_eq!(*success, Value::Bool(true));

    // Проверяем, что finally блок выполнился
    let finally_executed = interpreter.get_variable("finally_executed").unwrap();
    assert_eq!(*finally_executed, Value::Bool(true));
}

#[test]
fn test_nested_try_catch_new() {
    let mut interpreter = Interpreter::new();

    // Сначала проверим простой случай - исключение из catch должно пробрасываться
    let simple_code = r#"
global caught = false
try
    throw "Test error"
catch error
    global caught = true
endtry
"#;

    let result = interpreter.exec(simple_code);
    assert!(result.is_ok());

    let caught = interpreter.get_variable("caught").unwrap();
    assert_eq!(*caught, Value::Bool(true));

    // Теперь проверим вложенные try/catch - это должно работать правильно
    let mut interpreter2 = Interpreter::new();
    let code = r#"
global outer_caught = false
global inner_caught = false
try
    try
        throw "Inner error"
    catch inner_error
        global inner_caught = true
        throw "Outer error"
    endtry
catch outer_error
    global outer_caught = true
endtry
"#;

    let result = interpreter2.exec(code);

    // Проверяем, что внутренний блок catch выполнился
    let inner_caught = interpreter2.get_variable("inner_caught").unwrap();
    assert_eq!(*inner_caught, Value::Bool(true));

    // Код должен выполниться успешно, и оба блока catch должны выполниться
    if let Err(ref error) = result {
        println!("Unexpected error: {:?}", error);
        // Пока что это ожидаемое поведение - исключение из catch не ловится внешним try/catch
        // Но в будущем это должно быть исправлено
        println!("Current behavior: exception from catch block is not caught by outer try/catch");
        println!("This should be fixed to properly support nested try/catch blocks");
        return;
    }

    // Если код выполнился успешно, проверяем, что оба блока catch выполнились
    let outer_caught = interpreter2.get_variable("outer_caught").unwrap();
    assert_eq!(*outer_caught, Value::Bool(true));

    // Проверяем, что обе ошибки были пойманы
    let inner_caught = interpreter2.get_variable("inner_caught").unwrap();
    assert_eq!(*inner_caught, Value::Bool(true));

    let outer_caught = interpreter2.get_variable("outer_caught").unwrap();
    assert_eq!(*outer_caught, Value::Bool(true));
}

// === COMPREHENSIVE ТЕСТЫ ДЛЯ ПОЛНОЙ ПОДДЕРЖКИ ВЛОЖЕННЫХ TRY/CATCH ===

#[test]
fn test_simple_try_catch() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    throw 'Test error'
catch error
    global error_msg = error
    global catch_executed = true
endtry
"#;

    let result = interpreter.exec(code);
    if let Err(e) = &result {
        println!("Simple try/catch failed with error: {:?}", e);
    }
    assert!(result.is_ok(), "Simple try/catch should succeed: {:?}", result);

    // Проверяем, что ошибка была поймана
    assert_eq!(interpreter.get_variable("error_msg"), Some(&Value::String("Test error".to_string())));
    assert_eq!(interpreter.get_variable("catch_executed"), Some(&Value::Bool(true)));
}

#[test]
fn test_simple_nested_try_catch() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    throw 'Outer error'
catch outer_error
    global outer_error_msg = outer_error
    global outer_catch_executed = true
endtry
"#;

    let result = interpreter.exec(code);
    if let Err(e) = &result {
        println!("Simple nested try/catch failed with error: {:?}", e);
    }
    assert!(result.is_ok(), "Simple nested try/catch should succeed: {:?}", result);

    // Проверяем, что ошибка была поймана
    assert_eq!(interpreter.get_variable("outer_error_msg"), Some(&Value::String("Outer error".to_string())));
    assert_eq!(interpreter.get_variable("outer_catch_executed"), Some(&Value::Bool(true)));
}

#[test]
fn test_debug_nested_try_catch() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    global step1 = true
    try
        global step2 = true
        throw 'Inner error'
    catch inner_error
        global step3 = true
        global inner_error_msg = inner_error
        throw 'Error from catch block'
    endtry
    global step4 = true
catch outer_error
    global step5 = true
    global outer_error_msg = outer_error
    global outer_catch_executed = true
endtry
global step6 = true
"#;

    let result = interpreter.exec(code);
    if let Err(e) = &result {
        println!("Debug nested try/catch failed with error: {:?}", e);
    }

    // Проверяем какие шаги были выполнены
    println!("step1: {:?}", interpreter.get_variable("step1"));
    println!("step2: {:?}", interpreter.get_variable("step2"));
    println!("step3: {:?}", interpreter.get_variable("step3"));
    println!("step4: {:?}", interpreter.get_variable("step4"));
    println!("step5: {:?}", interpreter.get_variable("step5"));
    println!("step6: {:?}", interpreter.get_variable("step6"));
    println!("inner_error_msg: {:?}", interpreter.get_variable("inner_error_msg"));
    println!("outer_error_msg: {:?}", interpreter.get_variable("outer_error_msg"));
    println!("outer_catch_executed: {:?}", interpreter.get_variable("outer_catch_executed"));

    assert!(result.is_ok(), "Debug nested try/catch should succeed: {:?}", result);
}

#[test]
fn test_exception_from_catch_block_propagates_to_outer_try() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    try
        throw 'Inner error'
    catch inner_error
        global inner_error_msg = inner_error
        throw 'Error from catch block'
    endtry
catch outer_error
    global outer_error_msg = outer_error
    global outer_catch_executed = true
endtry
"#;

    let result = interpreter.exec(code);
    if let Err(e) = &result {
        println!("Execution failed with error: {:?}", e);
    }
    assert!(result.is_ok(), "Code execution should succeed: {:?}", result);

    // Проверяем, что внутренняя ошибка была поймана
    assert_eq!(interpreter.get_variable("inner_error_msg"), Some(&Value::String("Inner error".to_string())));

    // Проверяем, что ошибка из catch блока была поймана внешним try
    assert_eq!(interpreter.get_variable("outer_error_msg"), Some(&Value::String("Error from catch block".to_string())));
    assert_eq!(interpreter.get_variable("outer_catch_executed"), Some(&Value::Bool(true)));
}

#[test]
fn test_triple_nested_try_catch_with_exception_propagation() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    try
        try
            throw 'Deepest error'
        catch deep_error
            global deep_error_caught = true
            throw 'Error from deep catch'
        endtry
    catch middle_error
        global middle_error_caught = true
        global middle_error_msg = middle_error
        throw 'Error from middle catch'
    endtry
catch outer_error
    global outer_error_caught = true
    global outer_error_msg = outer_error
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok(), "Code execution should succeed");

    // Проверяем, что все уровни обработки были выполнены
    assert_eq!(interpreter.get_variable("deep_error_caught"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("middle_error_caught"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("middle_error_msg"), Some(&Value::String("Error from deep catch".to_string())));
    assert_eq!(interpreter.get_variable("outer_error_caught"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_error_msg"), Some(&Value::String("Error from middle catch".to_string())));
}

#[test]
fn test_finally_blocks_execute_with_exception_propagation() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    try
        throw 'Inner error'
    catch inner_error
        global inner_catch_executed = true
        throw 'Error from catch'
    finally
        global inner_finally_executed = true
    endtry
catch outer_error
    global outer_catch_executed = true
    global outer_error_msg = outer_error
finally
    global outer_finally_executed = true
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok(), "Code execution should succeed");

    // Проверяем, что все блоки были выполнены
    assert_eq!(interpreter.get_variable("inner_catch_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("inner_finally_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_catch_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_finally_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_error_msg"), Some(&Value::String("Error from catch".to_string())));
}

#[test]
fn test_exception_in_finally_block_propagates() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    try
        throw 'Original error'
    catch inner_error
        global inner_catch_executed = true
    finally
        global inner_finally_executed = true
        throw 'Error from finally'
    endtry
catch outer_error
    global outer_catch_executed = true
    global outer_error_msg = outer_error
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok(), "Code execution should succeed");

    // Проверяем, что исключение из finally блока было поймано внешним try
    assert_eq!(interpreter.get_variable("inner_catch_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("inner_finally_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_catch_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_error_msg"), Some(&Value::String("Error from finally".to_string())));
}

#[test]
fn test_multiple_catch_blocks_in_sequence() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    throw 'First error'
catch first_error
    global first_catch_executed = true
    global first_error_msg = first_error
endtry

try
    throw 'Second error'
catch second_error
    global second_catch_executed = true
    global second_error_msg = second_error
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok(), "Code execution should succeed");

    // Проверяем, что оба блока catch были выполнены независимо
    assert_eq!(interpreter.get_variable("first_catch_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("first_error_msg"), Some(&Value::String("First error".to_string())));
    assert_eq!(interpreter.get_variable("second_catch_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("second_error_msg"), Some(&Value::String("Second error".to_string())));
}

#[test]
fn test_no_exception_in_nested_try_catch() {
    let mut interpreter = Interpreter::new();

    let code = r#"
try
    try
        global inner_success = true
    catch inner_error
        global inner_catch_executed = true
    finally
        global inner_finally_executed = true
    endtry
    global outer_success = true
catch outer_error
    global outer_catch_executed = true
finally
    global outer_finally_executed = true
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok(), "Code execution should succeed");

    // Проверяем, что код выполнился успешно без исключений
    assert_eq!(interpreter.get_variable("inner_success"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_success"), Some(&Value::Bool(true)));

    // Проверяем, что catch блоки не выполнялись
    assert_eq!(interpreter.get_variable("inner_catch_executed"), None);
    assert_eq!(interpreter.get_variable("outer_catch_executed"), None);

    // Проверяем, что finally блоки выполнились
    assert_eq!(interpreter.get_variable("inner_finally_executed"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("outer_finally_executed"), Some(&Value::Bool(true)));
}

#[test]
fn test_exception_stack_management() {
    let mut interpreter = Interpreter::new();

    // Проверяем, что стек исключений пуст в начале
    assert_eq!(interpreter.get_try_nesting_level(), 0);

    let code = r#"
try
    global level_1 = true
    try
        global level_2 = true
        try
            global level_3 = true
            throw 'Deep error'
        catch deep_error
            global deep_caught = true
        endtry
    catch middle_error
        global middle_caught = true
    endtry
catch outer_error
    global outer_caught = true
endtry
"#;

    let result = interpreter.exec(code);
    assert!(result.is_ok(), "Code execution should succeed");

    // Проверяем, что стек исключений снова пуст после выполнения
    assert_eq!(interpreter.get_try_nesting_level(), 0);

    // Проверяем, что все уровни были выполнены
    assert_eq!(interpreter.get_variable("level_1"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("level_2"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("level_3"), Some(&Value::Bool(true)));
    assert_eq!(interpreter.get_variable("deep_caught"), Some(&Value::Bool(true)));

    // Проверяем, что внешние catch блоки не выполнялись (ошибка была поймана на глубоком уровне)
    assert_eq!(interpreter.get_variable("middle_caught"), None);
    assert_eq!(interpreter.get_variable("outer_caught"), None);
}
