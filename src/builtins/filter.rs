use crate::value::{Value, Table as TableStruct};
use crate::error::{DataCodeError, Result};
use std::collections::HashMap;

/// Table filtering functions
pub fn call_filter_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "table_filter" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_filter", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table), String(condition)) => {
                    filter_table_by_condition(table, condition, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_where" => {
            if args.len() != 4 {
                return Err(DataCodeError::wrong_argument_count("table_where", 4, args.len(), line));
            }
            
            match (&args[0], &args[1], &args[2], &args[3]) {
                (Value::Table(table), String(column), String(operator), value) => {
                    filter_table_where(table, column, operator, value, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, String, Value", "other", line)),
            }
        }

        "table_query" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_query", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table), String(query)) => {
                    filter_table_by_query(table, query, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_distinct" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_distinct", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table), String(column)) => {
                    get_distinct_values(table, column, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_sample" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_sample", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table), Number(n)) => {
                    sample_table_rows(table, *n as usize, line)
                }
                _ => Err(DataCodeError::type_error("Table and Number", "other", line)),
            }
        }

        "table_between" => {
            if args.len() != 4 {
                return Err(DataCodeError::wrong_argument_count("table_between", 4, args.len(), line));
            }
            
            match (&args[0], &args[1], &args[2], &args[3]) {
                (Value::Table(table), String(column), min_val, max_val) => {
                    filter_table_between(table, column, min_val, max_val, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, Value, Value", "other", line)),
            }
        }

        "table_in" => {
            if args.len() != 3 {
                return Err(DataCodeError::wrong_argument_count("table_in", 3, args.len(), line));
            }
            
            match (&args[0], &args[1], &args[2]) {
                (Value::Table(table), String(column), Array(values)) => {
                    filter_table_in(table, column, values, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, Array", "other", line)),
            }
        }

        "table_is_null" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_is_null", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table), String(column)) => {
                    filter_table_null(table, column, true, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_not_null" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_not_null", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table), String(column)) => {
                    filter_table_null(table, column, false, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to filter functions
pub fn is_filter_function(name: &str) -> bool {
    matches!(name, 
        "table_filter" | "table_where" | "table_query" | "table_distinct" | 
        "table_sample" | "table_between" | "table_in" | "table_is_null" | "table_not_null"
    )
}

// ========== IMPLEMENTATION FUNCTIONS ==========

// Фильтрация таблицы по условию с парсингом выражения
fn filter_table_by_condition(table: &TableStruct, condition: &str, line: usize) -> Result<Value> {
    let mut filtered_table = TableStruct::new(table.column_names.clone());
    
    for row in &table.rows {
        // Создаем контекст переменных для текущей строки
        let mut row_context = HashMap::new();
        for (i, col_name) in table.column_names.iter().enumerate() {
            if let Some(value) = row.get(i) {
                row_context.insert(col_name.clone(), value.clone());
            }
        }
        
        // Парсим выражение
        let mut parser = crate::parser::Parser::new(condition);
        let expr = match parser.parse_expression() {
            Ok(expr) => expr,
            Err(e) => return Err(DataCodeError::runtime_error(&format!("Ошибка парсинга условия: {}", e), line)),
        };
        
        // Создаем evaluator с контекстом строки
        let evaluator = crate::evaluator::Evaluator::new(&row_context, line);
        
        // Вычисляем условие
        match evaluator.evaluate(&expr) {
            Ok(Value::Bool(true)) => {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
            Ok(Value::Bool(false)) => {
                // Строка не прошла фильтр
            }
            Ok(_) => {
                return Err(DataCodeError::runtime_error("Условие фильтрации должно возвращать boolean", line));
            }
            Err(e) => {
                return Err(DataCodeError::runtime_error(&format!("Ошибка в условии фильтрации: {}", e), line));
            }
        }
    }
    
    Ok(Value::Table(filtered_table))
}

// SQL-подобная фильтрация WHERE
fn filter_table_where(table: &TableStruct, column: &str, operator: &str, value: &Value, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;
    
    let mut filtered_table = TableStruct::new(table.column_names.clone());
    
    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            let matches = match operator {
                "=" | "==" => values_equal(row_value, value),
                "!=" | "<>" => !values_equal(row_value, value),
                "<" => compare_values_for_filter(row_value, value) == std::cmp::Ordering::Less,
                ">" => compare_values_for_filter(row_value, value) == std::cmp::Ordering::Greater,
                "<=" => {
                    let cmp = compare_values_for_filter(row_value, value);
                    cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal
                }
                ">=" => {
                    let cmp = compare_values_for_filter(row_value, value);
                    cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal
                }
                "LIKE" => match (row_value, value) {
                    (Value::String(s1), Value::String(s2)) => {
                        // Простая реализация LIKE с поддержкой % и _
                        let pattern = s2.replace('%', ".*").replace('_', ".");
                        match regex::Regex::new(&pattern) {
                            Ok(re) => re.is_match(s1),
                            Err(_) => false,
                        }
                    }
                    _ => false,
                },
                "IN" => match value {
                    Value::Array(arr) => arr.iter().any(|v| values_equal(row_value, v)),
                    _ => false,
                },
                _ => return Err(DataCodeError::runtime_error(
                    &format!("Неподдерживаемый оператор: {}", operator),
                    line
                )),
            };
            
            if matches {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }
    
    Ok(Value::Table(filtered_table))
}

// Сложные запросы с парсингом выражений (алиас для filter_table_by_condition)
fn filter_table_by_query(table: &TableStruct, query: &str, line: usize) -> Result<Value> {
    filter_table_by_condition(table, query, line)
}

// Получение уникальных значений в колонке
fn get_distinct_values(table: &TableStruct, column: &str, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;
    
    let mut unique_values = std::collections::HashSet::new();
    let mut result_values = Vec::new();
    
    for row in &table.rows {
        if let Some(value) = row.get(col_index) {
            let value_str = format_value_for_table(value);
            if unique_values.insert(value_str) {
                result_values.push(value.clone());
            }
        }
    }
    
    Ok(Value::Array(result_values))
}

// Случайная выборка строк из таблицы
fn sample_table_rows(table: &TableStruct, n: usize, line: usize) -> Result<Value> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    
    if n >= table.rows.len() {
        // Если запрашиваем больше строк чем есть, возвращаем всю таблицу
        return Ok(Value::Table(table.clone()));
    }
    
    let mut rng = thread_rng();
    let sampled_indices: Vec<usize> = (0..table.rows.len()).collect::<Vec<_>>()
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect();
    
    let mut sampled_table = TableStruct::new(table.column_names.clone());
    
    for &index in &sampled_indices {
        if let Some(row) = table.rows.get(index) {
            if let Err(e) = sampled_table.add_row(row.clone()) {
                return Err(DataCodeError::runtime_error(&e, line));
            }
        }
    }
    
    Ok(Value::Table(sampled_table))
}

// Фильтрация по диапазону значений (BETWEEN)
fn filter_table_between(table: &TableStruct, column: &str, min_val: &Value, max_val: &Value, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;
    
    let mut filtered_table = TableStruct::new(table.column_names.clone());
    
    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            let cmp_min = compare_values_for_filter(row_value, min_val);
            let cmp_max = compare_values_for_filter(row_value, max_val);
            
            // Значение должно быть >= min_val и <= max_val
            if (cmp_min == std::cmp::Ordering::Greater || cmp_min == std::cmp::Ordering::Equal) &&
               (cmp_max == std::cmp::Ordering::Less || cmp_max == std::cmp::Ordering::Equal) {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }
    
    Ok(Value::Table(filtered_table))
}

// Фильтрация по списку значений (IN)
fn filter_table_in(table: &TableStruct, column: &str, values: &[Value], line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;
    
    let mut filtered_table = TableStruct::new(table.column_names.clone());
    
    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            if values.iter().any(|v| values_equal(row_value, v)) {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }
    
    Ok(Value::Table(filtered_table))
}

// Фильтрация по null/not null значениям
fn filter_table_null(table: &TableStruct, column: &str, is_null: bool, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;
    
    let mut filtered_table = TableStruct::new(table.column_names.clone());
    
    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            let is_value_null = matches!(row_value, Value::Null);
            if is_value_null == is_null {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }
    
    Ok(Value::Table(filtered_table))
}

// ========== HELPER FUNCTIONS ==========

// Функция для сравнения значений при фильтрации
fn compare_values_for_filter(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    use Value::*;

    match (a, b) {
        (Number(a), Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (String(a), String(b)) => a.cmp(b),
        (Currency(a), Currency(b)) => a.cmp(b),
        (Bool(a), Bool(b)) => a.cmp(b),
        (Null, Null) => Ordering::Equal,
        (Null, _) => Ordering::Less,
        (_, Null) => Ordering::Greater,
        // Для разных типов сравниваем их строковые представления
        _ => format_value_for_table(a).cmp(&format_value_for_table(b)),
    }
}

// Функция для проверки равенства значений
fn values_equal(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
        (String(a), String(b)) => a == b,
        (Currency(a), Currency(b)) => a == b,
        (Bool(a), Bool(b)) => a == b,
        (Null, Null) => true,
        (Array(a), Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        _ => false,
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
