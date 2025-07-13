use crate::value::Value;
use crate::error::{DataCodeError, Result};

/// Array operations functions
pub fn call_array_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "length" | "len" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count(name, 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                Currency(c) => Ok(Number(c.len() as f64)),
                Table(table) => Ok(Number(table.rows.len() as f64)),
                _ => Err(DataCodeError::type_error("Array, String, Currency, or Table", "other", line)),
            }
        }
        
        "push" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("push", 2, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(args[1].clone());
                    Ok(Array(new_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "pop" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("pop", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    if arr.is_empty() {
                        Ok(Null)
                    } else {
                        Ok(arr[arr.len() - 1].clone())
                    }
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "append" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("append", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Array(arr), value) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(value.clone());
                    Ok(Array(new_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "sort" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sort", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut sorted_arr = arr.clone();
                    sorted_arr.sort_by(|a, b| {
                        match (a, b) {
                            (Number(n1), Number(n2)) => n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal),
                            (String(s1), String(s2)) => s1.cmp(s2),
                            _ => std::cmp::Ordering::Equal,
                        }
                    });
                    Ok(Array(sorted_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "unique" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("unique", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut unique_items = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    
                    for item in arr {
                        let key = format!("{:?}", item);
                        if seen.insert(key) {
                            unique_items.push(item.clone());
                        }
                    }
                    
                    Ok(Array(unique_items))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "array" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("array", 0, args.len(), line));
            }
            Ok(Array(vec![]))
        }
        
        "sum" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sum", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut total = 0.0;
                    for item in arr {
                        match item {
                            Number(n) => total += n,
                            _ => return Err(DataCodeError::type_error("Array of Numbers", "other", line)),
                        }
                    }
                    Ok(Number(total))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "average" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("average", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    if arr.is_empty() {
                        return Err(DataCodeError::runtime_error("Cannot calculate average of empty array", line));
                    }
                    let mut total = 0.0;
                    for item in arr {
                        match item {
                            Number(n) => total += n,
                            _ => return Err(DataCodeError::type_error("Array of Numbers", "other", line)),
                        }
                    }
                    Ok(Number(total / arr.len() as f64))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        
        "count" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("count", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                Table(table) => Ok(Number(table.rows.len() as f64)),
                _ => Err(DataCodeError::type_error("Array, String, or Table", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to array functions
pub fn is_array_function(name: &str) -> bool {
    matches!(name, 
        "length" | "len" | "push" | "pop" | "append" | "sort" | 
        "unique" | "array" | "sum" | "average" | "count"
    )
}
