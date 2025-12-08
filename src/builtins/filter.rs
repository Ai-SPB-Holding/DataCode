use crate::value::{Value, Table as TableStruct, LazyTable};
use crate::error::{DataCodeError, Result};

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Table filtering functions
pub fn call_filter_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "table_filter" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_filter", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), String(condition)) => {
                    filter_table_by_condition_optimized(table_rc.clone(), condition, line)
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
                    filter_table_where_optimized(table_rc.clone(), column, operator, value, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, String, Value", "other", line)),
            }
        }

        "table_query" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_query", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), String(query)) => {
                    let table_borrowed = table_rc.borrow();
                    filter_table_by_query(&*table_borrowed, query, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_distinct" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_distinct", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), String(column)) => {
                    let table_borrowed = table_rc.borrow();
                    get_distinct_values(&*table_borrowed, column, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_sample" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_sample", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), Number(n)) => {
                    let table_borrowed = table_rc.borrow();
                    sample_table_rows(&*table_borrowed, *n as usize, line)
                }
                _ => Err(DataCodeError::type_error("Table and Number", "other", line)),
            }
        }

        "table_between" => {
            if args.len() != 4 {
                return Err(DataCodeError::wrong_argument_count("table_between", 4, args.len(), line));
            }
            
            match (&args[0], &args[1], &args[2], &args[3]) {
                (Value::Table(table_rc), String(column), min_val, max_val) => {
                    let table_borrowed = table_rc.borrow();
                    filter_table_between(&*table_borrowed, column, min_val, max_val, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, Value, Value", "other", line)),
            }
        }

        "table_in" => {
            if args.len() != 3 {
                return Err(DataCodeError::wrong_argument_count("table_in", 3, args.len(), line));
            }
            
            match (&args[0], &args[1], &args[2]) {
                (Value::Table(table_rc), String(column), Array(values)) => {
                    let table_borrowed = table_rc.borrow();
                    filter_table_in(&*table_borrowed, column, values, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, Array", "other", line)),
            }
        }

        "table_is_null" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_is_null", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), String(column)) => {
                    filter_table_null_optimized(table_rc.clone(), column, true, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        "table_not_null" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_not_null", 2, args.len(), line));
            }
            
            match (&args[0], &args[1]) {
                (Value::Table(table_rc), String(column)) => {
                    filter_table_null_optimized(table_rc.clone(), column, false, line)
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

// Оптимизированная фильтрация таблицы по условию с использованием Rc<RefCell<T>>
fn filter_table_by_condition_optimized(table_rc: Rc<RefCell<TableStruct>>, condition: &str, line: usize) -> Result<Value> {
    // Создаем ленивую таблицу и добавляем операцию фильтрации
    let lazy_table = LazyTable::new(table_rc).filter(condition.to_string(), line);

    // Материализуем результат
    match lazy_table.materialize() {
        Ok(materialized_table) => Ok(Value::table(materialized_table)),
        Err(e) => Err(e),
    }
}

// Оригинальная фильтрация таблицы по условию с парсингом выражения (для совместимости)
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
    
    Ok(Value::table(filtered_table))
}

// Оптимизированная SQL-подобная фильтрация WHERE с использованием Rc<RefCell<T>>
fn filter_table_where_optimized(table_rc: Rc<RefCell<TableStruct>>, column: &str, operator: &str, value: &Value, line: usize) -> Result<Value> {
    // Создаем ленивую таблицу и добавляем операцию WHERE
    let lazy_table = LazyTable::new(table_rc).where_op(column.to_string(), operator.to_string(), value.clone(), line);

    // Материализуем результат
    match lazy_table.materialize() {
        Ok(materialized_table) => Ok(Value::table(materialized_table)),
        Err(e) => Err(e),
    }
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
        return Ok(Value::table(table.clone()));
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
    
    Ok(Value::table(sampled_table))
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
    
    Ok(Value::table(filtered_table))
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
    
    Ok(Value::table(filtered_table))
}



// Оптимизированная фильтрация по null/not null значениям с использованием Rc<RefCell<T>>
fn filter_table_null_optimized(table_rc: Rc<RefCell<TableStruct>>, column: &str, is_null: bool, line: usize) -> Result<Value> {
    let table_borrowed = table_rc.borrow();

    // Находим индекс колонки
    let col_index = table_borrowed.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;

    let mut filtered_table = TableStruct::new(table_borrowed.column_names.clone());

    for row in &table_borrowed.rows {
        if let Some(row_value) = row.get(col_index) {
            let is_value_null = matches!(row_value, Value::Null);
            if is_value_null == is_null {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }

    Ok(Value::table(filtered_table))
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


