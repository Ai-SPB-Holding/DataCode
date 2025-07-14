use crate::value::{Value, Table as TableStruct, DataType};
use crate::error::{DataCodeError, Result};

/// Table core operations functions
pub fn call_table_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "table" | "table_create" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("table", 1, args.len(), line));
            }

            match &args[0] {
                Array(rows) => {
                    let headers = if args.len() > 1 {
                        match &args[1] {
                            Array(header_values) => {
                                header_values.iter()
                                    .map(|v| match v {
                                        String(s) => s.clone(),
                                        _ => format!("Column_{}", header_values.iter().position(|x| x == v).unwrap_or(0)),
                                    })
                                    .collect()
                            }
                            _ => return Err(DataCodeError::type_error("Array", "other", line)),
                        }
                    } else {
                        if let Some(Array(first_row)) = rows.first() {
                            (0..first_row.len()).map(|i| format!("Column_{}", i)).collect()
                        } else {
                            vec![]
                        }
                    };

                    let mut table = TableStruct::new(headers);
                    
                    for (row_index, row_value) in rows.iter().enumerate() {
                        match row_value {
                            Array(row_data) => {
                                if let Err(e) = table.add_row(row_data.clone()) {
                                    eprintln!("Warning: Row {}: {}", row_index + 1, e);
                                }
                            }
                            _ => {
                                return Err(DataCodeError::runtime_error(
                                    &format!("Row {} is not an array", row_index + 1),
                                    line
                                ));
                            }
                        }
                    }
                    
                    Ok(Table(table))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        "show_table" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("show_table", 1, args.len(), line));
            }
            match &args[0] {
                Value::Table(table) => {
                    print_table(table);
                    Ok(Value::Null)
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_info" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("table_info", 1, args.len(), line));
            }
            match &args[0] {
                Value::Table(table) => {
                    print_table_info(table);
                    Ok(Value::Null)
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_head" => {
            let n = if args.len() > 1 {
                match &args[1] {
                    Number(num) => *num as usize,
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            } else {
                5
            };

            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("table_head", 1, args.len(), line));
            }

            match &args[0] {
                Value::Table(table) => {
                    let mut head_table = TableStruct::new(table.column_names.clone());
                    for row in table.rows.iter().take(n) {
                        if let Err(e) = head_table.add_row(row.clone()) {
                            eprintln!("Warning: {}", e);
                        }
                    }
                    Ok(Value::Table(head_table))
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_tail" => {
            let n = if args.len() > 1 {
                match &args[1] {
                    Number(num) => *num as usize,
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            } else {
                5
            };

            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("table_tail", 1, args.len(), line));
            }

            match &args[0] {
                Value::Table(table) => {
                    let mut tail_table = TableStruct::new(table.column_names.clone());
                    let start_index = if table.rows.len() > n {
                        table.rows.len() - n
                    } else {
                        0
                    };
                    
                    for row in table.rows.iter().skip(start_index) {
                        if let Err(e) = tail_table.add_row(row.clone()) {
                            eprintln!("Warning: {}", e);
                        }
                    }
                    Ok(Value::Table(tail_table))
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_headers" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("table_headers", 1, args.len(), line));
            }

            match &args[0] {
                Value::Table(table) => {
                    let headers: Vec<Value> = table.column_names.iter()
                        .map(|name| String(name.clone()))
                        .collect();
                    Ok(Array(headers))
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_select" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_select", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Value::Table(table), Array(column_names)) => {
                    let selected_columns: Result<Vec<std::string::String>> = column_names.iter()
                        .map(|v| match v {
                            String(s) => Ok(s.clone()),
                            _ => Err(DataCodeError::type_error("Array of Strings", "other", line)),
                        })
                        .collect();
                    
                    let selected_columns = selected_columns?;
                    let mut selected_table = TableStruct::new(selected_columns.clone());
                    
                    let column_indices: Result<Vec<usize>> = selected_columns.iter()
                        .map(|col_name| {
                            table.column_names.iter()
                                .position(|name| name == col_name)
                                .ok_or_else(|| DataCodeError::runtime_error(
                                    &format!("Column '{}' not found", col_name),
                                    line
                                ))
                        })
                        .collect();
                    
                    let column_indices = column_indices?;
                    
                    for row in &table.rows {
                        let selected_row: Vec<Value> = column_indices.iter()
                            .map(|&index| row.get(index).cloned().unwrap_or(Value::Null))
                            .collect();
                        
                        if let Err(e) = selected_table.add_row(selected_row) {
                            eprintln!("Warning: {}", e);
                        }
                    }
                    
                    Ok(Value::Table(selected_table))
                }
                _ => Err(DataCodeError::type_error("Table and Array", "other", line)),
            }
        }

        "table_sort" => {
            if args.len() < 2 {
                return Err(DataCodeError::wrong_argument_count("table_sort", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Value::Table(table), String(column_name)) => {
                    sort_table_by_column(table, column_name, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to table functions
pub fn is_table_function(name: &str) -> bool {
    matches!(name,
        "table" | "table_create" | "show_table" | "table_info" | "table_head" | "table_tail" |
        "table_headers" | "table_select" | "table_sort"
    )
}

// Helper functions
fn print_table(table: &TableStruct) {
    if table.rows.is_empty() {
        println!("Empty table");
        return;
    }

    let mut max_widths = vec![0; table.column_names.len()];
    
    for (i, header) in table.column_names.iter().enumerate() {
        max_widths[i] = header.len();
    }
    
    for row in &table.rows {
        for (i, value) in row.iter().enumerate() {
            let formatted = format_value_for_table(value);
            if formatted.len() > max_widths[i] {
                max_widths[i] = formatted.len();
            }
        }
    }
    
    print!("┌");
    for (i, &width) in max_widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < max_widths.len() - 1 {
            print!("┬");
        }
    }
    println!("┐");
    
    print!("│");
    for (i, header) in table.column_names.iter().enumerate() {
        print!(" {:width$} ", header, width = max_widths[i]);
        if i < table.column_names.len() - 1 {
            print!("│");
        }
    }
    println!("│");
    
    print!("├");
    for (i, &width) in max_widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < max_widths.len() - 1 {
            print!("┼");
        }
    }
    println!("┤");
    
    for row in &table.rows {
        print!("│");
        for (i, value) in row.iter().enumerate() {
            let formatted = format_value_for_table(value);
            print!(" {:width$} ", formatted, width = max_widths[i]);
            if i < row.len() - 1 {
                print!("│");
            }
        }
        println!("│");
    }
    
    print!("└");
    for (i, &width) in max_widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < max_widths.len() - 1 {
            print!("┴");
        }
    }
    println!("┘");
}

fn print_table_info(table: &TableStruct) {
    println!("📊 Информация о таблице:");
    println!("   Строк: {}", table.rows.len());
    println!("   Колонок: {}", table.column_names.len());
    println!();
    println!("📋 Колонки:");
    
    for (i, column_name) in table.column_names.iter().enumerate() {
        let data_type = infer_column_type(table, i);
        let non_null_count = count_non_null_values(table, i);
        println!("   • {} ({:?}) - {} значений", column_name, data_type, non_null_count);
    }
}

fn format_value_for_table(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{:.2}", n)
            }
        }
        Value::String(s) => s.clone(),
        Value::Currency(c) => c.clone(),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::Null => "null".to_string(),
        Value::Array(arr) => format!("[{}]", arr.len()),
        Value::Object(obj) => format!("{{{}}}", obj.len()),
        Value::Table(table) => format!("Table({}x{})", table.rows.len(), table.columns.len()),
        Value::Path(p) => p.to_string_lossy().to_string(),
        Value::PathPattern(p) => format!("{}*", p.to_string_lossy()),
    }
}

fn infer_column_type(_table: &TableStruct, _column_index: usize) -> DataType {
    // Simple type inference - can be improved
    DataType::String
}

fn count_non_null_values(table: &TableStruct, column_index: usize) -> usize {
    table.rows.iter()
        .filter(|row| {
            row.get(column_index)
                .map(|v| !matches!(v, Value::Null))
                .unwrap_or(false)
        })
        .count()
}

fn sort_table_by_column(table: &TableStruct, column_name: &str, line: usize) -> Result<Value> {
    let col_index = table.column_names.iter()
        .position(|name| name == column_name)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Column '{}' not found", column_name),
            line
        ))?;

    let mut sorted_table = table.clone();
    sorted_table.rows.sort_by(|a, b| {
        let val_a = a.get(col_index).unwrap_or(&Value::Null);
        let val_b = b.get(col_index).unwrap_or(&Value::Null);
        
        match (val_a, val_b) {
            (Value::Number(n1), Value::Number(n2)) => n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
            (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
            (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
            (Value::Null, _) => std::cmp::Ordering::Less,
            (_, Value::Null) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });

    Ok(Value::Table(sorted_table))
}
