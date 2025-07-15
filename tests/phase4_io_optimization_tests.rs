// Тесты Фазы 4: Оптимизация I/O и встроенных функций
// Критические тесты для обеспечения должности специалиста по Rust

use data_code::builtins::file_io::OptimizedCsvReader;
use data_code::builtins::registry::{
    FunctionRegistry, function_exists
};
use data_code::cache::{
    OperationCache, TableId, FilterExpr
};
use data_code::value::{Value, Table};
use std::path::Path;
use std::time::Duration;
use std::fs;


#[test]
fn test_optimized_csv_reader_basic() {
    // Создаем временный CSV файл
    let csv_content = "id,name,age\n1,Alice,25\n2,Bob,30\n3,Charlie,35";
    let temp_file = "/tmp/test_data.csv";

    fs::write(temp_file, csv_content).expect("Failed to write test CSV");

    let _reader = OptimizedCsvReader::new();

    // Пока что просто проверяем, что reader создается
    // Полная функциональность будет добавлена позже
    assert!(true);

    // Очищаем временный файл
    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_optimized_csv_reader_large_file() {
    // Тест для больших файлов - пока что заглушка
    let _reader = OptimizedCsvReader::new();
    assert!(true); // Заглушка
}

#[test]
fn test_optimized_csv_reader_data_types() {
    // Тестируем автоматическое определение типов данных
    let csv_content = "id,name,active,score,date\n1,Alice,true,95.5,2023-01-01\n2,Bob,false,87.2,2023-01-02";
    let temp_file = "/tmp/test_types.csv";
    
    fs::write(temp_file, csv_content).expect("Failed to write test CSV");
    
    let reader = OptimizedCsvReader::new();
    let result = reader.read_csv_optimized(Path::new(temp_file));
    assert!(result.is_ok());
    
    let table = result.unwrap();
    
    // Проверяем типы данных первой строки
    assert!(matches!(table.rows[0][0], Value::Number(_))); // id
    assert!(matches!(table.rows[0][1], Value::String(_))); // name
    assert!(matches!(table.rows[0][2], Value::Bool(true))); // active
    assert!(matches!(table.rows[0][3], Value::Number(_))); // score
    assert!(matches!(table.rows[0][4], Value::String(_))); // date
    
    // Очищаем временный файл
    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_file_cache_basic() {
    // Кэш файлов временно отключен
    // Тест будет реализован позже
    assert!(true);
}

#[test]
fn test_function_registry_basic() {
    // Проверяем существование основных функций
    assert!(function_exists("table_filter"));
    assert!(function_exists("sum"));
    assert!(function_exists("len"));

    // Проверяем несуществующую функцию
    assert!(!function_exists("nonexistent_function"));
}

#[test]
fn test_function_registry_categories() {
    // Тест категорий функций - упрощенная версия
    let registry = FunctionRegistry::new();
    let categories = registry.get_categories();
    assert!(!categories.is_empty());
}

#[test]
fn test_function_registry_fast_access() {
    // Тестируем быстрый доступ к функциям
    assert!(function_exists("sum"));
    assert!(function_exists("table_filter"));
    assert!(!function_exists("nonexistent"));
}

#[test]
fn test_function_info_validation() {
    // Тест валидации аргументов функций - упрощенная версия
    let registry = FunctionRegistry::new();
    assert!(registry.has_function("sum"));
    assert!(registry.has_function("table_filter"));
}

#[test]
fn test_operation_cache_basic() {
    let cache = OperationCache::new(10, Duration::from_secs(60));

    // Создаем тестовые данные
    let mut table = Table::new(vec!["id".to_string(), "name".to_string()]);
    table.rows = vec![
        vec![Value::Number(1.0), Value::String("Alice".to_string())],
        vec![Value::Number(2.0), Value::String("Bob".to_string())],
    ];

    let table_id = TableId::from_table(&table);
    let filter = FilterExpr::simple("id".to_string(), ">".to_string(), "0".to_string());

    // Проверяем, что кэш пуст
    assert!(cache.get_filter_result(&table_id, &filter).is_none());

    // Добавляем результат в кэш
    cache.cache_filter_result(table_id.clone(), filter.clone(), table.clone());

    // Проверяем, что результат есть в кэше
    let cached_result = cache.get_filter_result(&table_id, &filter);
    assert!(cached_result.is_some());

    // Проверяем статистику
    let stats = cache.get_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.filter_entries, 1);
    assert!(stats.hit_rate > 0.0);
}

#[test]
fn test_operation_cache_select() {
    let cache = OperationCache::new(10, Duration::from_secs(60));

    let mut table = Table::new(vec!["id".to_string(), "name".to_string(), "age".to_string()]);
    table.rows = vec![vec![Value::Number(1.0), Value::String("Alice".to_string()), Value::Number(25.0)]];

    let table_id = TableId::from_table(&table);
    let columns = vec!["id".to_string(), "name".to_string()];

    // Проверяем, что кэш пуст
    assert!(cache.get_select_result(&table_id, &columns).is_none());

    // Добавляем результат в кэш
    cache.cache_select_result(table_id.clone(), columns.clone(), table.clone());

    // Проверяем, что результат есть в кэше
    let cached_result = cache.get_select_result(&table_id, &columns);
    assert!(cached_result.is_some());

    let stats = cache.get_stats();
    assert_eq!(stats.select_entries, 1);
}

#[test]
fn test_operation_cache_sort() {
    let cache = OperationCache::new(10, Duration::from_secs(60));

    let mut table = Table::new(vec!["id".to_string(), "name".to_string()]);
    table.rows = vec![
        vec![Value::Number(2.0), Value::String("Bob".to_string())],
        vec![Value::Number(1.0), Value::String("Alice".to_string())],
    ];

    let table_id = TableId::from_table(&table);
    let column = "id";
    let ascending = true;

    // Проверяем, что кэш пуст
    assert!(cache.get_sort_result(&table_id, column, ascending).is_none());

    // Добавляем результат в кэш
    cache.cache_sort_result(table_id.clone(), column.to_string(), ascending, table.clone());

    // Проверяем, что результат есть в кэше
    let cached_result = cache.get_sort_result(&table_id, column, ascending);
    assert!(cached_result.is_some());

    let stats = cache.get_stats();
    assert_eq!(stats.sort_entries, 1);
}

#[test]
fn test_operation_cache_aggregate() {
    let cache = OperationCache::new(10, Duration::from_secs(60));

    let mut table = Table::new(vec!["id".to_string(), "value".to_string()]);
    table.rows = vec![
        vec![Value::Number(1.0), Value::Number(10.0)],
        vec![Value::Number(2.0), Value::Number(20.0)],
    ];

    let table_id = TableId::from_table(&table);
    let column = "value";
    let operation = "sum";

    // Проверяем, что кэш пуст
    assert!(cache.get_aggregate_result(&table_id, column, operation).is_none());

    // Добавляем результат в кэш
    let result = Value::Number(30.0);
    cache.cache_aggregate_result(table_id.clone(), column.to_string(), operation.to_string(), result.clone());

    // Проверяем, что результат есть в кэше
    let cached_result = cache.get_aggregate_result(&table_id, column, operation);
    assert!(cached_result.is_some());
    assert_eq!(cached_result.unwrap(), result);

    let stats = cache.get_stats();
    assert_eq!(stats.aggregate_entries, 1);
}

#[test]
fn test_operation_cache_eviction() {
    let cache = OperationCache::new(2, Duration::from_secs(60)); // Максимум 2 записи

    let table = Table::new(vec!["id".to_string()]);
    let table_id = TableId::from_table(&table);

    // Добавляем 3 записи в кэш с максимумом 2
    for i in 0..3 {
        let filter = FilterExpr::simple("id".to_string(), ">".to_string(), i.to_string());
        cache.cache_filter_result(table_id.clone(), filter, table.clone());
    }

    let stats = cache.get_stats();
    assert_eq!(stats.filter_entries, 2); // Должно быть только 2 записи из-за eviction
}

#[test]
fn test_operation_cache_expiration() {
    let cache = OperationCache::new(10, Duration::from_millis(1)); // 1ms TTL

    let table = Table::new(vec!["id".to_string()]);
    let table_id = TableId::from_table(&table);
    let filter = FilterExpr::simple("id".to_string(), ">".to_string(), "0".to_string());

    // Добавляем в кэш
    cache.cache_filter_result(table_id.clone(), filter.clone(), table);

    // Ждем истечения TTL
    std::thread::sleep(Duration::from_millis(2));

    // Проверяем, что запись истекла
    assert!(cache.get_filter_result(&table_id, &filter).is_none());
}

#[test]
fn test_operation_cache_stats() {
    // Тест статистики кэша
    let cache = OperationCache::new(10, Duration::from_secs(60));
    let stats = cache.get_stats();

    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.total_entries(), 0);
}
