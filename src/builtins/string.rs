use crate::value::Value;
use crate::error::{DataCodeError, Result};

/// String operations functions
pub fn call_string_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "split" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("split", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (String(text), String(delimiter)) => {
                    let parts: Vec<Value> = text.split(delimiter)
                        .map(|s| String(s.to_string()))
                        .collect();
                    Ok(Array(parts))
                }
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        
        "join" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("join", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Array(arr), String(delimiter)) => {
                    let strings: Result<Vec<std::string::String>> = arr.iter()
                        .map(|v| match v {
                            String(s) => Ok(s.clone()),
                            _ => Err(DataCodeError::type_error("Array of Strings", "other", line)),
                        })
                        .collect();
                    Ok(String(strings?.join(delimiter)))
                }
                _ => Err(DataCodeError::type_error("Array and String", "other", line)),
            }
        }
        
        "trim" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("trim", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(String(s.trim().to_string())),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        
        "upper" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("upper", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(String(s.to_uppercase())),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        
        "lower" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("lower", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(String(s.to_lowercase())),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        
        "contains" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("contains", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (String(text), String(substring)) => {
                    Ok(Bool(text.contains(substring)))
                }
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to string functions
pub fn is_string_function(name: &str) -> bool {
    matches!(name, "split" | "join" | "trim" | "upper" | "lower" | "contains")
}
