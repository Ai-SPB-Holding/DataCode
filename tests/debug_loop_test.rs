use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[test]
fn test_simple_range_loop() {
    let mut interp = Interpreter::new();

    // Простой тест цикла с range
    let code = r#"
        global counter = 0
        for i in range(3) do
            global counter = counter + 1
            print("Loop iteration, i =", i, "counter =", counter)
        forend
    "#;

    let result = interp.exec(code);
    println!("Result: {:?}", result);

    if let Err(e) = &result {
        println!("Error: {:?}", e);
    }

    assert!(result.is_ok(), "Loop execution failed: {:?}", result);

    let counter_value = interp.get_variable("counter");
    println!("Counter value: {:?}", counter_value);

    let i_value = interp.get_variable("i");
    println!("i value: {:?}", i_value);

    assert_eq!(counter_value, Some(&Value::Number(3.0)));
}

#[test]
fn test_range_function_alone() {
    let mut interp = Interpreter::new();
    
    // Тестируем только функцию range
    let result = interp.exec("global test_range = range(5)");
    assert!(result.is_ok(), "Range function failed: {:?}", result);
    
    let range_value = interp.get_variable("test_range");
    println!("Range value: {:?}", range_value);
    
    if let Some(Value::Array(arr)) = range_value {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], Value::Number(0.0));
        assert_eq!(arr[4], Value::Number(4.0));
    } else {
        panic!("Expected array, got {:?}", range_value);
    }
}

#[test]
fn test_manual_array_loop() {
    let mut interp = Interpreter::new();
    
    // Тестируем цикл с предварительно созданным массивом
    let code = r#"
        global test_array = [0, 1, 2]
        global counter = 0
        for i in test_array do
            global counter = counter + 1
        forend
    "#;
    
    let result = interp.exec(code);
    println!("Manual array loop result: {:?}", result);
    
    if let Err(e) = &result {
        println!("Error: {:?}", e);
    }
    
    assert!(result.is_ok(), "Manual array loop failed: {:?}", result);
    
    let counter_value = interp.get_variable("counter");
    println!("Counter value: {:?}", counter_value);
    
    assert_eq!(counter_value, Some(&Value::Number(3.0)));
}
