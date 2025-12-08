use data_code::value::Value;
use data_code::interpreter::Interpreter;

#[test]
fn test_merge_tables_basic() {
    let code = r#"
global table1 = table_create([[1, "Alice"], [2, "Bob"]], ["id", "name"])
global table2 = table_create([[3, "Charlie"], [4, "David"]], ["id", "name"])
global merged = merge_tables([table1, table2])
global result = len(merged)
result
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code).unwrap();
    
    match result {
        Value::Number(n) => assert_eq!(n, 4.0, "Merged table should have 4 rows"),
        _ => panic!("Expected number, got {:?}", result),
    }
}

#[test]
fn test_merge_tables_empty_array() {
    let code = r#"
global merged = merge_tables([])
merged == null
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code).unwrap();
    
    match result {
        Value::Boolean(b) => assert!(b, "merge_tables([]) should return null"),
        _ => panic!("Expected boolean, got {:?}", result),
    }
}

#[test]
fn test_merge_tables_single_table() {
    let code = r#"
global table1 = table_create([[1, "Alice"], [2, "Bob"]], ["id", "name"])
global merged = merge_tables([table1])
global result = len(merged)
result
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code).unwrap();
    
    match result {
        Value::Number(n) => assert_eq!(n, 2.0, "Merged table should have 2 rows"),
        _ => panic!("Expected number, got {:?}", result),
    }
}

#[test]
fn test_merge_tables_multiple_tables() {
    let code = r#"
global table1 = table_create([[1, "A"]], ["id", "name"])
global table2 = table_create([[2, "B"]], ["id", "name"])
global table3 = table_create([[3, "C"]], ["id", "name"])
global merged = merge_tables([table1, table2, table3])
global result = len(merged)
result
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code).unwrap();
    
    match result {
        Value::Number(n) => assert_eq!(n, 3.0, "Merged table should have 3 rows"),
        _ => panic!("Expected number, got {:?}", result),
    }
}

#[test]
fn test_merge_tables_different_headers_error() {
    let code = r#"
global table1 = table_create([[1, "Alice"]], ["id", "name"])
global table2 = table_create([[2, 30]], ["id", "age"])
global merged = merge_tables([table1, table2])
merged
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code);
    
    assert!(result.is_err(), "Should return error when headers don't match");
}

#[test]
fn test_merge_tables_preserves_headers() {
    let code = r#"
global table1 = table_create([[1, "Alice"]], ["id", "name"])
global table2 = table_create([[2, "Bob"]], ["id", "name"])
global merged = merge_tables([table1, table2])
global headers = table_headers(merged)
headers
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code).unwrap();
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 2, "Should have 2 headers");
            match (&arr[0], &arr[1]) {
                (Value::String(s1), Value::String(s2)) => {
                    assert_eq!(s1, "id");
                    assert_eq!(s2, "name");
                }
                _ => panic!("Headers should be strings"),
            }
        }
        _ => panic!("Expected array of headers, got {:?}", result),
    }
}

#[test]
fn test_merge_tables_with_invalid_rows() {
    let code = r#"
global table1 = table_create([[1, "Alice"], [2]], ["id", "name"])
global table2 = table_create([[3, "Bob"]], ["id", "name"])
global merged = merge_tables([table1, table2])
global result = len(merged)
result
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code).unwrap();
    
    // Should skip invalid row [2] and merge valid rows
    match result {
        Value::Number(n) => assert_eq!(n, 2.0, "Merged table should have 2 valid rows"),
        _ => panic!("Expected number, got {:?}", result),
    }
}

#[test]
fn test_merge_tables_large_dataset() {
    let code = r#"
global table1 = table_create([[i, "Row" + i] for i in range(1, 101)], ["id", "name"])
global table2 = table_create([[i, "Row" + i] for i in range(101, 201)], ["id", "name"])
global merged = merge_tables([table1, table2])
global result = len(merged)
result
"#;

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code).unwrap();
    
    match result {
        Value::Number(n) => assert_eq!(n, 200.0, "Merged table should have 200 rows"),
        _ => panic!("Expected number, got {:?}", result),
    }
}

