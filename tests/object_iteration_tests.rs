use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod object_iteration_tests {
    use super::*;

    #[test]
    fn test_object_iteration_single_variable() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {name: \"John\", age: 30, city: \"NYC\"}").unwrap();
        interp.exec("global pairs = []").unwrap();
        
        let loop_code = r#"
            for pair in obj do
                global pairs = push(pairs, pair)
            next pair
        "#;
        
        interp.exec(loop_code).unwrap();
        
        if let Some(Value::Array(pairs)) = interp.get_variable("pairs") {
            assert_eq!(pairs.len(), 3);
            
            // Check that each pair is an array with [key, value]
            for pair in pairs {
                if let Value::Array(kv) = pair {
                    assert_eq!(kv.len(), 2);
                    // First element should be string (key)
                    assert!(matches!(kv[0], Value::String(_)));
                } else {
                    panic!("Expected key-value pair array");
                }
            }
        } else {
            panic!("Expected pairs array");
        }
    }

    #[test]
    fn test_object_iteration_two_variables() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {name: \"John\", age: 30, city: \"NYC\"}").unwrap();
        interp.exec("global keys = []").unwrap();
        interp.exec("global values = []").unwrap();
        
        let loop_code = r#"
            for key, value in obj do
                global keys = push(keys, key)
                global values = push(values, value)
            next key, value
        "#;
        
        interp.exec(loop_code).unwrap();
        
        if let Some(Value::Array(keys)) = interp.get_variable("keys") {
            assert_eq!(keys.len(), 3);
            
            // Keys should be sorted alphabetically
            let expected_keys = vec!["age", "city", "name"];
            for (i, expected_key) in expected_keys.iter().enumerate() {
                if let Value::String(key) = &keys[i] {
                    assert_eq!(key, expected_key);
                } else {
                    panic!("Expected string key");
                }
            }
        } else {
            panic!("Expected keys array");
        }
        
        if let Some(Value::Array(values)) = interp.get_variable("values") {
            assert_eq!(values.len(), 3);
            
            // Values should correspond to sorted keys: age=30, city="NYC", name="John"
            assert_eq!(values[0], Value::Number(30.0)); // age
            assert_eq!(values[1], Value::String("NYC".to_string())); // city
            assert_eq!(values[2], Value::String("John".to_string())); // name
        } else {
            panic!("Expected values array");
        }
    }

    #[test]
    fn test_empty_object_iteration() {
        let mut interp = Interpreter::new();
        
        interp.exec("global empty_obj = {}").unwrap();
        interp.exec("global count = 0").unwrap();
        
        let loop_code = r#"
            for key, value in empty_obj do
                global count = count + 1
            next key, value
        "#;
        
        interp.exec(loop_code).unwrap();
        
        assert_eq!(interp.get_variable("count"), Some(&Value::Number(0.0)));
    }

    #[test]
    fn test_object_iteration_with_nested_values() {
        let mut interp = Interpreter::new();

        interp.exec("global obj = {data: [1, 2, 3], person: {name: \"Alice\"}, active: true}").unwrap();
        interp.exec("global keys_collected = []").unwrap();
        interp.exec("global values_collected = []").unwrap();

        let loop_code = r#"
            for key, value in obj do
                global keys_collected = push(keys_collected, key)
                global values_collected = push(values_collected, value)
            next key, value
        "#;

        interp.exec(loop_code).unwrap();

        if let Some(Value::Array(keys)) = interp.get_variable("keys_collected") {
            assert_eq!(keys.len(), 3);

            // Check keys are sorted alphabetically: active, data, person
            assert_eq!(keys[0], Value::String("active".to_string()));
            assert_eq!(keys[1], Value::String("data".to_string()));
            assert_eq!(keys[2], Value::String("person".to_string()));
        } else {
            panic!("Expected keys array");
        }

        if let Some(Value::Array(values)) = interp.get_variable("values_collected") {
            assert_eq!(values.len(), 3);

            // Check values correspond to sorted keys
            assert_eq!(values[0], Value::Bool(true)); // active

            if let Value::Array(data_array) = &values[1] { // data
                assert_eq!(data_array.len(), 3);
                assert_eq!(data_array[0], Value::Number(1.0));
                assert_eq!(data_array[1], Value::Number(2.0));
                assert_eq!(data_array[2], Value::Number(3.0));
            } else {
                panic!("Expected data array");
            }

            if let Value::Object(person_obj) = &values[2] { // person
                assert_eq!(person_obj.len(), 1);
                assert_eq!(person_obj.get("name"), Some(&Value::String("Alice".to_string())));
            } else {
                panic!("Expected person object");
            }
        } else {
            panic!("Expected values array");
        }
    }

    #[test]
    fn test_object_iteration_key_ordering() {
        let mut interp = Interpreter::new();
        
        // Create object with keys in non-alphabetical order
        interp.exec("global obj = {zebra: 1, apple: 2, banana: 3, dog: 4}").unwrap();
        interp.exec("global key_order = []").unwrap();
        
        let loop_code = r#"
            for key, value in obj do
                global key_order = push(key_order, key)
            next key, value
        "#;
        
        interp.exec(loop_code).unwrap();
        
        if let Some(Value::Array(key_order)) = interp.get_variable("key_order") {
            assert_eq!(key_order.len(), 4);
            
            // Keys should be in alphabetical order
            let expected_order = vec!["apple", "banana", "dog", "zebra"];
            for (i, expected_key) in expected_order.iter().enumerate() {
                if let Value::String(key) = &key_order[i] {
                    assert_eq!(key, expected_key);
                } else {
                    panic!("Expected string key");
                }
            }
        } else {
            panic!("Expected key_order array");
        }
    }

    #[test]
    fn test_nested_object_iteration() {
        let mut interp = Interpreter::new();
        
        interp.exec("global outer = {a: {x: 1, y: 2}, b: {z: 3}}").unwrap();
        interp.exec("global all_values = []").unwrap();
        
        let loop_code = r#"
            for outer_key, inner_obj in outer do
                for inner_key, value in inner_obj do
                    global combined_key = outer_key + "." + inner_key
                    global all_values = push(all_values, [combined_key, value])
                next inner_key, value
            next outer_key, inner_obj
        "#;
        
        interp.exec(loop_code).unwrap();
        
        if let Some(Value::Array(all_values)) = interp.get_variable("all_values") {
            assert_eq!(all_values.len(), 3);
            
            // Should have: a.x=1, a.y=2, b.z=3 (sorted by outer key, then inner key)
            if let Value::Array(first) = &all_values[0] {
                assert_eq!(first[0], Value::String("a.x".to_string()));
                assert_eq!(first[1], Value::Number(1.0));
            }
            
            if let Value::Array(second) = &all_values[1] {
                assert_eq!(second[0], Value::String("a.y".to_string()));
                assert_eq!(second[1], Value::Number(2.0));
            }
            
            if let Value::Array(third) = &all_values[2] {
                assert_eq!(third[0], Value::String("b.z".to_string()));
                assert_eq!(third[1], Value::Number(3.0));
            }
        } else {
            panic!("Expected all_values array");
        }
    }

    #[test]
    fn test_object_iteration_complete() {
        let mut interp = Interpreter::new();

        interp.exec("global obj = {a: 1, b: 2, c: 3, d: 4}").unwrap();
        interp.exec("global collected = []").unwrap();

        let loop_code = r#"
            for key, value in obj do
                global collected = push(collected, key)
            next key, value
        "#;

        interp.exec(loop_code).unwrap();

        if let Some(Value::Array(collected)) = interp.get_variable("collected") {
            // Should have all keys in alphabetical order
            assert_eq!(collected.len(), 4);
            assert_eq!(collected[0], Value::String("a".to_string()));
            assert_eq!(collected[1], Value::String("b".to_string()));
            assert_eq!(collected[2], Value::String("c".to_string()));
            assert_eq!(collected[3], Value::String("d".to_string()));
        } else {
            panic!("Expected collected array");
        }
    }

    #[test]
    fn test_object_iteration_error_with_wrong_variable_count() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {a: 1, b: 2}").unwrap();
        
        // Should error with 3 variables (only 1 or 2 supported)
        let result = interp.exec(r#"
            for x, y, z in obj do
                print("This should not work")
            next x, y, z
        "#);
        
        assert!(result.is_err());
    }
}
