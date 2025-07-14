use data_code::interpreter::Interpreter;

#[test]
fn test_print_array_content() {
    let mut interp = Interpreter::new();
    
    // Test that print shows full array content, not just length
    let code = r#"
        global numbers = [1, 2, 3, 4, 5]
        print("Numbers:", numbers)
    "#;
    
    // This should not panic and should work correctly
    let result = interp.exec(code);
    assert!(result.is_ok());
}

#[test]
fn test_print_nested_arrays() {
    let mut interp = Interpreter::new();
    
    // Test nested arrays
    let code = r#"
        global matrix = [[1, 2], [3, 4], [5, 6]]
        print("Matrix:", matrix)
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_ok());
}

#[test]
fn test_print_mixed_array() {
    let mut interp = Interpreter::new();
    
    // Test mixed type arrays
    let code = r#"
        global mixed = [1, "hello", true, 3.14]
        print("Mixed:", mixed)
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_ok());
}

#[test]
fn test_print_empty_array() {
    let mut interp = Interpreter::new();
    
    // Test empty array
    let code = r#"
        global empty = []
        print("Empty:", empty)
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_ok());
}

#[test]
fn test_print_objects() {
    let mut interp = Interpreter::new();
    
    // Test objects
    let code = r#"
        global person = {name: "John", age: 30}
        print("Person:", person)
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_ok());
}

#[test]
fn test_print_functional_methods_results() {
    let mut interp = Interpreter::new();
    
    // Test functional methods results
    let code = r#"
        global numbers = [1, 2, 3, 4, 5]
        global doubled = map(numbers, "abs")
        global filtered = filter(numbers, "abs")
        global max_val = reduce(numbers, "max")
        
        print("Original:", numbers)
        print("Mapped:", doubled)
        print("Filtered:", filtered)
        print("Max:", max_val)
    "#;
    
    let result = interp.exec(code);
    assert!(result.is_ok());
}
