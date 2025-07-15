use crate::value::Value;
use crate::error::{DataCodeError, Result};
use chrono::Utc;

/// Форматировать значение для вывода
fn format_value_for_print(value: &Value) -> String {
    use Value::*;
    match value {
        Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        String(s) => s.clone(),
        Bool(b) => b.to_string(),
        Currency(c) => c.clone(),
        Array(arr) => {
            let items: Vec<std::string::String> = arr.iter().map(format_value_for_print).collect();
            format!("[{}]", items.join(", "))
        }
        Object(obj) => {
            let items: Vec<std::string::String> = obj.iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_for_print(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        Table(table) => {
            let table_borrowed = table.borrow();
            format!("Table({} rows, {} columns)", table_borrowed.rows.len(), table_borrowed.column_names.len())
        }
        Null => "null".to_string(),
        Path(p) => p.display().to_string(),
        PathPattern(p) => format!("Pattern({})", p.display()),
    }
}

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
                .map(|v| format_value_for_print(&v))
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
                    match value {
                        Currency(_) => true,
                        String(s) => {
                            use crate::value::conversions::is_currency_string;
                            is_currency_string(s)
                        },
                        _ => false,
                    }
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

        "isset" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("isset", 1, args.len(), line));
            }

            // Для базовой реализации - проверяем, что значение не является Null
            // В будущем можно будет расширить для проверки существования переменных
            match &args[0] {
                Null => Ok(Bool(false)),
                _ => Ok(Bool(true)),
            }
        }

        "str" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("str", 1, args.len(), line));
            }
            Ok(String(format_value_for_print(&args[0])))
        }

        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to system functions
pub fn is_system_function(name: &str) -> bool {
    matches!(name, "now" | "print" | "getcwd" | "isinstance" | "isset" | "str")
}
