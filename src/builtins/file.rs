use crate::value::{Value, Table, DataType};
use crate::error::{DataCodeError, Result};
use std::fs;
use std::path::PathBuf;
use glob::glob;

/// File operations functions
pub fn call_file_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
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
                Value::PathPattern(pattern) => {
                    let pattern_str = pattern.to_string_lossy();
                    let mut files = vec![];
                    
                    for entry in glob(&pattern_str).map_err(|e| 
                        DataCodeError::runtime_error(&format!("Invalid glob pattern: {}", e), line))? {
                        match entry {
                            Ok(path) => {
                                if path.is_file() {
                                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                        files.push(String(name.to_string()));
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Error reading file: {}", e);
                            }
                        }
                    }
                    Ok(Array(files))
                }
                _ => Err(DataCodeError::type_error("Path or PathPattern", "other", line)),
            }
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
                            read_csv_file(p, line)
                        }
                        "xlsx" => {
                            read_xlsx_file(p, line)
                        }
                        _ => Err(DataCodeError::runtime_error(&format!("Unsupported file extension: {}", ext), line)),
                    }
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }
        
        "analyze_csv" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("analyze_csv", 1, args.len(), line));
            }
            match &args[0] {
                Value::Path(p) => {
                    analyze_csv_file(p, line)
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }
        
        "read_csv_safe" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("read_csv_safe", 1, args.len(), line));
            }
            match &args[0] {
                Value::Path(p) => {
                    read_csv_safe_file(p, line)
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to file functions
pub fn is_file_function(name: &str) -> bool {
    matches!(name, "path" | "list_files" | "read_file" | "analyze_csv" | "read_csv_safe")
}

// Helper functions for file operations
fn read_csv_file(p: &std::path::Path, line: usize) -> Result<Value> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(p)
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV: {}", e), line))?;

    let headers: Vec<String> = rdr.headers()
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read headers: {}", e), line))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut table = Table::new(headers);
    let mut warnings = Vec::new();

    for (row_index, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| DataCodeError::runtime_error(&format!("Failed to read row {}: {}", row_index + 1, e), line))?;
        
        let mut row_values = Vec::new();
        for (col_index, field) in record.iter().enumerate() {
            let value = crate::value::parse_value_with_type_inference(field.trim());
            row_values.push(value);
        }
        
        if let Err(e) = table.add_row(row_values) {
            warnings.push(format!("Row {}: {}", row_index + 1, e));
        }
    }

    for warning in warnings {
        eprintln!("⚠️  {}", warning);
    }

    Ok(Value::Table(table))
}

fn read_xlsx_file(p: &std::path::Path, line: usize) -> Result<Value> {
    use calamine::{Reader, open_workbook, Xlsx};
    let mut workbook: Xlsx<_> = open_workbook(p)
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to open xlsx: {}", e), line))?;
    let range = workbook.worksheet_range_at(0)
        .ok_or_else(|| DataCodeError::runtime_error("No sheets found", line))?
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read sheet: {}", e), line))?;

    let mut rows = range.rows();
    let headers: Vec<String> = if let Some(header_row) = rows.next() {
        header_row.iter().map(|cell| {
            match cell {
                calamine::Data::String(s) => s.clone(),
                calamine::Data::Float(f) => f.to_string(),
                calamine::Data::Int(i) => i.to_string(),
                _ => "Column".to_string(),
            }
        }).collect()
    } else {
        return Err(DataCodeError::runtime_error("Empty Excel file", line));
    };

    let mut table = Table::new(headers);
    
    for row in rows {
        let row_values: Vec<Value> = row.iter()
            .map(|cell| parse_excel_value(cell))
            .collect();
        
        if let Err(e) = table.add_row(row_values) {
            eprintln!("Warning: {}", e);
        }
    }

    Ok(Value::Table(table))
}

fn analyze_csv_file(p: &std::path::Path, line: usize) -> Result<Value> {
    // Implementation for CSV analysis
    // This is a placeholder - you can implement detailed CSV analysis here
    Ok(Value::String("CSV analysis not yet implemented".to_string()))
}

fn read_csv_safe_file(p: &std::path::Path, line: usize) -> Result<Value> {
    // Implementation for safe CSV reading
    // This is a placeholder - you can implement safe CSV reading here
    read_csv_file(p, line)
}

fn parse_excel_value(cell: &calamine::Data) -> Value {
    match cell {
        calamine::Data::Empty => Value::Null,
        calamine::Data::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Value::Null
            } else {
                Value::String(trimmed.to_string())
            }
        }
        calamine::Data::Float(f) => Value::Number(*f),
        calamine::Data::Int(i) => Value::Number(*i as f64),
        calamine::Data::Bool(b) => Value::Bool(*b),
        calamine::Data::DateTime(dt) => Value::String(dt.to_string()),
        calamine::Data::DateTimeIso(dt) => Value::String(dt.clone()),
        calamine::Data::DurationIso(dur) => Value::String(dur.clone()),
        calamine::Data::Error(e) => Value::String(format!("ERROR: {:?}", e)),
    }
}
