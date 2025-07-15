use data_code::interpreter::Interpreter;
use data_code::value::Value;


#[test]
fn test_empty_object_literal() {
    let mut interp = Interpreter::new();
    
    // Test empty object {}
    interp.exec("global obj = {}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert!(map.is_empty());
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_simple_object_literal() {
    let mut interp = Interpreter::new();
    
    // Test object with simple values
    interp.exec("global obj = {name: 'John', age: 30, active: true}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("name"), Some(&Value::String("John".to_string())));
        assert_eq!(map.get("age"), Some(&Value::Number(30.0)));
        assert_eq!(map.get("active"), Some(&Value::Bool(true)));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_object_with_string_keys() {
    let mut interp = Interpreter::new();
    
    // Test object with string keys
    interp.exec("global obj = {'first name': 'John', 'last name': 'Doe'}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("first name"), Some(&Value::String("John".to_string())));
        assert_eq!(map.get("last name"), Some(&Value::String("Doe".to_string())));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_object_with_expressions() {
    let mut interp = Interpreter::new();
    
    // Test object with expression values
    interp.exec("global x = 10").unwrap();
    interp.exec("global obj = {sum: x + 5, product: x * 2}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("sum"), Some(&Value::Number(15.0)));
        assert_eq!(map.get("product"), Some(&Value::Number(20.0)));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_object_with_nested_array() {
    let mut interp = Interpreter::new();
    
    // Test object with array values
    interp.exec("global obj = {numbers: [1, 2, 3], empty: []}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 2);
        
        if let Some(Value::Array(numbers)) = map.get("numbers") {
            assert_eq!(numbers.len(), 3);
            assert_eq!(numbers[0], Value::Number(1.0));
            assert_eq!(numbers[1], Value::Number(2.0));
            assert_eq!(numbers[2], Value::Number(3.0));
        } else {
            panic!("Expected array for 'numbers' key");
        }
        
        if let Some(Value::Array(empty)) = map.get("empty") {
            assert!(empty.is_empty());
        } else {
            panic!("Expected array for 'empty' key");
        }
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_object_with_trailing_comma() {
    let mut interp = Interpreter::new();
    
    // Test object with trailing comma
    interp.exec("global obj = {a: 1, b: 2,}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("a"), Some(&Value::Number(1.0)));
        assert_eq!(map.get("b"), Some(&Value::Number(2.0)));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_object_indexing() {
    let mut interp = Interpreter::new();
    
    // Test object indexing
    interp.exec("global obj = {name: 'John', age: 30}").unwrap();
    interp.exec("global name_val = obj['name']").unwrap();
    interp.exec("global age_val = obj['age']").unwrap();
    
    let name_result = interp.get_variable("name_val").unwrap();
    assert_eq!(*name_result, Value::String("John".to_string()));
    
    let age_result = interp.get_variable("age_val").unwrap();
    assert_eq!(*age_result, Value::Number(30.0));
}

#[test]
fn test_object_member_access() {
    let mut interp = Interpreter::new();
    
    // Test object member access with dot notation
    interp.exec("global obj = {name: 'John', age: 30}").unwrap();
    interp.exec("global name_val = obj.name").unwrap();
    interp.exec("global age_val = obj.age").unwrap();
    
    let name_result = interp.get_variable("name_val").unwrap();
    assert_eq!(*name_result, Value::String("John".to_string()));
    
    let age_result = interp.get_variable("age_val").unwrap();
    assert_eq!(*age_result, Value::Number(30.0));
}

#[test]
fn test_object_with_null_values() {
    let mut interp = Interpreter::new();
    
    // Test object with null values
    interp.exec("global obj = {value: null, number: 42}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("value"), Some(&Value::Null));
        assert_eq!(map.get("number"), Some(&Value::Number(42.0)));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_object_key_not_found() {
    let mut interp = Interpreter::new();
    
    // Test accessing non-existent key
    interp.exec("global obj = {name: 'John'}").unwrap();
    let result = interp.exec("global missing = obj['missing']");
    
    // Should return an error for missing key
    assert!(result.is_err());
}

#[test]
fn test_nested_objects() {
    let mut interp = Interpreter::new();
    
    // Test nested objects
    interp.exec("global obj = {person: {name: 'John', age: 30}, active: true}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("active"), Some(&Value::Bool(true)));
        
        if let Some(Value::Object(person)) = map.get("person") {
            assert_eq!(person.len(), 2);
            assert_eq!(person.get("name"), Some(&Value::String("John".to_string())));
            assert_eq!(person.get("age"), Some(&Value::Number(30.0)));
        } else {
            panic!("Expected nested object for 'person' key");
        }
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

#[test]
fn test_object_with_function_calls() {
    let mut interp = Interpreter::new();
    
    // Test object with function call values
    interp.exec("global obj = {length: len('hello'), sum: sum([1, 2, 3])}").unwrap();
    
    let result = interp.get_variable("obj").unwrap();
    if let Value::Object(map) = result {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("length"), Some(&Value::Number(5.0)));
        assert_eq!(map.get("sum"), Some(&Value::Number(6.0)));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}
