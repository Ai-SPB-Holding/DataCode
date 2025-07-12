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
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}