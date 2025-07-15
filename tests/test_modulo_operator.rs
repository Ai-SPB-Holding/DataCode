use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_modulo_basic() {
    let mut interp = Interpreter::new();
    
    // Test basic modulo operations
    interp.exec("global result1 = 10 % 3").unwrap();
    interp.exec("global result2 = 15 % 4").unwrap();
    interp.exec("global result3 = 20 % 5").unwrap();
    
    assert_eq!(interp.get_variable("result1"), Some(&Value::Number(1.0)));
    assert_eq!(interp.get_variable("result2"), Some(&Value::Number(3.0)));
    assert_eq!(interp.get_variable("result3"), Some(&Value::Number(0.0)));
}

#[test]
fn test_modulo_with_variables() {
    let mut interp = Interpreter::new();
    
    // Test modulo with variables
    interp.exec("global a = 17").unwrap();
    interp.exec("global b = 5").unwrap();
    interp.exec("global result = a % b").unwrap();
    
    assert_eq!(interp.get_variable("result"), Some(&Value::Number(2.0)));
}

#[test]
fn test_modulo_in_expressions() {
    let mut interp = Interpreter::new();
    
    // Test modulo in complex expressions
    interp.exec("global result = (10 + 5) % 4").unwrap();
    assert_eq!(interp.get_variable("result"), Some(&Value::Number(3.0)));
    
    // Test modulo with multiplication
    interp.exec("global result2 = 7 * 3 % 5").unwrap();
    assert_eq!(interp.get_variable("result2"), Some(&Value::Number(1.0)));
}

#[test]
fn test_modulo_zero_error() {
    let mut interp = Interpreter::new();
    
    // Test modulo by zero should fail
    let result = interp.exec("global result = 10 % 0");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Modulo by zero"));
}

#[test]
fn test_modulo_precedence() {
    let mut interp = Interpreter::new();
    
    // Test that modulo has same precedence as multiplication and division
    interp.exec("global result1 = 2 + 3 % 2").unwrap(); // Should be 2 + (3 % 2) = 2 + 1 = 3
    interp.exec("global result2 = 10 % 3 * 2").unwrap(); // Should be (10 % 3) * 2 = 1 * 2 = 2
    
    assert_eq!(interp.get_variable("result1"), Some(&Value::Number(3.0)));
    assert_eq!(interp.get_variable("result2"), Some(&Value::Number(2.0)));
}

#[test]
fn test_modulo_performance_test_expressions() {
    let mut interp = Interpreter::new();
    
    // Test the specific expressions used in performance tests
    interp.exec("global i = 25").unwrap();
    interp.exec("global dept_mod = i % 10").unwrap();
    interp.exec("global salary_mod = i % 20").unwrap();
    
    assert_eq!(interp.get_variable("dept_mod"), Some(&Value::Number(5.0)));
    assert_eq!(interp.get_variable("salary_mod"), Some(&Value::Number(5.0)));
}
