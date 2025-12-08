use crate::value::{Value, Table as TableStruct, DataType, LazyTable};
use crate::value::relations::{Relation, add_relation};
use crate::error::{DataCodeError, Result};

/// Вспомогательная функция для вывода с перехватом через WebSocket
fn output_line(line: &str) {
    use crate::websocket::output_capture::OutputCapture;
    OutputCapture::write_output(line);
}


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
                    let start_time = std::time::Instant::now();
                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG table_create: Начало создания таблицы, строк: {}", rows.len());
                    }

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

                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG table_create: Заголовки созданы ({} колонок), время: {:?}", headers.len(), start_time.elapsed());
                    }

                    let mut table = TableStruct::new(headers);

                    // Phase 1 Optimization: Collect all rows first, then add in bulk
                    let mut processed_rows = Vec::with_capacity(rows.len());
                    let process_start = std::time::Instant::now();

                    for (row_index, row_value) in rows.iter().enumerate() {
                        match row_value {
                            Array(row_data) => {
                                processed_rows.push(row_data.clone());
                                
                                // Выводим прогресс каждые 5000 строк
                                if std::env::var("DATACODE_DEBUG").is_ok() && (row_index + 1) % 5000 == 0 {
                                    eprintln!("🔍 DEBUG table_create: Обработано строк: {}/{}, время: {:?}", 
                                        row_index + 1, rows.len(), process_start.elapsed());
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

                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG table_create: Все строки обработаны ({}), время обработки: {:?}", 
                            processed_rows.len(), process_start.elapsed());
                    }

                    // Use bulk add operation for better performance
                    // Skip invalid rows instead of failing completely
                    let add_start = std::time::Instant::now();
                    let (_added, skipped) = table.add_rows_skip_invalid(processed_rows);
                    if skipped > 0 {
                        eprintln!("Warning: Пропущено {} строк с неверным количеством колонок", skipped);
                    }

                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG table_create: Строки добавлены в таблицу, время добавления: {:?}, общее время: {:?}", 
                            add_start.elapsed(), start_time.elapsed());
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
                        Ok(materialized_table) => {
                            // Выводим таблицу
                            print_table(&materialized_table);
                            Ok(Value::table(materialized_table))
                        }
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

        "merge_tables" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("merge_tables", 1, args.len(), line));
            }

            match &args[0] {
                Array(tables) => {
                    if tables.is_empty() {
                        return Ok(Value::Null);
                    }

                    // Get headers from first table
                    let first_table = match tables.first() {
                        Some(Value::Table(table_rc)) => {
                            let borrowed = table_rc.borrow();
                            borrowed.column_names.clone()
                        }
                        _ => return Err(DataCodeError::type_error("Array of Tables", "other", line)),
                    };

                    // Collect all rows from all tables
                    let mut all_rows = Vec::new();
                    let mut skipped_rows = 0;
                    let mut skipped_tables = 0;

                    for (table_idx, table_value) in tables.iter().enumerate() {
                        match table_value {
                            Value::Table(table_rc) => {
                                let table_borrowed = table_rc.borrow();
                                
                                // Skip tables with different headers instead of failing
                                if table_borrowed.column_names != first_table {
                                    skipped_tables += 1;
                                    skipped_rows += table_borrowed.rows.len();
                                    if std::env::var("DATACODE_DEBUG").is_ok() {
                                        eprintln!("🔍 DEBUG merge_tables: Пропущена таблица {} с несовпадающими заголовками", table_idx + 1);
                                    }
                                    continue;
                                }

                                // Add all rows from this table
                                for row in &table_borrowed.rows {
                                    if row.len() == first_table.len() {
                                        all_rows.push(row.clone());
                                    } else {
                                        skipped_rows += 1;
                                    }
                                }
                            }
                            _ => return Err(DataCodeError::type_error("Array of Tables", "other", line)),
                        }
                    }

                    if skipped_tables > 0 {
                        eprintln!("Warning: Пропущено {} таблиц с несовпадающими заголовками", skipped_tables);
                    }
                    if skipped_rows > 0 {
                        eprintln!("Warning: Пропущено {} строк с неверным количеством колонок при объединении", skipped_rows);
                    }

                    // Create merged table
                    let mut merged_table = TableStruct::new(first_table);
                    let (_added, skipped) = merged_table.add_rows_skip_invalid(all_rows);
                    
                    if skipped > 0 {
                        eprintln!("Warning: Дополнительно пропущено {} строк при создании объединенной таблицы", skipped);
                    }

                    Ok(Value::table(merged_table))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
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

        "relate" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("relate", 2, args.len(), line));
            }
            relate_columns(&args[0], &args[1], line)
        }

        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to table functions
pub fn is_table_function(name: &str) -> bool {
    matches!(name,
        "table" | "table_create" | "show_table" | "table_info" | "table_head" | "table_tail" |
        "table_headers" | "table_select" | "table_sort" | "merge_tables" | "relate"
    )
}

// Helper functions
fn print_table(table: &TableStruct) {
    if table.rows.is_empty() {
        output_line("Empty table");
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
    
    // Формируем строки таблицы
    let mut lines = Vec::new();
    
    // Верхняя граница
    let mut line = String::from("┌");
    for (i, &width) in max_widths.iter().enumerate() {
        line.push_str(&"─".repeat(width + 2));
        if i < max_widths.len() - 1 {
            line.push('┬');
        }
    }
    line.push('┐');
    lines.push(line);
    
    // Заголовки
    let mut line = String::from("│");
    for (i, header) in table.column_names.iter().enumerate() {
        line.push_str(&format!(" {:width$} ", header, width = max_widths[i]));
        if i < table.column_names.len() - 1 {
            line.push('│');
        }
    }
    line.push('│');
    lines.push(line);
    
    // Разделитель
    let mut line = String::from("├");
    for (i, &width) in max_widths.iter().enumerate() {
        line.push_str(&"─".repeat(width + 2));
        if i < max_widths.len() - 1 {
            line.push('┼');
        }
    }
    line.push('┤');
    lines.push(line);
    
    // Строки данных
    for row in &table.rows {
        let mut line = String::from("│");
        for (i, value) in row.iter().enumerate() {
            let formatted = format_value_for_table(value);
            line.push_str(&format!(" {:width$} ", formatted, width = max_widths[i]));
            if i < row.len() - 1 {
                line.push('│');
            }
        }
        line.push('│');
        lines.push(line);
    }
    
    // Нижняя граница
    let mut line = String::from("└");
    for (i, &width) in max_widths.iter().enumerate() {
        line.push_str(&"─".repeat(width + 2));
        if i < max_widths.len() - 1 {
            line.push('┴');
        }
    }
    line.push('┘');
    lines.push(line);
    
    // Выводим все строки
    for line in lines {
        output_line(&line);
    }
}

fn print_table_info(table: &TableStruct) {
    output_line(&format!("📊 Информация о таблице:"));
    output_line(&format!("   Строк: {}", table.rows.len()));
    output_line(&format!("   Колонок: {}", table.column_names.len()));
    output_line("");
    output_line("📋 Колонки:");
    
    for (i, column_name) in table.column_names.iter().enumerate() {
        let data_type = infer_column_type(table, i);
        let non_null_count = count_non_null_values(table, i);
        output_line(&format!("   • {} ({:?}) - {} значений", column_name, data_type, non_null_count));
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
        Value::TableColumn(_table, column) => {
            format!("Column({})", column)
        },
        Value::TableIndexer(table) => {
            let table_borrowed = table.borrow();
            format!("TableIndexer({}x{})", table_borrowed.rows.len(), table_borrowed.columns.len())
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
/// Создать связь между двумя колонками таблиц
fn relate_columns(col1: &Value, col2: &Value, line: usize) -> Result<Value> {
    use Value::*;
    
    // Извлекаем информацию о колонках
    let (table1, column1) = match col1 {
        TableColumn(table, col) => (table.clone(), col.clone()),
        Array(_) => {
            // Если передан массив, пытаемся найти таблицу в контексте
            // Для простоты, если передан массив, это ошибка
            return Err(DataCodeError::runtime_error(
                "relate() expects table columns (use table[\"column_name\"]), not arrays",
                line
            ));
        }
        _ => {
            let found_type: &str = match col1 {
                Value::Number(_) => "Number",
                Value::String(_) => "String",
                Value::Bool(_) => "Bool",
                Value::Array(_) => "Array",
                Value::Object(_) => "Object",
                Value::Table(_) => "Table",
                Value::Currency(_) => "Currency",
                Value::Null => "Null",
                Value::Path(_) => "Path",
                Value::PathPattern(_) => "PathPattern",
                _ => "Unknown",
            };
            return Err(DataCodeError::type_error(
                "TableColumn",
                found_type,
                line
            ));
        }
    };
    
    let (table2, column2) = match col2 {
        TableColumn(table, col) => (table.clone(), col.clone()),
        Array(_) => {
            return Err(DataCodeError::runtime_error(
                "relate() expects table columns (use table[\"column_name\"]), not arrays",
                line
            ));
        }
        _ => {
            let found_type: &str = match col2 {
                Value::Number(_) => "Number",
                Value::String(_) => "String",
                Value::Bool(_) => "Bool",
                Value::Array(_) => "Array",
                Value::Object(_) => "Object",
                Value::Table(_) => "Table",
                Value::Currency(_) => "Currency",
                Value::Null => "Null",
                Value::Path(_) => "Path",
                Value::PathPattern(_) => "PathPattern",
                _ => "Unknown",
            };
            return Err(DataCodeError::type_error(
                "TableColumn",
                found_type,
                line
            ));
        }
    };
    
    // Получаем информацию о типах колонок
    let table1_borrowed = table1.borrow();
    let table2_borrowed = table2.borrow();
    
    // Находим колонки в таблицах
    let col1_info = table1_borrowed.get_column_by_name(&column1)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Column '{}' not found in first table", column1),
            line
        ))?;
    
    let col2_info = table2_borrowed.get_column_by_name(&column2)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Column '{}' not found in second table", column2),
            line
        ))?;
    
    // Проверяем совместимость типов
    let type1 = &col1_info.inferred_type;
    let type2 = &col2_info.inferred_type;
    
    if !type1.is_compatible_with(type2) {
        return Err(DataCodeError::runtime_error(
            &format!(
                "Cannot relate columns with incompatible types: {} ({}) and {} ({})",
                column1, type1.to_string(),
                column2, type2.to_string()
            ),
            line
        ));
    }
    
    // Определяем тип связи (используем более общий тип)
    let relation_type = if type1.is_numeric() && type2.is_numeric() {
        // Если оба числовые, используем Float если хотя бы один Float
        if matches!(type1, DataType::Float) || matches!(type2, DataType::Float) {
            DataType::Float
        } else {
            DataType::Integer
        }
    } else {
        // Используем первый тип, если он не Null
        if matches!(type1, DataType::Null) {
            type2.clone()
        } else {
            type1.clone()
        }
    };
    
    // Создаем связь
    let relation = Relation::new(
        table1.clone(),
        column1.clone(),
        table2.clone(),
        column2.clone(),
        relation_type.clone(),
    );
    
    // Отладочный вывод до добавления
    if std::env::var("DATACODE_DEBUG").is_ok() {
        eprintln!("🔍 DEBUG relate_columns: Creating relation {}[{}] <-> {}[{}]",
            column1, column2, column1, column2);
        eprintln!("  Table1 Rc pointer: {:p}", table1.as_ptr());
        eprintln!("  Table2 Rc pointer: {:p}", table2.as_ptr());
    }
    
    // Добавляем связь в реестр (используем клон, чтобы сохранить relation для возврата)
    add_relation(relation.clone());
    
    // Отладочный вывод после добавления
    if std::env::var("DATACODE_DEBUG").is_ok() {
        let relations_count = crate::value::relations::get_all_relations().len();
        eprintln!("🔍 DEBUG relate_columns: Added relation, total relations in registry: {}", relations_count);
    }
    
    // Возвращаем объект связи
    let mut relation_obj = std::collections::HashMap::new();
    relation_obj.insert("table1".to_string(), Value::Table(table1.clone()));
    relation_obj.insert("column1".to_string(), Value::String(column1.clone()));
    relation_obj.insert("table2".to_string(), Value::Table(table2.clone()));
    relation_obj.insert("column2".to_string(), Value::String(column2.clone()));
    relation_obj.insert("type".to_string(), Value::String(relation_type.to_string().to_string()));
    
    Ok(Value::Object(relation_obj))
}

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

