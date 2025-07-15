use crate::value::Value;
use crate::error::{DataCodeError, Result};

/// Iteration and enumeration functions
pub fn call_iteration_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "enum" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("enum", 1, args.len(), line));
            }
            
            match &args[0] {
                Array(arr) => {
                    let enumerated: Vec<Value> = arr.iter()
                        .enumerate()
                        .map(|(i, value)| Array(vec![Number(i as f64), value.clone()]))
                        .collect();
                    Ok(Array(enumerated))
                }
                String(s) => {
                    let enumerated: Vec<Value> = s.chars()
                        .enumerate()
                        .map(|(i, ch)| Array(vec![Number(i as f64), String(ch.to_string())]))
                        .collect();
                    Ok(Array(enumerated))
                }
                Table(table) => {
                    // Enumerate table rows
                    let table_borrowed = table.borrow();
                    let enumerated: Vec<Value> = table_borrowed.rows.iter()
                        .enumerate()
                        .map(|(i, row)| Array(vec![Number(i as f64), Array(row.clone())]))
                        .collect();
                    Ok(Array(enumerated))
                }
                _ => Err(DataCodeError::type_error("Array, String, or Table", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to iteration functions
pub fn is_iteration_function(name: &str) -> bool {
    matches!(name, "enum")
}
