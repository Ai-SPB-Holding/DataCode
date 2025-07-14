use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_isset_with_valid_value() {
    let mut interp = Interpreter::new();
    
    // Test isset with a valid number
    interp.exec("global num = 42").unwrap();
    interp.exec("global result = isset(num)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(true));
}

#[test]
fn test_isset_with_string() {
    let mut interp = Interpreter::new();
    
    // Test isset with a string
    interp.exec("global text = 'hello'").unwrap();
    interp.exec("global result = isset(text)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(true));
}

#[test]
fn test_isset_with_array() {
    let mut interp = Interpreter::new();
    
    // Test isset with an array
    interp.exec("global arr = [1, 2, 3]").unwrap();
    interp.exec("global result = isset(arr)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(true));
}

#[test]
fn test_isset_with_null() {
    let mut interp = Interpreter::new();
    
    // Test isset with null value
    interp.exec("global nullval = null").unwrap();
    interp.exec("global result = isset(nullval)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(false));
}

#[test]
fn test_isset_with_boolean_true() {
    let mut interp = Interpreter::new();
    
    // Test isset with boolean true
    interp.exec("global flag = true").unwrap();
    interp.exec("global result = isset(flag)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(true));
}

#[test]
fn test_isset_with_boolean_false() {
    let mut interp = Interpreter::new();
    
    // Test isset with boolean false
    interp.exec("global flag = false").unwrap();
    interp.exec("global result = isset(flag)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(true)); // false is still a valid value, not null
}

#[test]
fn test_isset_with_zero() {
    let mut interp = Interpreter::new();
    
    // Test isset with zero (should be true, zero is a valid value)
    interp.exec("global zero = 0").unwrap();
    interp.exec("global result = isset(zero)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(true));
}

#[test]
fn test_isset_with_empty_string() {
    let mut interp = Interpreter::new();
    
    // Test isset with empty string (should be true, empty string is still a valid value)
    interp.exec("global empty = ''").unwrap();
    interp.exec("global result = isset(empty)").unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Bool(true));
}

#[test]
fn test_isset_wrong_argument_count() {
    let mut interp = Interpreter::new();
    
    // Test isset with no arguments - should error
    let result = interp.exec("global result = isset()");
    assert!(result.is_err());
    
    // Test isset with too many arguments - should error
    let result = interp.exec("global result = isset(1, 2)");
    assert!(result.is_err());
}

#[test]
fn test_isset_direct_call() {
    let mut interp = Interpreter::new();

    // Test direct isset call with value
    interp.exec("global value = 42").unwrap();
    interp.exec("global result1 = isset(value)").unwrap();

    let result1 = interp.get_variable("result1").unwrap();
    assert_eq!(*result1, Value::Bool(true));

    // Test direct isset call with null
    interp.exec("global nullval = null").unwrap();
    interp.exec("global result2 = isset(nullval)").unwrap();

    let result2 = interp.get_variable("result2").unwrap();
    assert_eq!(*result2, Value::Bool(false));
}
