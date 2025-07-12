use crate::value::Value;
use std::fs;
use std::path::PathBuf;
use std::env;
use chrono::Utc;

pub fn call_function(name: &str, args: Vec<Value>) -> Result<Value, String> {
    use Value::*;
    match name {
        "now" => {
            if !args.is_empty() {
                return Err("now() takes no arguments".to_string());
            }
            Ok(String(Utc::now().to_rfc3339()))
        }
        "path" => {
            if args.len() != 1 {
                return Err("path() expects exactly 1 argument".to_string());
            }
            match &args[0] {
                String(s) => Ok(Path(PathBuf::from(s))),
                _ => Err("path() argument must be a string".to_string()),
            }
        }
        "list_files" => {
            if args.len() != 1 {
                return Err("list_files() expects exactly 1 argument".to_string());
            }
            match &args[0] {
                Path(p) => {
                    let entries = fs::read_dir(p).map_err(|e| format!("Failed to read dir: {}", e))?;
                    let mut files = vec![];
                    for entry in entries {
                        let entry = entry.map_err(|e| e.to_string())?;
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
                _ => Err("list_files() argument must be a path".to_string()),
            }
        }
        "getcwd" => {
            if !args.is_empty() {
                return Err("getcwd() takes no arguments".to_string());
            }
            let cwd = std::env::current_dir()
                .map_err(|e| format!("Failed to get current dir: {}", e))?;
            Ok(Value::Path(cwd))
        }
        "read_file" => {
            if args.len() != 1 {
                return Err("read_file() expects exactly 1 argument".to_string());
            }

            match &args[0] {
                Value::Path(p) => {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    match ext.as_str() {
                        "txt" => {
                            let contents = std::fs::read_to_string(p)
                                .map_err(|e| format!("Failed to read file: {}", e))?;
                            Ok(Value::String(contents))
                        }
                        "csv" => {
                            let mut rdr = csv::Reader::from_path(p)
                                .map_err(|e| format!("Failed to read CSV: {}", e))?;
                            let mut rows = vec![];
                            for result in rdr.records() {
                                let record = result.map_err(|e| e.to_string())?;
                                let row = record.iter().map(|s| Value::String(s.to_string())).collect();
                                rows.push(Value::Array(row));
                            }
                            Ok(Value::Array(rows))
                        }
                        "xlsx" => {
                            use calamine::{Reader, open_workbook, Xlsx};
                            let mut workbook: Xlsx<_> = open_workbook(p)
                                .map_err(|e| format!("Failed to open xlsx: {}", e))?;
                            let range = workbook.worksheet_range_at(0)
                                .ok_or("No sheets found")?
                                .map_err(|e| format!("Sheet error: {}", e))?;

                            let rows = range.rows().map(|row| {
                                Value::Array(row.iter().map(|cell| {
                                    Value::String(cell.to_string())
                                }).collect())
                            }).collect();
                            Ok(Value::Array(rows))
                        }
                        _ => Err("Unsupported file extension".to_string()),
                    }
                }
                _ => Err("read_file() expects a path".to_string()),
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
        _ => Err(format!("Unknown built-in function: {}", name)),
    }
}