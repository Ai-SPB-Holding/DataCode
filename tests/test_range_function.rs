use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_range_single_argument() {
    let mut interp = Interpreter::new();
    
    // Test range(5) -> [0, 1, 2, 3, 4]
    interp.exec("global nums = range(5)").unwrap();
    
    let result = interp.get_variable("nums").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Number(0.0));
        assert_eq!(arr[1], Value::Number(1.0));
        assert_eq!(arr[2], Value::Number(2.0));
        assert_eq!(arr[3], Value::Number(3.0));
        assert_eq!(arr[4], Value::Number(4.0));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_range_two_arguments() {
    let mut interp = Interpreter::new();
    
    // Test range(2, 7) -> [2, 3, 4, 5, 6]
    interp.exec("global nums = range(2, 7)").unwrap();
    
    let result = interp.get_variable("nums").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Number(2.0));
        assert_eq!(arr[1], Value::Number(3.0));
        assert_eq!(arr[2], Value::Number(4.0));
        assert_eq!(arr[3], Value::Number(5.0));
        assert_eq!(arr[4], Value::Number(6.0));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_range_three_arguments_positive_step() {
    let mut interp = Interpreter::new();
    
    // Test range(0, 10, 2) -> [0, 2, 4, 6, 8]
    interp.exec("global nums = range(0, 10, 2)").unwrap();
    
    let result = interp.get_variable("nums").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Number(0.0));
        assert_eq!(arr[1], Value::Number(2.0));
        assert_eq!(arr[2], Value::Number(4.0));
        assert_eq!(arr[3], Value::Number(6.0));
        assert_eq!(arr[4], Value::Number(8.0));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_range_three_arguments_negative_step() {
    let mut interp = Interpreter::new();
    
    // Test range(10, 0, -2) -> [10, 8, 6, 4, 2]
    interp.exec("global nums = range(10, 0, -2)").unwrap();
    
    let result = interp.get_variable("nums").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Number(10.0));
        assert_eq!(arr[1], Value::Number(8.0));
        assert_eq!(arr[2], Value::Number(6.0));
        assert_eq!(arr[3], Value::Number(4.0));
        assert_eq!(arr[4], Value::Number(2.0));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_range_empty() {
    let mut interp = Interpreter::new();
    
    // Test range(0) -> []
    interp.exec("global nums = range(0)").unwrap();
    
    let result = interp.get_variable("nums").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_range_error_negative() {
    let mut interp = Interpreter::new();
    
    // Test range(-5) should error
    let result = interp.exec("global nums = range(-5)");
    assert!(result.is_err());
}

#[test]
fn test_range_error_zero_step() {
    let mut interp = Interpreter::new();
    
    // Test range(0, 10, 0) should error
    let result = interp.exec("global nums = range(0, 10, 0)");
    assert!(result.is_err());
}

#[test]
fn test_range_in_for_loop() {
    let mut interp = Interpreter::new();
    
    // Test using range in for loop
    let code = r#"
        global sum = 0
        for i in range(5) do
            sum = sum + i
        forend
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("sum").unwrap();
    assert_eq!(*result, Value::Number(10.0)); // 0+1+2+3+4 = 10
}

#[test]
fn test_range_performance() {
    let mut interp = Interpreter::new();
    
    // Test range with larger numbers for performance
    interp.exec("global nums = range(1000)").unwrap();
    
    let result = interp.get_variable("nums").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 1000);
        assert_eq!(arr[0], Value::Number(0.0));
        assert_eq!(arr[999], Value::Number(999.0));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}
