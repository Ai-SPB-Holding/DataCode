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
            let output = parts.join(" ");
            
            // Используем OutputCapture, который сам решает, куда писать
            // Если перехват активен - пишет в буфер, иначе в stdout
            use crate::websocket::output_capture::OutputCapture;
            OutputCapture::write_output(&output);
            
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
            // Поддерживаем как строки, так и переменные с именами типов
            // В DataCode можно использовать isinstance(value, num) где num - это имя типа (не переменная)
            // Если переменная не определена (Null), это означает, что использовалось имя типа напрямую
            let type_name: std::string::String = match &args[1] {
                String(s) => s.clone(),
                // Если передана переменная, преобразуем её в строку
                // Если это Null (неопределенная переменная), это означает, что использовалось имя типа напрямую
                // В этом случае мы не можем узнать исходное имя, поэтому возвращаем ошибку
                // Но на самом деле, если переменная не определена, должна быть ошибка "variable not found"
                // Значит, если мы получили Null, это может быть только если переменная была явно установлена в null
                Null => {
                    // Если это Null, это может быть либо неопределенная переменная (ошибка должна была быть раньше),
                    // либо переменная со значением null. В последнем случае, null - это валидный тип для проверки
                    "null".to_string()
                }
                other => {
                    let type_str = format_value_for_print(other);
                    // Удаляем кавычки если они есть
                    let trimmed = type_str.trim_matches('"').trim_matches('\'');
                    // Если это известное имя типа, используем его
                    match trimmed.to_lowercase().as_str() {
                        "int" | "integer" => "int".to_string(),
                        "float" | "number" | "num" => "number".to_string(),
                        "str" | "string" | "text" => "string".to_string(),
                        "bool" | "boolean" => "bool".to_string(),
                        "date" | "datetime" => "date".to_string(),
                        "money" | "currency" => "money".to_string(),
                        "array" | "list" | "vec" => "array".to_string(),
                        "object" | "dict" | "map" => "object".to_string(),
                        "table" => "table".to_string(),
                        "null" | "none" => "null".to_string(),
                        "path" => "path".to_string(),
                        "pathpattern" | "pattern" => "pathpattern".to_string(),
                        // Если это не известное имя типа, проверяем, может быть это число или другое значение
                        // В этом случае это ошибка - второй аргумент должен быть именем типа
                        _ => {
                            // Если trimmed выглядит как число или другое значение (не имя типа), это ошибка
                            if trimmed.parse::<f64>().is_ok() || trimmed == "true" || trimmed == "false" {
                                return Err(DataCodeError::runtime_error(
                                    &format!("isinstance: second argument must be a type name, got value: {}", trimmed),
                                    line
                                ));
                            }
                            // Иначе используем как есть (может быть пользовательский тип)
                            trimmed.to_string()
                        }
                    }
                }
            };

            let is_instance = match type_name.as_str().to_lowercase().as_str() {
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
                "date" | "datetime" => {
                    match value {
                        String(s) => {
                            use crate::value::conversions::is_date_string;
                            is_date_string(s)
                        },
                        _ => false,
                    }
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

        "int" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("int", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(n.trunc())),
                String(s) => {
                    use crate::value::conversions::try_parse_number;
                    match try_parse_number(s) {
                        Some(n) => Ok(Number(n.trunc())),
                        None => Err(DataCodeError::runtime_error(
                            &format!("Cannot convert '{}' to integer", s),
                            line,
                        )),
                    }
                }
                Bool(b) => Ok(Number(if *b { 1.0 } else { 0.0 })),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot convert {:?} to integer", args[0]),
                    line,
                )),
            }
        }

        "float" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("float", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(*n)),
                String(s) => {
                    use crate::value::conversions::try_parse_number;
                    match try_parse_number(s) {
                        Some(n) => Ok(Number(n)),
                        None => Err(DataCodeError::runtime_error(
                            &format!("Cannot convert '{}' to float", s),
                            line,
                        )),
                    }
                }
                Bool(b) => Ok(Number(if *b { 1.0 } else { 0.0 })),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot convert {:?} to float", args[0]),
                    line,
                )),
            }
        }

        "bool" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("bool", 1, args.len(), line));
            }
            match &args[0] {
                Bool(b) => Ok(Bool(*b)),
                Number(n) => Ok(Bool(*n != 0.0)),
                String(s) => {
                    use crate::value::conversions::try_parse_bool;
                    match try_parse_bool(s) {
                        Some(b) => Ok(Bool(b)),
                        None => Ok(Bool(!s.is_empty())),
                    }
                }
                Null => Ok(Bool(false)),
                Array(arr) => Ok(Bool(!arr.is_empty())),
                Object(obj) => Ok(Bool(!obj.is_empty())),
                Table(table) => Ok(Bool(!table.borrow().rows.is_empty())),
                _ => Ok(Bool(true)),
            }
        }

        "date" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("date", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => {
                    // Проверяем, является ли строка датой
                    use crate::value::conversions::is_date_string;
                    if is_date_string(s) {
                        Ok(String(s.clone()))
                    } else {
                        Err(DataCodeError::runtime_error(
                            &format!("Cannot parse '{}' as date", s),
                            line,
                        ))
                    }
                }
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot convert {:?} to date", args[0]),
                    line,
                )),
            }
        }

        "money" => {
            if args.len() < 1 || args.len() > 2 {
                return Err(DataCodeError::wrong_argument_count("money", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => {
                    // Если передан формат, используем его (пока просто возвращаем строку)
                    // В будущем можно добавить форматирование
                    if args.len() == 2 {
                        match &args[1] {
                            String(format) => {
                                // Простая реализация - просто возвращаем строку с форматированием
                                // В будущем можно добавить реальное форматирование
                                Ok(String(format!("{} {}", s, format)))
                            }
                            _ => Err(DataCodeError::type_error("String", "other", line)),
                        }
                    } else {
                        // Проверяем, является ли строка валютой
                        use crate::value::conversions::is_currency_string;
                        if is_currency_string(s) {
                            Ok(Value::Currency(s.clone()))
                        } else {
                            // Пытаемся создать валюту из числа
                            use crate::value::conversions::try_parse_number;
                            match try_parse_number(s) {
                                Some(n) => Ok(Value::Currency(format!("${:.2}", n))),
                                None => Err(DataCodeError::runtime_error(
                                    &format!("Cannot convert '{}' to money", s),
                                    line,
                                )),
                            }
                        }
                    }
                }
                Number(n) => {
                    // Преобразуем число в валюту
                    if args.len() == 2 {
                        match &args[1] {
                            String(format) => Ok(Value::Currency(format!("{} {}", n, format))),
                            _ => Err(DataCodeError::type_error("String", "other", line)),
                        }
                    } else {
                        Ok(Value::Currency(format!("${:.2}", n)))
                    }
                }
                Currency(c) => Ok(Value::Currency(c.clone())),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot convert {:?} to money", args[0]),
                    line,
                )),
            }
        }

        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to system functions
pub fn is_system_function(name: &str) -> bool {
    matches!(name, "now" | "print" | "getcwd" | "isinstance" | "isset" | "str" | "int" | "float" | "bool" | "date" | "money")
}
