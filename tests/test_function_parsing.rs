use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_simple_function_definition() {
    let mut interp = Interpreter::new();
    
    // Test simple function definition and call
    let code = r#"
        global function add(a, b) do
            return a + b
        endfunction
        
        global result = add(5, 3)
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(8.0));
}

#[test]
fn test_function_without_parameters() {
    let mut interp = Interpreter::new();
    
    // Test function without parameters
    let code = r#"
        global function get_hello() do
            return "Hello, World!"
        endfunction
        
        global message = get_hello()
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("message").unwrap();
    assert_eq!(*result, Value::String("Hello, World!".to_string()));
}

#[test]
fn test_function_with_local_variables() {
    let mut interp = Interpreter::new();
    
    // Test function with local variables
    let code = r#"
        global function calculate() do
            local x = 10
            local y = 20
            return x * y
        endfunction
        
        global result = calculate()
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(200.0));
}

#[test]
fn test_recursive_function() {
    let mut interp = Interpreter::new();
    
    // Test recursive function (factorial)
    let code = r#"
        global function factorial(n) do
            if n <= 1 do
                return 1
            endif
            return n * factorial(n - 1)
        endfunction
        
        global result = factorial(5)
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(120.0)); // 5! = 120
}

#[test]
fn test_function_with_array_parameter() {
    let mut interp = Interpreter::new();
    
    // Test function with array parameter
    let code = r#"
        global function sum_array(arr) do
            local total = 0
            for item in arr do
                total = total + item
            forend
            return total
        endfunction

        global numbers = [1, 2, 3, 4, 5]
        global result = sum_array(numbers)
    "#;

    interp.exec(code).unwrap();

    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(15.0)); // 1+2+3+4+5 = 15
}

#[test]
fn test_local_function() {
    let mut interp = Interpreter::new();
    
    // Test local function definition
    let code = r#"
        local function helper(x) do
            return x * 2
        endfunction
        
        global result = helper(21)
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(42.0));
}

#[test]
fn test_function_with_object_return() {
    let mut interp = Interpreter::new();
    
    // Test function returning object
    let code = r#"
        global function create_person(name, age) do
            return {name: name, age: age}
        endfunction
        
        global person = create_person("John", 30)
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("person").unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(obj.get("name"), Some(&Value::String("John".to_string())));
        assert_eq!(obj.get("age"), Some(&Value::Number(30.0)));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_function_call_in_expression() {
    let mut interp = Interpreter::new();
    
    // Test function call within expression
    let code = r#"
        global function double(x) do
            return x * 2
        endfunction
        
        global function triple(x) do
            return x * 3
        endfunction
        
        global result = double(5) + triple(4)
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Number(22.0)); // (5*2) + (4*3) = 10 + 12 = 22
}

#[test]
fn test_function_without_return() {
    let mut interp = Interpreter::new();
    
    // Test function without explicit return (should return null)
    let code = r#"
        global function no_return() do
            local x = 42
        endfunction
        
        global result = no_return()
    "#;
    
    interp.exec(code).unwrap();
    
    let result = interp.get_variable("result").unwrap();
    assert_eq!(*result, Value::Null);
}

#[test]
fn test_function_with_early_return() {
    let mut interp = Interpreter::new();
    
    // Test function with early return
    let code = r#"
        global function check_positive(x) do
            if x > 0 do
                return "positive"
            endif
            return "not positive"
        endfunction
        
        global result1 = check_positive(5)
        global result2 = check_positive(-3)
    "#;
    
    interp.exec(code).unwrap();
    
    let result1 = interp.get_variable("result1").unwrap();
    assert_eq!(*result1, Value::String("positive".to_string()));
    
    let result2 = interp.get_variable("result2").unwrap();
    assert_eq!(*result2, Value::String("not positive".to_string()));
}
