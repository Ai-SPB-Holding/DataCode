use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod table_create_tests {
    use super::*;

    #[test]
    fn test_table_create_basic() {
        let mut interp = Interpreter::new();
        
        // Create test data
        interp.set_variable("data".to_string(), Value::Array(vec![
            Value::Array(vec![
                Value::Number(1.0),
                Value::String("Alice".to_string()),
                Value::Number(25.0)
            ]),
            Value::Array(vec![
                Value::Number(2.0),
                Value::String("Bob".to_string()),
                Value::Number(30.0)
            ])
        ]), true);
        
        // Create headers
        interp.set_variable("headers".to_string(), Value::Array(vec![
            Value::String("id".to_string()),
            Value::String("name".to_string()),
            Value::String("age".to_string())
        ]), true);
        
        // Test table_create function
        let result = interp.exec("global my_table = table_create(data, headers)");
        assert!(result.is_ok(), "Failed to create table with table_create: {:?}", result);
        
        let table_value = interp.get_variable("my_table").unwrap();
        match table_value {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), 2);
                assert_eq!(table.column_names.len(), 3);
                assert!(table.column_names.contains(&"id".to_string()));
                assert!(table.column_names.contains(&"name".to_string()));
                assert!(table.column_names.contains(&"age".to_string()));
            }
            _ => panic!("Expected Table, got {:?}", table_value),
        }
    }

    #[test]
    fn test_table_create_without_headers() {
        let mut interp = Interpreter::new();
        
        // Create test data
        interp.set_variable("data".to_string(), Value::Array(vec![
            Value::Array(vec![
                Value::Number(1.0),
                Value::Number(25.0)
            ]),
            Value::Array(vec![
                Value::Number(2.0),
                Value::Number(30.0)
            ])
        ]), true);
        
        // Test table_create function without headers
        let result = interp.exec("global my_table = table_create(data)");
        assert!(result.is_ok(), "Failed to create table without headers: {:?}", result);
        
        let table_value = interp.get_variable("my_table").unwrap();
        match table_value {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), 2);
                assert_eq!(table.column_names.len(), 2);
                assert!(table.column_names.contains(&"Column_0".to_string()));
                assert!(table.column_names.contains(&"Column_1".to_string()));
            }
            _ => panic!("Expected Table, got {:?}", table_value),
        }
    }

    #[test]
    fn test_table_create_equivalence_with_table() {
        let mut interp = Interpreter::new();
        
        // Create test data
        interp.set_variable("data".to_string(), Value::Array(vec![
            Value::Array(vec![
                Value::Number(1.0),
                Value::Number(25.0)
            ]),
            Value::Array(vec![
                Value::Number(2.0),
                Value::Number(30.0)
            ])
        ]), true);
        
        // Create table with table() function
        let result1 = interp.exec("global table1 = table(data)");
        assert!(result1.is_ok());
        
        // Create table with table_create() function
        let result2 = interp.exec("global table2 = table_create(data)");
        assert!(result2.is_ok());
        
        // Both should create equivalent tables
        let table1 = interp.get_variable("table1").unwrap();
        let table2 = interp.get_variable("table2").unwrap();
        
        match (table1, table2) {
            (Value::Table(t1), Value::Table(t2)) => {
                assert_eq!(t1.rows.len(), t2.rows.len());
                assert_eq!(t1.column_names.len(), t2.column_names.len());
                assert_eq!(t1.column_names, t2.column_names);
            }
            _ => panic!("Both should be tables"),
        }
    }

    #[test]
    fn test_table_create_error_handling() {
        let mut interp = Interpreter::new();
        
        // Test with no arguments
        let result = interp.exec("global my_table = table_create()");
        assert!(result.is_err(), "table_create should fail with no arguments");
        
        // Test with wrong argument type
        interp.set_variable("not_array".to_string(), Value::String("not an array".to_string()), true);
        let result = interp.exec("global my_table = table_create(not_array)");
        assert!(result.is_err(), "table_create should fail with non-array argument");
    }

    #[test]
    fn test_table_create_with_mixed_data_types() {
        let mut interp = Interpreter::new();
        
        // Create test data with mixed types
        interp.set_variable("data".to_string(), Value::Array(vec![
            Value::Array(vec![
                Value::Number(1.0),
                Value::String("Alice".to_string()),
                Value::Bool(true),
                Value::Currency("$50000".to_string())
            ]),
            Value::Array(vec![
                Value::Number(2.0),
                Value::String("Bob".to_string()),
                Value::Bool(false),
                Value::Currency("$60000".to_string())
            ])
        ]), true);
        
        // Create headers
        interp.set_variable("headers".to_string(), Value::Array(vec![
            Value::String("id".to_string()),
            Value::String("name".to_string()),
            Value::String("active".to_string()),
            Value::String("salary".to_string())
        ]), true);
        
        // Test table_create function with mixed data types
        let result = interp.exec("global my_table = table_create(data, headers)");
        assert!(result.is_ok(), "Failed to create table with mixed data types: {:?}", result);
        
        let table_value = interp.get_variable("my_table").unwrap();
        match table_value {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), 2);
                assert_eq!(table.column_names.len(), 4);
                
                // Check first row data types
                let first_row = &table.rows[0];
                assert!(matches!(first_row[0], Value::Number(_)));
                assert!(matches!(first_row[1], Value::String(_)));
                assert!(matches!(first_row[2], Value::Bool(_)));
                assert!(matches!(first_row[3], Value::Currency(_)));
            }
            _ => panic!("Expected Table, got {:?}", table_value),
        }
    }
}
