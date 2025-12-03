use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod spread_operator_tests {
    use super::*;

    #[test]
    fn test_spread_object_with_print() {
        let mut interp = Interpreter::new();
        
        // Create object and test spread with print function
        interp.exec("global obj = {name: \"John\", age: 30, city: \"NYC\"}").unwrap();
        
        // This should work without errors - print accepts multiple arguments
        let result = interp.exec("print(\"Values:\", *obj)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_array_with_print() {
        let mut interp = Interpreter::new();
        
        // Create array and test spread with print function
        interp.exec("global arr = [1, 2, 3, 4, 5]").unwrap();
        
        // This should work without errors - print accepts multiple arguments
        let result = interp.exec("print(\"Numbers:\", *arr)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_empty_object() {
        let mut interp = Interpreter::new();
        
        interp.exec("global empty_obj = {}").unwrap();
        
        // Spreading empty object should work
        let result = interp.exec("print(\"Empty:\", *empty_obj)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_empty_array() {
        let mut interp = Interpreter::new();
        
        interp.exec("global empty_arr = []").unwrap();
        
        // Spreading empty array should work
        let result = interp.exec("print(\"Empty:\", *empty_arr)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_with_mixed_arguments() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {a: 1, b: 2}").unwrap();
        interp.exec("global arr = [3, 4, 5]").unwrap();
        
        // Mix regular arguments with spread arguments
        let result = interp.exec("print(\"Mixed:\", *obj, \"separator\", *arr)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_object_key_value_order() {
        let mut interp = Interpreter::new();

        // Test that object spread follows sorted key order
        interp.exec("global obj = {zebra: \"z\", apple: \"a\", banana: \"b\"}").unwrap();

        // Test spread with print function - should work without errors
        let result = interp.exec("print(\"Object spread:\", *obj)");
        assert!(result.is_ok());

        // Test that we can iterate over the object to verify order
        interp.exec("global values_in_order = []").unwrap();
        let result = interp.exec(r#"
            for key, value in obj do
                global values_in_order = push(values_in_order, value)
            next key, value
        "#);
        assert!(result.is_ok());

        // Check that values are in sorted key order (apple, banana, zebra)
        if let Some(Value::Array(values)) = interp.get_variable("values_in_order") {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], Value::String("a".to_string())); // apple value
            assert_eq!(values[1], Value::String("b".to_string())); // banana value
            assert_eq!(values[2], Value::String("z".to_string())); // zebra value
        } else {
            panic!("Expected values array");
        }
    }

    #[test]
    fn test_spread_array_preserves_order() {
        let mut interp = Interpreter::new();

        interp.exec("global arr = [\"first\", \"second\", \"third\"]").unwrap();

        // Test spread with print function - should work without errors
        let result = interp.exec("print(\"Array spread:\", *arr)");
        assert!(result.is_ok());

        // Test that we can iterate over the array to verify order
        interp.exec("global items_in_order = []").unwrap();
        let result = interp.exec(r#"
            for item in arr do
                global items_in_order = push(items_in_order, item)
            next item
        "#);
        assert!(result.is_ok());

        if let Some(Value::Array(items)) = interp.get_variable("items_in_order") {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::String("first".to_string()));
            assert_eq!(items[1], Value::String("second".to_string()));
            assert_eq!(items[2], Value::String("third".to_string()));
        } else {
            panic!("Expected items array");
        }
    }

    #[test]
    fn test_spread_with_nested_structures() {
        let mut interp = Interpreter::new();
        
        // Test spreading object with nested arrays
        interp.exec("global obj = {data: [1, 2], names: [\"Alice\", \"Bob\"]}").unwrap();
        
        let result = interp.exec("print(\"Nested:\", *obj)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_error_with_invalid_types() {
        let mut interp = Interpreter::new();
        
        // Test that spreading non-object/array types produces error
        interp.exec("global num = 42").unwrap();
        interp.exec("global text = \"hello\"").unwrap();
        interp.exec("global flag = true").unwrap();
        
        // These should produce errors
        let result1 = interp.exec("print(*num)");
        assert!(result1.is_err());
        
        let result2 = interp.exec("print(*text)");
        assert!(result2.is_err());
        
        let result3 = interp.exec("print(*flag)");
        assert!(result3.is_err());
    }

    #[test]
    fn test_spread_multiple_objects() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj1 = {a: 1, b: 2}").unwrap();
        interp.exec("global obj2 = {x: 10, y: 20}").unwrap();
        
        // Spread multiple objects in one call
        let result = interp.exec("print(\"Multiple:\", *obj1, \"separator\", *obj2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_multiple_arrays() {
        let mut interp = Interpreter::new();
        
        interp.exec("global arr1 = [1, 2, 3]").unwrap();
        interp.exec("global arr2 = [\"a\", \"b\", \"c\"]").unwrap();
        
        // Spread multiple arrays in one call
        let result = interp.exec("print(\"Multiple:\", *arr1, \"separator\", *arr2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spread_in_complex_expression() {
        let mut interp = Interpreter::new();
        
        interp.exec("global data = {count: 3, items: [\"a\", \"b\", \"c\"]}").unwrap();
        
        // Use spread in more complex context
        let result = interp.exec("print(\"Complex:\", \"start\", *data, \"end\")");
        assert!(result.is_ok());
    }
}
