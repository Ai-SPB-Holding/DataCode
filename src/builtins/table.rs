use crate::value::{Value, Table as TableStruct, DataType, LazyTable};
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
                    
                    Ok(Value::table(table))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        "show_table" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("show_table", 1, args.len(), line));
            }
            match &args[0] {
                Value::Table(table_rc) => {
                    let table_borrowed = table_rc.borrow();
                    print_table(&*table_borrowed);
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
                Value::Table(table_rc) => {
                    let table_borrowed = table_rc.borrow();
                    print_table_info(&*table_borrowed);
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
                Value::Table(table_rc) => {
                    // Используем ленивую обработку для head операции
                    let lazy_table = LazyTable::new(table_rc.clone()).head(n, line);
                    match lazy_table.materialize() {
                        Ok(materialized_table) => Ok(Value::table(materialized_table)),
                        Err(e) => Err(e),
                    }
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
                Value::Table(table_rc) => {
                    // Используем ленивую обработку для tail операции
                    let lazy_table = LazyTable::new(table_rc.clone()).tail(n, line);
                    match lazy_table.materialize() {
                        Ok(materialized_table) => Ok(Value::table(materialized_table)),
                        Err(e) => Err(e),
                    }
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_headers" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("table_headers", 1, args.len(), line));
            }

            match &args[0] {
                Value::Table(table_rc) => {
                    let table_borrowed = table_rc.borrow();
                    let headers: Vec<Value> = table_borrowed.column_names.iter()
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
                (Value::Table(table_rc), Array(column_names)) => {
                    let selected_columns: Result<Vec<std::string::String>> = column_names.iter()
                        .map(|v| match v {
                            String(s) => Ok(s.clone()),
                            _ => Err(DataCodeError::type_error("Array of Strings", "other", line)),
                        })
                        .collect();

                    let selected_columns = selected_columns?;

                    // Используем ленивую обработку для select операции
                    let lazy_table = LazyTable::new(table_rc.clone()).select(selected_columns, line);
                    match lazy_table.materialize() {
                        Ok(materialized_table) => Ok(Value::table(materialized_table)),
                        Err(e) => Err(e),
                    }
                }
                _ => Err(DataCodeError::type_error("Table and Array", "other", line)),
            }
        }

        "table_sort" => {
            if args.len() < 2 {
                return Err(DataCodeError::wrong_argument_count("table_sort", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), String(column_name)) => {
                    let table_borrowed = table_rc.borrow();
                    sort_table_by_column(&*table_borrowed, column_name, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_filter" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_filter", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), String(condition)) => {
                    filter_table_with_expression(table_rc, condition, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_where" => {
            if args.len() != 4 {
                return Err(DataCodeError::wrong_argument_count("table_where", 4, args.len(), line));
            }
            match (&args[0], &args[1], &args[2], &args[3]) {
                (Value::Table(table_rc), String(column), String(operator), value) => {
                    filter_table_where(table_rc, column, operator, value, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, String, Value", "other", line)),
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
        Value::Table(table) => {
            let table_borrowed = table.borrow();
            format!("Table({}x{})", table_borrowed.rows.len(), table_borrowed.columns.len())
        },
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

    Ok(Value::table(sorted_table))
}

/// Фильтрация таблицы по условию WHERE
fn filter_table_where(table_rc: &std::rc::Rc<std::cell::RefCell<TableStruct>>,
                     column: &str,
                     operator: &str,
                     value: &Value,
                     line: usize) -> Result<Value> {
    let table_borrowed = table_rc.borrow();

    // Найти индекс колонки
    let column_index = table_borrowed.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Column '{}' not found", column),
            line
        ))?;

    // Создать новую таблицу с теми же заголовками
    let mut filtered_table = TableStruct::new(table_borrowed.column_names.clone());

    // Фильтровать строки
    for row in &table_borrowed.rows {
        if let Some(cell_value) = row.get(column_index) {
            let matches = match operator {
                "=" | "==" => values_equal(cell_value, value),
                "!=" | "<>" => !values_equal(cell_value, value),
                ">" => compare_values(cell_value, value) == std::cmp::Ordering::Greater,
                "<" => compare_values(cell_value, value) == std::cmp::Ordering::Less,
                ">=" => {
                    let cmp = compare_values(cell_value, value);
                    cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal
                }
                "<=" => {
                    let cmp = compare_values(cell_value, value);
                    cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal
                }
                _ => return Err(DataCodeError::runtime_error(
                    &format!("Unsupported operator: {}", operator),
                    line
                )),
            };

            if matches {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    eprintln!("Warning: Failed to add filtered row: {}", e);
                }
            }
        }
    }

    Ok(Value::table(filtered_table))
}

/// Сравнить два значения для равенства
fn values_equal(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
        (String(a), String(b)) => a == b,
        (Bool(a), Bool(b)) => a == b,
        (Null, Null) => true,
        _ => false,
    }
}

/// Сравнить два значения для упорядочивания
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use Value::*;
    match (a, b) {
        (Number(a), Number(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (String(a), String(b)) => a.cmp(b),
        (Bool(a), Bool(b)) => a.cmp(b),
        _ => std::cmp::Ordering::Equal,
    }
}

/// Фильтрация таблицы с выражением (простая реализация)
fn filter_table_with_expression(table_rc: &std::rc::Rc<std::cell::RefCell<TableStruct>>,
                               condition: &str,
                               line: usize) -> Result<Value> {
    // Простой парсер для условий вида "column operator value"
    let parts: Vec<&str> = condition.split_whitespace().collect();

    if parts.len() != 3 {
        return Err(DataCodeError::runtime_error(
            &format!("Invalid filter condition: '{}'. Expected format: 'column operator value'", condition),
            line
        ));
    }

    let column = parts[0];
    let operator = parts[1];
    let value_str = parts[2];

    // Попытаться распарсить значение
    let value = if let Ok(num) = value_str.parse::<f64>() {
        Value::Number(num)
    } else if value_str == "true" {
        Value::Bool(true)
    } else if value_str == "false" {
        Value::Bool(false)
    } else if value_str == "null" {
        Value::Null
    } else {
        // Убрать кавычки, если есть
        let cleaned = value_str.trim_matches('\'').trim_matches('"');
        Value::String(cleaned.to_string())
    };

    filter_table_where(table_rc, column, operator, &value, line)
}
