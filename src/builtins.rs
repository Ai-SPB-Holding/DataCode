use crate::value::Value;
use crate::error::{DataCodeError, Result};
use std::fs;
use std::path::PathBuf;
use std::env;
use chrono::Utc;

pub fn call_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;
    match name {
        "now" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("now", 0, args.len(), line));
            }
            Ok(String(Utc::now().to_rfc3339()))
        }
        "path" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("path", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(Path(PathBuf::from(s))),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        "list_files" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("list_files", 1, args.len(), line));
            }
            match &args[0] {
                Path(p) => {
                    let entries = fs::read_dir(p).map_err(|e|
                        DataCodeError::runtime_error(&format!("Failed to read dir: {}", e), line))?;
                    let mut files = vec![];
                    for entry in entries {
                        let entry = entry.map_err(|e| DataCodeError::runtime_error(&e.to_string(), line))?;
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                if let Some(name) = entry.file_name().to_str() {
                                    files.push(String(name.to_string()));
                                }
                            }
                        }
                    }
                    Ok(Array(files))
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }
        "getcwd" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("getcwd", 0, args.len(), line));
            }
            let cwd = std::env::current_dir()
                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to get current dir: {}", e), line))?;
            Ok(Value::Path(cwd))
        }
        "read_file" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("read_file", 1, args.len(), line));
            }

            match &args[0] {
                Value::Path(p) => {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    match ext.as_str() {
                        "txt" => {
                            let contents = std::fs::read_to_string(p)
                                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read file: {}", e), line))?;
                            Ok(Value::String(contents))
                        }
                        "csv" => {
                            let mut rdr = csv::Reader::from_path(p)
                                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV: {}", e), line))?;
                            let mut rows = vec![];

                            // Добавляем заголовки
                            if let Ok(headers) = rdr.headers() {
                                let header_row = headers.iter().map(|s| Value::String(s.to_string())).collect();
                                rows.push(Value::Array(header_row));
                            }

                            // Добавляем данные
                            for result in rdr.records() {
                                let record = result.map_err(|e| DataCodeError::runtime_error(&e.to_string(), line))?;
                                let row = record.iter().map(|s| Value::String(s.to_string())).collect();
                                rows.push(Value::Array(row));
                            }
                            Ok(Value::Array(rows))
                        }
                        "xlsx" => {
                            use calamine::{Reader, open_workbook, Xlsx};
                            let mut workbook: Xlsx<_> = open_workbook(p)
                                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to open xlsx: {}", e), line))?;
                            let range = workbook.worksheet_range_at(0)
                                .ok_or_else(|| DataCodeError::runtime_error("No sheets found", line))?
                                .map_err(|e| DataCodeError::runtime_error(&format!("Sheet error: {}", e), line))?;

                            let rows = range.rows().map(|row| {
                                Value::Array(row.iter().map(|cell| {
                                    Value::String(cell.to_string())
                                }).collect())
                            }).collect();
                            Ok(Value::Array(rows))
                        }
                        _ => Err(DataCodeError::runtime_error("Unsupported file extension", line)),
                    }
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }
        "print" => {
            let parts: Vec<std::string::String> = args.into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    Value::Path(p) => p.display().to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Array(_) | Value::Object(_) => format!("{:?}", v),
                })
                .collect();
            println!("{}", parts.join(" "));
            Ok(Value::Null)
        }

        // Математические функции
        "abs" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("abs", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(n.abs())),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sqrt", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => {
                    if *n < 0.0 {
                        Err(DataCodeError::runtime_error("Cannot take square root of negative number", line))
                    } else {
                        Ok(Number(n.sqrt()))
                    }
                }
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "pow" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("pow", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Number(base), Number(exp)) => Ok(Number(base.powf(*exp))),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "min" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("min", 1, 0, line));
            }
            let mut min_val = match &args[0] {
                Number(n) => *n,
                _ => return Err(DataCodeError::type_error("Number", "other", line)),
            };
            for arg in &args[1..] {
                match arg {
                    Number(n) => {
                        if *n < min_val {
                            min_val = *n;
                        }
                    }
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            }
            Ok(Number(min_val))
        }
        "max" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("max", 1, 0, line));
            }
            let mut max_val = match &args[0] {
                Number(n) => *n,
                _ => return Err(DataCodeError::type_error("Number", "other", line)),
            };
            for arg in &args[1..] {
                match arg {
                    Number(n) => {
                        if *n > max_val {
                            max_val = *n;
                        }
                    }
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            }
            Ok(Number(max_val))
        }
        "round" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("round", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(n.round())),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }

        // Функции для работы с массивами
        "length" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("length", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                _ => Err(DataCodeError::type_error("Array or String", "other", line)),
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
                        Ok(Value::Null)
                    } else {
                        Ok(arr[arr.len() - 1].clone())
                    }
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
                            (Number(x), Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            (String(x), String(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        }
                    });
                    Ok(Array(sorted_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        // Функции для работы со строками
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
                    let mut strings = Vec::new();
                    for v in arr {
                        match v {
                            String(s) => strings.push(s.clone()),
                            _ => return Err(DataCodeError::type_error("Array of Strings", "other", line)),
                        }
                    }
                    Ok(String(strings.join(delimiter)))
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

        // Функции агрегации данных
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
                        return Ok(Number(0.0));
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
                _ => Err(DataCodeError::type_error("Array or String", "other", line)),
            }
        }
        "unique" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("unique", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut unique_items = Vec::new();
                    for item in arr {
                        if !unique_items.iter().any(|existing| {
                            match (existing, item) {
                                (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
                                (String(a), String(b)) => a == b,
                                (Bool(a), Bool(b)) => a == b,
                                _ => false,
                            }
                        }) {
                            unique_items.push(item.clone());
                        }
                    }
                    Ok(Array(unique_items))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        "len" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("len", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                _ => Err(DataCodeError::type_error("Array or String", "other", line)),
            }
        }
        "div" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("div", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Number(a), Number(b)) => {
                    if *b == 0.0 {
                        Err(DataCodeError::runtime_error("Division by zero", line))
                    } else {
                        Ok(Number(a / b))
                    }
                }
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "array" => {
            if args.len() != 0 {
                return Err(DataCodeError::wrong_argument_count("array", 0, args.len(), line));
            }
            Ok(Array(vec![]))
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

        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}