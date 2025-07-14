use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_map_with_builtin_function() {
    let mut interp = Interpreter::new();

    // Test map with built-in math function
    let code = r#"
        global numbers = [1, 2, 3, 4, 5]
        global result = map(numbers, "abs")
    "#;

    interp.exec(code).unwrap();

    let result = interp.get_variable("result").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Number(1.0));
        assert_eq!(arr[1], Value::Number(2.0));
        assert_eq!(arr[2], Value::Number(3.0));
        assert_eq!(arr[3], Value::Number(4.0));
        assert_eq!(arr[4], Value::Number(5.0));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_filter_with_builtin_function() {
    let mut interp = Interpreter::new();

    // Test filter with built-in function
    let code = r#"
        global numbers = [1, 2, 3, 4, 5]
        global result = filter(numbers, "abs")
    "#;

    interp.exec(code).unwrap();

    let result = interp.get_variable("result").unwrap();
    if let Value::Array(arr) = result {
        // All positive numbers should pass through (abs returns truthy values)
        assert_eq!(arr.len(), 5);
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_reduce_with_builtin_function() {
    let mut interp = Interpreter::new();

    // Test reduce with built-in max function
    let code = r#"
        global numbers = [1, 5, 3, 9, 2]
        global max_result = reduce(numbers, "max")
    "#;

    interp.exec(code).unwrap();

    let result = interp.get_variable("max_result").unwrap();
    assert_eq!(*result, Value::Number(9.0));
}

#[test]
fn test_reduce_with_initial_value() {
    let mut interp = Interpreter::new();
    
    // Test reduce with initial value
    let code = r#"
        global numbers = [1, 2, 3]
        global result = reduce(numbers, "max", 10)
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(10.0)); // Initial value 10 is max
}

#[test]
fn test_reduce_empty_array_with_initial() {
    let mut interp = Interpreter::new();
    
    // Test reduce with empty array and initial value
    let code = r#"
        global empty_array = []
        global result = reduce(empty_array, "max", 42)
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(42.0)); // Should return initial value
}

#[test]
fn test_reduce_empty_array_without_initial() {
    let mut interp = Interpreter::new();
    
    // Test reduce with empty array without initial value (should error)
    let code = r#"
        global empty_array = []
        global result = reduce(empty_array, "max")
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_err()); // Should fail
}

#[test]
fn test_map_empty_array() {
    let mut interp = Interpreter::new();
    
    // Test map with empty array
    let code = r#"
        global empty_array = []
        global result = map(empty_array, "abs")
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_filter_empty_array() {
    let mut interp = Interpreter::new();
    
    // Test filter with empty array
    let code = r#"
        global empty_array = []
        global result = filter(empty_array, "abs")
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_map_wrong_arguments() {
    let mut interp = Interpreter::new();
    
    // Test map with wrong number of arguments
    let code = r#"
        global numbers = [1, 2, 3]
        global result = map(numbers)
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_err()); // Should fail due to wrong argument count
}

#[test]
fn test_filter_wrong_arguments() {
    let mut interp = Interpreter::new();
    
    // Test filter with wrong number of arguments
    let code = r#"
        global numbers = [1, 2, 3]
        global result = filter(numbers, "abs", "extra")
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_err()); // Should fail due to wrong argument count
}

#[test]
fn test_reduce_wrong_arguments() {
    let mut interp = Interpreter::new();
    
    // Test reduce with wrong number of arguments
    let code = r#"
        global numbers = [1, 2, 3]
        global result = reduce(numbers)
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_err()); // Should fail due to wrong argument count
}

#[test]
fn test_map_with_non_array() {
    let mut interp = Interpreter::new();
    
    // Test map with non-array input
    let code = r#"
        global not_array = "hello"
        global result = map(not_array, "abs")
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_err()); // Should fail due to type error
}

#[test]
fn test_filter_with_non_array() {
    let mut interp = Interpreter::new();
    
    // Test filter with non-array input
    let code = r#"
        global not_array = 42
        global result = filter(not_array, "abs")
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_err()); // Should fail due to type error
}

#[test]
fn test_reduce_with_non_array() {
    let mut interp = Interpreter::new();
    
    // Test reduce with non-array input
    let code = r#"
        global not_array = true
        global result = reduce(not_array, "max")
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_err()); // Should fail due to type error
}

#[test]
fn test_functional_methods_chaining() {
    let mut interp = Interpreter::new();
    
    // Test chaining functional methods (conceptually)
    let code = r#"
        global numbers = [1, 2, 3, 4, 5, 6]
        global doubled = map(numbers, "abs")
        global filtered = filter(doubled, "abs")
        global sum_result = reduce(filtered, "max")
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("sum_result").unwrap();
    assert_eq!(*result, Value::Number(6.0)); // Max of [1,2,3,4,5,6] is 6
}

#[test]
fn test_map_with_mixed_types() {
    let mut interp = Interpreter::new();
    
    // Test map with mixed types (should work with abs)
    let code = r#"
        global mixed = [-1, 2, -3, 4]
        global result = map(mixed, "abs")
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0], Value::Number(1.0));
        assert_eq!(arr[1], Value::Number(2.0));
        assert_eq!(arr[2], Value::Number(3.0));
        assert_eq!(arr[3], Value::Number(4.0));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}
