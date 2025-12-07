//! Модуль экспорта таблиц DataCode в SQLite базу данных
//! 
//! Этот модуль предоставляет функциональность для автоматического экспорта
//! всех таблиц из обработанных данных DataCode в SQLite базу данных с поддержкой
//! зависимостей между таблицами.

use crate::interpreter::Interpreter;
use crate::value::{Value, Table, DataType, ValueOperations};
use crate::error::{DataCodeError, Result};
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use chrono::Utc;

/// Экспортировать все таблицы из интерпретатора в SQLite базу данных
/// 
/// # Аргументы
/// * `interpreter` - интерпретатор с глобальными переменными
/// * `output_path` - путь к выходному файлу SQLite базы данных
/// 
/// # Возвращает
/// `Result<()>` - успех или ошибка экспорта
/// Экспортировать все глобальные переменные из интерпретатора в SQLite базу данных
/// 
/// Экспортируются только глобальные переменные с их финальными значениями.
/// Локальные переменные (local) не экспортируются.
/// 
/// # Аргументы
/// * `interpreter` - интерпретатор с глобальными переменными (финальные значения после выполнения)
/// * `output_path` - путь к выходному файлу SQLite базы данных
/// 
/// # Возвращает
/// `Result<()>` - успех или ошибка экспорта
pub fn export_tables_to_sqlite(interpreter: &Interpreter, output_path: &str) -> Result<()> {
    // Получаем все глобальные переменные (только глобальные, не локальные)
    let global_vars = interpreter.get_all_variables();
    
    if global_vars.is_empty() {
        return Err(DataCodeError::runtime_error(
            "No global variables found to export",
            0
        ));
    }

    // Собираем все таблицы из глобальных переменных
    let tables = collect_tables_from_interpreter(interpreter)?;

    // Создаем SQLite базу данных
    let mut conn = Connection::open(output_path)
        .map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to create SQLite database: {}", e),
            0
        ))?;

    // Включаем внешние ключи
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to enable foreign keys: {}", e),
            0
        ))?;

    // Создаем таблицу метаданных о глобальных переменных
    create_metadata_table(&mut conn)?;

    // Экспортируем каждую таблицу (только глобальные таблицы)
    let mut table_names = Vec::new();
    for (var_name, table) in &tables {
        let sanitized_name = sanitize_table_name(var_name);
        export_table(&mut conn, &sanitized_name, table)?;
        table_names.push((var_name.clone(), sanitized_name));
    }

    // Записываем метаданные о ВСЕХ глобальных переменных (включая не-таблицы)
    // Это гарантирует, что экспортируются только глобальные переменные с их финальными значениями
    write_variables_metadata(&mut conn, interpreter, &table_names)?;

    // Определяем и создаем зависимости между таблицами (только для глобальных таблиц)
    if !tables.is_empty() {
        create_foreign_keys(&mut conn, &tables, &table_names)?;
        // Создаем индексы для производительности
        create_indexes(&mut conn, &tables, &table_names)?;
    }

    Ok(())
}

/// Собрать все таблицы из глобальных переменных интерпретатора
/// 
/// ВАЖНО: Использует get_all_variables(), который возвращает ТОЛЬКО глобальные переменные.
/// Локальные переменные (local) не включаются в экспорт.
fn collect_tables_from_interpreter(interpreter: &Interpreter) -> Result<HashMap<String, Rc<RefCell<Table>>>> {
    let mut tables = HashMap::new();
    // get_all_variables() возвращает только глобальные переменные (не локальные)
    let variables = interpreter.get_all_variables();

    for (name, value) in variables {
        if let Value::Table(table) = value {
            tables.insert(name.clone(), table.clone());
        }
    }

    Ok(tables)
}

/// Создать таблицу метаданных о глобальных переменных
fn create_metadata_table(conn: &mut Connection) -> Result<()> {
    // Сначала создаем таблицу без поля value (для обратной совместимости)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _datacode_variables (
            variable_name TEXT PRIMARY KEY,
            variable_type TEXT NOT NULL,
            table_name TEXT,
            row_count INTEGER,
            column_count INTEGER,
            created_at TEXT,
            description TEXT
        )",
        []
    ).map_err(|e| DataCodeError::runtime_error(
        &format!("Failed to create metadata table: {}", e),
        0
    ))?;

    // Добавляем поле value, если его еще нет (для обратной совместимости с существующими БД)
    conn.execute(
        "ALTER TABLE _datacode_variables ADD COLUMN value TEXT",
        []
    ).ok(); // Игнорируем ошибку, если колонка уже существует

    Ok(())
}

/// Записать метаданные о всех глобальных переменных
/// 
/// ВАЖНО: Использует get_all_variables(), который возвращает ТОЛЬКО глобальные переменные
/// с их финальными значениями после выполнения скрипта.
/// Локальные переменные (local) не экспортируются.
fn write_variables_metadata(
    conn: &mut Connection,
    interpreter: &Interpreter,
    table_names: &[(String, String)]
) -> Result<()> {
    // get_all_variables() возвращает только глобальные переменные (не локальные)
    // Значения являются финальными, так как экспорт вызывается после выполнения скрипта
    let variables = interpreter.get_all_variables();
    let created_at = Utc::now().to_rfc3339();
    let table_map: HashMap<&String, &String> = table_names.iter()
        .map(|(var, table)| (var, table))
        .collect();

    for (var_name, value) in variables {
        let (variable_type, table_name, row_count, column_count) = match value {
            Value::Table(table) => {
                let table_ref = table.borrow();
                let sqlite_table_name = table_map.get(&var_name).map(|s| s.as_str());
                (
                    "Table",
                    sqlite_table_name,
                    Some(table_ref.row_count() as i64),
                    Some(table_ref.column_count() as i64)
                )
            }
            Value::Array(_) => ("Array", None, None, None),
            Value::Object(_) => ("Object", None, None, None),
            Value::Number(_) => ("Number", None, None, None),
            Value::String(_) => ("String", None, None, None),
            Value::Bool(_) => ("Bool", None, None, None),
            Value::Currency(_) => ("Currency", None, None, None),
            Value::Null => ("Null", None, None, None),
            Value::Path(_) => ("Path", None, None, None),
            Value::PathPattern(_) => ("PathPattern", None, None, None),
        };

        // Преобразуем значение в строку для сохранения
        // Для больших значений (таблицы) ограничиваем длину
        let value_str = value.to_display_string();
        let value_to_store = if value_str.len() > 10000 {
            // Для очень больших значений сохраняем только префикс
            format!("{}... (truncated, len: {})", 
                &value_str[..10000], 
                value_str.len())
        } else {
            value_str
        };

        conn.execute(
            "INSERT OR REPLACE INTO _datacode_variables 
             (variable_name, variable_type, table_name, row_count, column_count, created_at, description, value)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                var_name,
                variable_type,
                table_name,
                row_count,
                column_count,
                created_at,
                None::<String>,
                value_to_store
            ]
        ).map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to insert metadata for variable '{}': {}", var_name, e),
            0
        ))?;
    }

    Ok(())
}

/// Экспортировать одну таблицу в SQLite
fn export_table(conn: &mut Connection, table_name: &str, table: &Rc<RefCell<Table>>) -> Result<()> {
    let table_ref = table.borrow();
    
    if table_ref.column_names.is_empty() {
        return Err(DataCodeError::runtime_error(
            &format!("Table '{}' has no columns", table_name),
            0
        ));
    }

    // Создаем SQL для создания таблицы
    let create_sql = build_create_table_sql(table_name, &table_ref)?;
    
    // Удаляем таблицу если она существует
    conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), [])
        .map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to drop table '{}': {}", table_name, e),
            0
        ))?;

    // Создаем таблицу
    conn.execute(&create_sql, [])
        .map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to create table '{}': {}", table_name, e),
            0
        ))?;

    // Вставляем данные batch-ом
    insert_table_data(conn, table_name, &table_ref)?;

    Ok(())
}

/// Построить SQL для создания таблицы
fn build_create_table_sql(table_name: &str, table: &Table) -> Result<String> {
    let mut columns = Vec::new();
    
    for (i, column_name) in table.column_names.iter().enumerate() {
        let sanitized_col_name = sanitize_column_name(column_name);
        let sql_type = datacode_type_to_sqlite(&table.columns[i].inferred_type);
        columns.push(format!("{} {}", sanitized_col_name, sql_type));
    }

    let sql = format!(
        "CREATE TABLE {} ({})",
        table_name,
        columns.join(", ")
    );

    Ok(sql)
}

/// Преобразовать тип DataCode в тип SQLite
fn datacode_type_to_sqlite(data_type: &DataType) -> &'static str {
    match data_type {
        DataType::Integer => "INTEGER",
        DataType::Float => "REAL",
        DataType::String => "TEXT",
        DataType::Bool => "INTEGER",
        DataType::Date => "TEXT",
        DataType::Currency => "REAL",
        DataType::Null => "TEXT",
        DataType::Mixed => "TEXT",
    }
}

/// Вставить данные таблицы в SQLite
fn insert_table_data(conn: &mut Connection, table_name: &str, table: &Table) -> Result<()> {
    if table.rows.is_empty() {
        return Ok(());
    }

    // Строим SQL для вставки
    let placeholders: Vec<String> = (0..table.column_names.len())
        .map(|i| format!("?{}", i + 1))
        .collect();
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        table.column_names.iter()
            .map(|n| sanitize_column_name(n))
            .collect::<Vec<_>>()
            .join(", "),
        placeholders.join(", ")
    );

    // Начинаем транзакцию для batch вставки
    let tx = conn.transaction()
        .map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to start transaction: {}", e),
            0
        ))?;

    // Вставляем данные
    let mut stmt = tx.prepare(&insert_sql)
        .map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to prepare insert statement: {}", e),
            0
        ))?;

    for row in &table.rows {
        // Конвертируем значения в параметры для rusqlite
        // Создаем вектор параметров, которые реализуют ToSql
        let params: Vec<Box<dyn rusqlite::ToSql>> = row.iter()
            .map(|v| value_to_sqlite_box(v))
            .collect();
        
        // Конвертируем в слайс ссылок
        let sql_params: Vec<&dyn rusqlite::ToSql> = params.iter()
            .map(|p| p.as_ref())
            .collect();
        
        stmt.execute(&sql_params[..])
            .map_err(|e| DataCodeError::runtime_error(
                &format!("Failed to insert row: {}", e),
                0
            ))?;
    }

    // Освобождаем statement перед commit
    drop(stmt);
    
    tx.commit()
        .map_err(|e| DataCodeError::runtime_error(
            &format!("Failed to commit transaction: {}", e),
            0
        ))?;

    Ok(())
}

/// Конвертировать Value в Box<dyn ToSql> для параметров SQLite
fn value_to_sqlite_box(value: &Value) -> Box<dyn rusqlite::ToSql> {
    match value {
        Value::Number(n) => {
            // Проверяем, является ли число целым
            if n.fract() == 0.0 {
                Box::new(*n as i64)
            } else {
                Box::new(*n)
            }
        }
        Value::String(s) => Box::new(s.clone()),
        Value::Bool(b) => Box::new(if *b { 1i64 } else { 0i64 }),
        Value::Null => Box::new(None::<String>),
        Value::Currency(c) => {
            // Пытаемся извлечь число из строки валюты
            let num_str = c.trim_start_matches(|ch: char| !ch.is_ascii_digit() && ch != '-' && ch != '.');
            if let Ok(num) = num_str.parse::<f64>() {
                Box::new(num)
            } else {
                Box::new(c.clone())
            }
        }
        _ => Box::new(format!("{:?}", value)),
    }
}

/// Санитизировать имя таблицы для SQLite
fn sanitize_table_name(name: &str) -> String {
    // Заменяем недопустимые символы на подчеркивания
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

/// Санитизировать имя колонки для SQLite
fn sanitize_column_name(name: &str) -> String {
    // Заменяем недопустимые символы на подчеркивания
    let sanitized: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    
    // Если имя начинается с цифры, добавляем префикс
    if sanitized.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("col_{}", sanitized)
    } else {
        sanitized
    }
}

/// Определить и создать внешние ключи между таблицами
fn create_foreign_keys(
    conn: &mut Connection,
    tables: &HashMap<String, Rc<RefCell<Table>>>,
    table_names: &[(String, String)]
) -> Result<()> {
    // Находим первичные ключи для каждой таблицы
    let mut primary_keys: HashMap<String, String> = HashMap::new();
    
    for (var_name, table) in tables {
        if let Some(pk_col) = find_primary_key(table) {
            let sanitized_name = table_names.iter()
                .find(|(v, _)| v == var_name)
                .map(|(_, t)| t.clone())
                .unwrap_or_else(|| sanitize_table_name(var_name));
            primary_keys.insert(sanitized_name, pk_col);
        }
    }

    // Ищем внешние ключи
    for (var_name, table) in tables {
        let table_ref = table.borrow();
        let sanitized_table_name = table_names.iter()
            .find(|(v, _)| v == var_name)
            .map(|(_, t)| t.clone())
            .unwrap_or_else(|| sanitize_table_name(var_name));

        // Ищем колонки с ID-подобными именами
        for col_name in &table_ref.column_names {
            if is_id_column(col_name) {
                // Ищем таблицу с соответствующим первичным ключом
                for (ref_table_name, ref_pk_col) in &primary_keys {
                    if ref_table_name != &sanitized_table_name {
                        // Проверяем совместимость типов и значений
                        if let Some(fk_col) = table_ref.get_column_by_name(col_name) {
                            if fk_col.inferred_type == DataType::Integer {
                                // Создаем внешний ключ
                                create_foreign_key_constraint(
                                    conn,
                                    &sanitized_table_name,
                                    col_name,
                                    ref_table_name,
                                    ref_pk_col
                                )?;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Найти первичный ключ в таблице
fn find_primary_key(table: &Rc<RefCell<Table>>) -> Option<String> {
    let table_ref = table.borrow();
    
    // Ищем колонку с именем "id" или "*_id" с типом Integer
    for col in &table_ref.columns {
        if col.name == "id" && col.inferred_type == DataType::Integer {
            return Some(col.name.clone());
        }
    }

    // Ищем колонку с префиксом "pk_" или "key_"
    for col in &table_ref.columns {
        if col.name.starts_with("pk_") || col.name.starts_with("key_") {
            if col.inferred_type == DataType::Integer {
                return Some(col.name.clone());
            }
        }
    }

    // Ищем колонку, где все значения уникальны (эвристика)
    for col in &table_ref.columns {
        if col.inferred_type == DataType::Integer {
            if let Some(values) = table_ref.get_column_values(&col.name) {
                let unique_count: std::collections::HashSet<String> = values.iter()
                    .map(|v| format!("{:?}", v))
                    .collect();
                if unique_count.len() == values.len() && values.len() > 0 {
                    return Some(col.name.clone());
                }
            }
        }
    }

    None
}

/// Проверить, является ли колонка ID-колонкой
fn is_id_column(name: &str) -> bool {
    name == "id" || 
    name.ends_with("_id") || 
    name.ends_with("Id") || 
    name.ends_with("ID")
}

/// Создать ограничение внешнего ключа
fn create_foreign_key_constraint(
    conn: &mut Connection,
    table_name: &str,
    column_name: &str,
    _ref_table_name: &str,
    _ref_column_name: &str
) -> Result<()> {
    // SQLite требует пересоздания таблицы для добавления внешнего ключа
    // Это сложная операция, поэтому пока просто логируем
    // В будущем можно реализовать полное пересоздание таблицы
    
    // Для упрощения, создаем индекс на внешнем ключе
    let index_name = format!("idx_{}_{}", table_name, column_name);
    let sanitized_col = sanitize_column_name(column_name);
    
    conn.execute(
        &format!(
            "CREATE INDEX IF NOT EXISTS {} ON {} ({})",
            index_name, table_name, sanitized_col
        ),
        []
    ).map_err(|e| DataCodeError::runtime_error(
        &format!("Failed to create index for foreign key: {}", e),
        0
    ))?;

    Ok(())
}

/// Создать индексы для производительности
fn create_indexes(
    conn: &mut Connection,
    tables: &HashMap<String, Rc<RefCell<Table>>>,
    table_names: &[(String, String)]
) -> Result<()> {
    for (var_name, table) in tables {
        let table_ref = table.borrow();
        let sanitized_table_name = table_names.iter()
            .find(|(v, _)| v == var_name)
            .map(|(_, t)| t.clone())
            .unwrap_or_else(|| sanitize_table_name(var_name));

        // Создаем индексы для ID-колонок
        for col_name in &table_ref.column_names {
            if is_id_column(col_name) {
                let index_name = format!("idx_{}_{}", sanitized_table_name, col_name);
                let sanitized_col = sanitize_column_name(col_name);
                
                conn.execute(
                    &format!(
                        "CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                        index_name, sanitized_table_name, sanitized_col
                    ),
                    []
                ).map_err(|e| DataCodeError::runtime_error(
                    &format!("Failed to create index: {}", e),
                    0
                ))?;
            }
        }
    }

    Ok(())
}

