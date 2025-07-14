use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_basic_try_catch() {
    let mut interp = Interpreter::new();
    
    // Test basic try/catch functionality
    let code = r#"
        global result = "no_error"
        try
            global x = 10 / 0
        catch error
            result = "caught_error"
        endtry
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::String("caught_error".to_string()));
}

#[test]
fn test_try_catch_with_variable() {
    let mut interp = Interpreter::new();
    
    // Test try/catch with error variable
    let code = r#"
        global error_msg = ""
        try
            throw "Custom error message"
        catch e
            error_msg = e
        endtry
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("error_msg").unwrap();
    assert!(matches!(result, Value::String(s) if s.contains("Custom error message")));
}

#[test]
fn test_try_catch_finally() {
    let mut interp = Interpreter::new();
    
    // Test try/catch/finally
    let code = r#"
        global cleanup_called = false
        global error_caught = false
        
        try
            throw "Test error"
        catch e
            error_caught = true
        finally
            cleanup_called = true
        endtry
    "#;
    
    interp.exec(code).unwrap();
    
    let error_caught = interp.get_variable("error_caught").unwrap();
    assert_eq!(*error_caught, Value::Bool(true));
    
    let cleanup_called = interp.get_variable("cleanup_called").unwrap();
    assert_eq!(*cleanup_called, Value::Bool(true));
}

#[test]
fn test_try_without_error() {
    let mut interp = Interpreter::new();
    
    // Test try block without error
    let code = r#"
        global result = "initial"
        global catch_called = false
        global finally_called = false
        
        try
            result = "success"
        catch e
            catch_called = true
        finally
            finally_called = true
        endtry
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::String("success".to_string()));
    
    let catch_called = interp.get_variable("catch_called").unwrap();
    assert_eq!(*catch_called, Value::Bool(false));
    
    let finally_called = interp.get_variable("finally_called").unwrap();
    assert_eq!(*finally_called, Value::Bool(true));
}

#[test]
fn test_nested_try_catch() {
    let mut interp = Interpreter::new();
    
    // Test nested try/catch blocks
    let code = r#"
        global outer_error = ""
        global inner_error = ""
        
        try
            try
                throw "Inner error"
            catch e
                inner_error = e
                throw "Outer error"
            endtry
        catch e
            outer_error = e
        endtry
    "#;
    
    interp.exec(code).unwrap();
    
    let inner_error = interp.get_variable("inner_error").unwrap();
    assert!(matches!(inner_error, Value::String(s) if s.contains("Inner error")));
    
    let outer_error = interp.get_variable("outer_error").unwrap();
    assert!(matches!(outer_error, Value::String(s) if s.contains("Outer error")));
}

#[test]
fn test_throw_in_function() {
    let mut interp = Interpreter::new();
    
    // Test throw in function
    let code = r#"
        global function risky_function() do
            throw "Function error"
        endfunction
        
        global error_msg = ""
        try
            risky_function()
        catch e
            error_msg = e
        endtry
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("error_msg").unwrap();
    assert!(matches!(result, Value::String(s) if s.contains("Function error")));
}

#[test]
fn test_try_catch_with_return() {
    let mut interp = Interpreter::new();
    
    // Test try/catch with return in function
    let code = r#"
        global function safe_divide(a, b) do
            try
                if b == 0 do
                    throw "Division by zero"
                endif
                return a / b
            catch e
                return -1
            endtry
        endfunction
        
        global result1 = safe_divide(10, 2)
        global result2 = safe_divide(10, 0)
    "#;
    
    interp.exec(code).unwrap();
    
    let result1 = interp.get_variable("result1").unwrap();
    assert_eq!(*result1, Value::Number(5.0));
    
    let result2 = interp.get_variable("result2").unwrap();
    assert_eq!(*result2, Value::Number(-1.0));
}

#[test]
fn test_throw_different_types() {
    let mut interp = Interpreter::new();
    
    // Test throwing different value types
    let code = r#"
        global string_error = ""
        global number_error = ""
        global bool_error = ""
        
        try
            throw "String error"
        catch e
            string_error = e
        endtry
        
        try
            throw 404
        catch e
            number_error = e
        endtry
        
        try
            throw true
        catch e
            bool_error = e
        endtry
    "#;
    
    interp.exec(code).unwrap();
    
    let string_error = interp.get_variable("string_error").unwrap();
    assert!(matches!(string_error, Value::String(s) if s.contains("String error")));
    
    let number_error = interp.get_variable("number_error").unwrap();
    assert!(matches!(number_error, Value::String(s) if s.contains("404")));
    
    let bool_error = interp.get_variable("bool_error").unwrap();
    assert!(matches!(bool_error, Value::String(s) if s.contains("true")));
}

#[test]
fn test_finally_always_executes() {
    let mut interp = Interpreter::new();
    
    // Test that finally always executes, even with return
    let code = r#"
        global finally_count = 0
        
        global function test_finally() do
            try
                finally_count = finally_count + 1
                return "early_return"
            finally
                finally_count = finally_count + 10
            endtry
            return "normal_return"
        endfunction
        
        global result = test_finally()
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::String("early_return".to_string()));
    
    let finally_count = interp.get_variable("finally_count").unwrap();
    assert_eq!(*finally_count, Value::Number(11.0)); // 1 + 10
}
