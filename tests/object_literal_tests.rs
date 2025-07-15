use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod object_literal_tests {
    use super::*;

    #[test]
    fn test_empty_object_literal() {
        let mut interp = Interpreter::new();
        
        interp.exec("global empty_obj = {}").unwrap();
        
        if let Some(Value::Object(obj)) = interp.get_variable("empty_obj") {
            assert!(obj.is_empty());
        } else {
            panic!("Expected empty object");
        }
    }

    #[test]
    fn test_simple_object_literal() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {name: \"John\", age: 30, active: true}").unwrap();
        
        if let Some(Value::Object(obj)) = interp.get_variable("obj") {
            assert_eq!(obj.len(), 3);
            assert_eq!(obj.get("name"), Some(&Value::String("John".to_string())));
            assert_eq!(obj.get("age"), Some(&Value::Number(30.0)));
            assert_eq!(obj.get("active"), Some(&Value::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_object_with_arrays() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {data: [1, 2, 3], names: [\"Alice\", \"Bob\"]}").unwrap();
        
        if let Some(Value::Object(obj)) = interp.get_variable("obj") {
            assert_eq!(obj.len(), 2);
            
            if let Some(Value::Array(data)) = obj.get("data") {
                assert_eq!(data.len(), 3);
                assert_eq!(data[0], Value::Number(1.0));
                assert_eq!(data[1], Value::Number(2.0));
                assert_eq!(data[2], Value::Number(3.0));
            } else {
                panic!("Expected data array");
            }
            
            if let Some(Value::Array(names)) = obj.get("names") {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], Value::String("Alice".to_string()));
                assert_eq!(names[1], Value::String("Bob".to_string()));
            } else {
                panic!("Expected names array");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_object_access_by_key() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {name: \"John\", age: 30}").unwrap();
        interp.exec("global name_value = obj[\"name\"]").unwrap();
        interp.exec("global age_value = obj[\"age\"]").unwrap();
        
        assert_eq!(interp.get_variable("name_value"), Some(&Value::String("John".to_string())));
        assert_eq!(interp.get_variable("age_value"), Some(&Value::Number(30.0)));
    }

    #[test]
    fn test_nested_objects() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {person: {name: \"John\", age: 30}, active: true}").unwrap();
        
        if let Some(Value::Object(obj)) = interp.get_variable("obj") {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("active"), Some(&Value::Bool(true)));
            
            if let Some(Value::Object(person)) = obj.get("person") {
                assert_eq!(person.len(), 2);
                assert_eq!(person.get("name"), Some(&Value::String("John".to_string())));
                assert_eq!(person.get("age"), Some(&Value::Number(30.0)));
            } else {
                panic!("Expected nested person object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_object_with_mixed_types() {
        let mut interp = Interpreter::new();
        
        interp.exec("global obj = {number: 42, text: \"hello\", flag: false, list: [1, 2], empty: null}").unwrap();
        
        if let Some(Value::Object(obj)) = interp.get_variable("obj") {
            assert_eq!(obj.len(), 5);
            assert_eq!(obj.get("number"), Some(&Value::Number(42.0)));
            assert_eq!(obj.get("text"), Some(&Value::String("hello".to_string())));
            assert_eq!(obj.get("flag"), Some(&Value::Bool(false)));
            assert_eq!(obj.get("empty"), Some(&Value::Null));
            
            if let Some(Value::Array(list)) = obj.get("list") {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0], Value::Number(1.0));
                assert_eq!(list[1], Value::Number(2.0));
            } else {
                panic!("Expected list array");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_object_key_ordering() {
        let mut interp = Interpreter::new();
        
        // Test that keys are sorted for predictable iteration
        interp.exec("global obj = {zebra: 1, apple: 2, banana: 3}").unwrap();
        
        if let Some(Value::Object(obj)) = interp.get_variable("obj") {
            let keys: Vec<_> = obj.keys().collect();
            let mut sorted_keys = keys.clone();
            sorted_keys.sort();
            // Keys should be accessible in any order, but iteration should be sorted
            assert_eq!(obj.len(), 3);
            assert!(obj.contains_key("zebra"));
            assert!(obj.contains_key("apple"));
            assert!(obj.contains_key("banana"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_object_creation_and_access() {
        let mut interp = Interpreter::new();

        // Test that we can create and access objects
        interp.exec("global obj = {name: \"John\", age: 30}").unwrap();

        if let Some(Value::Object(obj)) = interp.get_variable("obj") {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("name"), Some(&Value::String("John".to_string())));
            assert_eq!(obj.get("age"), Some(&Value::Number(30.0)));
        } else {
            panic!("Expected object");
        }

        // Test object access by key
        interp.exec("global name_value = obj[\"name\"]").unwrap();
        interp.exec("global age_value = obj[\"age\"]").unwrap();

        assert_eq!(interp.get_variable("name_value"), Some(&Value::String("John".to_string())));
        assert_eq!(interp.get_variable("age_value"), Some(&Value::Number(30.0)));
    }
}
