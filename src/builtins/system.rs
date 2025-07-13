use crate::value::Value;
use crate::error::{DataCodeError, Result};
use std::path::PathBuf;
use chrono::Utc;

/// System and utility functions
pub fn call_system_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "now" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("now", 0, args.len(), line));
            }
            Ok(String(Utc::now().to_rfc3339()))
        }
        
        "print" => {
            let parts: Vec<std::string::String> = args.into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    Value::Currency(c) => c,
                    Value::Path(p) => p.display().to_string(),
                    Value::PathPattern(p) => format!("{}*", p.display()),
                    Value::Number(n) => {
                        if n.fract() == 0.0 {
                            format!("{}", n as i64)
                        } else {
                            format!("{}", n)
                        }
                    }
                    Value::Bool(b) => if b { "true".to_string() } else { "false".to_string() },
                    Value::Null => "null".to_string(),
                    Value::Array(arr) => format!("[{}]", arr.len()),
                    Value::Object(obj) => format!("{{{}}}", obj.len()),
                    Value::Table(table) => format!("Table({}x{})", table.rows.len(), table.columns.len()),
                })
                .collect();
            println!("{}", parts.join(" "));
            Ok(Value::Null)
        }
        
        "getcwd" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("getcwd", 0, args.len(), line));
            }
            let cwd = std::env::current_dir()
                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to get current dir: {}", e), line))?;
            Ok(Value::Path(cwd))
        }
        
        "isinstance" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("isinstance", 2, args.len(), line));
            }

            let value = &args[0];
            let type_name = match &args[1] {
                String(s) => s.as_str(),
                _ => return Err(DataCodeError::type_error("String", "other", line)),
            };

            let is_instance = match type_name.to_lowercase().as_str() {
                "number" | "num" | "float" | "double" => {
                    matches!(value, Number(_))
                }
                "int" | "integer" => {
                    match value {
                        Number(n) => n.fract() == 0.0,
                        _ => false,
                    }
                }
                "string" | "str" | "text" => {
                    matches!(value, String(_))
                }
                "bool" | "boolean" => {
                    matches!(value, Bool(_))
                }
                "array" | "list" | "vec" => {
                    matches!(value, Array(_))
                }
                "object" | "dict" | "map" => {
                    matches!(value, Object(_))
                }
                "table" => {
                    matches!(value, Table(_))
                }
                "currency" | "money" => {
                    matches!(value, Currency(_))
                }
                "null" | "none" => {
                    matches!(value, Null)
                }
                "path" => {
                    matches!(value, Path(_))
                }
                "pathpattern" | "pattern" => {
                    matches!(value, PathPattern(_))
                }
                _ => return Err(DataCodeError::runtime_error(&format!("Unknown type: {}", type_name), line)),
            };

            Ok(Bool(is_instance))
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to system functions
pub fn is_system_function(name: &str) -> bool {
    matches!(name, "now" | "print" | "getcwd" | "isinstance")
}
