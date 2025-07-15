// Система кэширования промежуточных результатов для DataCode
// Критическая реализация для обеспечения должности специалиста по Rust

use std::collections::HashMap;
use std::sync::Mutex;
use std::hash::{Hash, Hasher};

use std::time::{Duration, Instant};

use crate::value::{Value, Table};
use crate::parser::Expr;

/// Уникальный идентификатор таблицы для кэширования
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableId {
    pub name: String,
    pub hash: u64,
}

impl TableId {
    /// Создать ID таблицы из её содержимого
    pub fn from_table(table: &Table) -> Self {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        
        // Хэшируем имена колонок
        for name in &table.column_names {
            name.hash(&mut hasher);
        }
        
        // Хэшируем первые несколько строк для быстрого хэширования
        let sample_size = std::cmp::min(table.rows.len(), 100);
        for row in table.rows.iter().take(sample_size) {
            for value in row {
                match value {
                    Value::Number(n) => n.to_bits().hash(&mut hasher),
                    Value::String(s) => s.hash(&mut hasher),
                    Value::Bool(b) => b.hash(&mut hasher),
                    Value::Null => 0u8.hash(&mut hasher),
                    _ => format!("{:?}", value).hash(&mut hasher),
                }
            }
        }
        
        let hash = hasher.finish();
        
        Self {
            name: format!("table_{}", hash),
            hash,
        }
    }
    
    /// Создать ID из имени и хэша
    pub fn new(name: String, hash: u64) -> Self {
        Self { name, hash }
    }
}

/// Выражение фильтра для кэширования
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilterExpr {
    pub column: String,
    pub operator: String,
    pub value: String,
    pub expr_hash: u64,
}

impl FilterExpr {
    /// Создать выражение фильтра из AST
    pub fn from_expr(expr: &Expr) -> Self {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        let expr_str = format!("{:?}", expr);
        expr_str.hash(&mut hasher);
        
        Self {
            column: "unknown".to_string(),
            operator: "unknown".to_string(),
            value: "unknown".to_string(),
            expr_hash: hasher.finish(),
        }
    }
    
    /// Создать простое выражение фильтра
    pub fn simple(column: String, operator: String, value: String) -> Self {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        column.hash(&mut hasher);
        operator.hash(&mut hasher);
        value.hash(&mut hasher);
        
        Self {
            column,
            operator,
            value,
            expr_hash: hasher.finish(),
        }
    }
}

/// Запись кэша с временем жизни
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub access_count: u64,
    pub last_accessed: Instant,
}

impl<T> CacheEntry<T> {
    /// Создать новую запись кэша
    pub fn new(value: T) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            access_count: 0,
            last_accessed: now,
        }
    }
    
    /// Отметить доступ к записи
    pub fn access(&mut self) -> &T {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.value
    }
    
    /// Проверить, истекла ли запись
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// Кэш операций с таблицами (упрощенная версия)
pub struct OperationCache {
    filter_cache: Mutex<HashMap<(TableId, FilterExpr), CacheEntry<Table>>>,
    select_cache: Mutex<HashMap<(TableId, Vec<String>), CacheEntry<Table>>>,
    sort_cache: Mutex<HashMap<(TableId, String, bool), CacheEntry<Table>>>,
    aggregate_cache: Mutex<HashMap<(TableId, String, String), CacheEntry<Value>>>,

    max_entries: usize,
    ttl: Duration,

    // Статистика
    hits: Mutex<u64>,
    misses: Mutex<u64>,
}

impl OperationCache {
    /// Создать новый кэш операций
    pub fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            filter_cache: Mutex::new(HashMap::new()),
            select_cache: Mutex::new(HashMap::new()),
            sort_cache: Mutex::new(HashMap::new()),
            aggregate_cache: Mutex::new(HashMap::new()),
            max_entries,
            ttl,
            hits: Mutex::new(0),
            misses: Mutex::new(0),
        }
    }
    
    /// Получить результат фильтрации из кэша
    pub fn get_filter_result(&self, table_id: &TableId, filter: &FilterExpr) -> Option<Table> {
        let mut cache = self.filter_cache.lock().unwrap();

        if let Some(entry) = cache.get_mut(&(table_id.clone(), filter.clone())) {
            if !entry.is_expired(self.ttl) {
                *self.hits.lock().unwrap() += 1;
                return Some(entry.access().clone());
            } else {
                // Удаляем истекшую запись
                cache.remove(&(table_id.clone(), filter.clone()));
            }
        }

        *self.misses.lock().unwrap() += 1;
        None
    }

    /// Сохранить результат фильтрации в кэш
    pub fn cache_filter_result(&self, table_id: TableId, filter: FilterExpr, result: Table) {
        let mut cache = self.filter_cache.lock().unwrap();

        // Проверяем размер кэша
        if cache.len() >= self.max_entries {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }

        cache.insert((table_id, filter), CacheEntry::new(result));
    }
    
    /// Получить результат выборки колонок из кэша
    pub fn get_select_result(&self, table_id: &TableId, columns: &[String]) -> Option<Table> {
        let mut cache = self.select_cache.lock().unwrap();
        let key = (table_id.clone(), columns.to_vec());

        if let Some(entry) = cache.get_mut(&key) {
            if !entry.is_expired(self.ttl) {
                *self.hits.lock().unwrap() += 1;
                return Some(entry.access().clone());
            } else {
                cache.remove(&key);
            }
        }

        *self.misses.lock().unwrap() += 1;
        None
    }

    /// Сохранить результат выборки колонок в кэш
    pub fn cache_select_result(&self, table_id: TableId, columns: Vec<String>, result: Table) {
        let mut cache = self.select_cache.lock().unwrap();

        if cache.len() >= self.max_entries {
            // Простое удаление первого элемента
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }

        cache.insert((table_id, columns), CacheEntry::new(result));
    }

    /// Получить результат сортировки из кэша
    pub fn get_sort_result(&self, table_id: &TableId, column: &str, ascending: bool) -> Option<Table> {
        let mut cache = self.sort_cache.lock().unwrap();
        let key = (table_id.clone(), column.to_string(), ascending);

        if let Some(entry) = cache.get_mut(&key) {
            if !entry.is_expired(self.ttl) {
                *self.hits.lock().unwrap() += 1;
                return Some(entry.access().clone());
            } else {
                cache.remove(&key);
            }
        }

        *self.misses.lock().unwrap() += 1;
        None
    }

    /// Сохранить результат сортировки в кэш
    pub fn cache_sort_result(&self, table_id: TableId, column: String, ascending: bool, result: Table) {
        let mut cache = self.sort_cache.lock().unwrap();

        if cache.len() >= self.max_entries {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }

        cache.insert((table_id, column, ascending), CacheEntry::new(result));
    }

    /// Получить результат агрегации из кэша
    pub fn get_aggregate_result(&self, table_id: &TableId, column: &str, operation: &str) -> Option<Value> {
        let mut cache = self.aggregate_cache.lock().unwrap();
        let key = (table_id.clone(), column.to_string(), operation.to_string());

        if let Some(entry) = cache.get_mut(&key) {
            if !entry.is_expired(self.ttl) {
                *self.hits.lock().unwrap() += 1;
                return Some(entry.access().clone());
            } else {
                cache.remove(&key);
            }
        }

        *self.misses.lock().unwrap() += 1;
        None
    }

    /// Сохранить результат агрегации в кэш
    pub fn cache_aggregate_result(&self, table_id: TableId, column: String, operation: String, result: Value) {
        let mut cache = self.aggregate_cache.lock().unwrap();

        if cache.len() >= self.max_entries {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }

        cache.insert((table_id, column, operation), CacheEntry::new(result));
    }
    
    /// Очистить все кэши
    pub fn clear_all(&self) {
        self.filter_cache.lock().unwrap().clear();
        self.select_cache.lock().unwrap().clear();
        self.sort_cache.lock().unwrap().clear();
        self.aggregate_cache.lock().unwrap().clear();

        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
    }

    /// Получить статистику кэша
    pub fn get_stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let total = hits + misses;

        CacheStats {
            hits,
            misses,
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
            filter_entries: self.filter_cache.lock().unwrap().len(),
            select_entries: self.select_cache.lock().unwrap().len(),
            sort_entries: self.sort_cache.lock().unwrap().len(),
            aggregate_entries: self.aggregate_cache.lock().unwrap().len(),
        }
    }

}

impl Default for OperationCache {
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(300)) // 1000 записей, 5 минут TTL
    }
}

/// Статистика кэша
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub filter_entries: usize,
    pub select_entries: usize,
    pub sort_entries: usize,
    pub aggregate_entries: usize,
}

impl CacheStats {
    /// Общее количество записей в кэше
    pub fn total_entries(&self) -> usize {
        self.filter_entries + self.select_entries + self.sort_entries + self.aggregate_entries
    }
}

// Глобальный кэш операций временно отключен из-за проблем с потокобезопасностью
// lazy_static::lazy_static! {
//     pub static ref GLOBAL_OPERATION_CACHE: OperationCache = OperationCache::default();
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Table;
    
    #[test]
    fn test_table_id_creation() {
        let mut table = Table::new(vec!["id".to_string(), "name".to_string()]);
        table.rows = vec![
            vec![Value::Number(1.0), Value::String("Alice".to_string())],
            vec![Value::Number(2.0), Value::String("Bob".to_string())],
        ];
        
        let id1 = TableId::from_table(&table);
        let id2 = TableId::from_table(&table);
        
        assert_eq!(id1, id2);
        assert_eq!(id1.hash, id2.hash);
    }
    
    #[test]
    fn test_filter_expr_creation() {
        let expr1 = FilterExpr::simple("age".to_string(), ">".to_string(), "18".to_string());
        let expr2 = FilterExpr::simple("age".to_string(), ">".to_string(), "18".to_string());
        let expr3 = FilterExpr::simple("age".to_string(), "<".to_string(), "18".to_string());
        
        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }
    
    #[test]
    fn test_operation_cache_basic() {
        let cache = OperationCache::new(10, Duration::from_secs(60));
        
        let mut table = Table::new(vec!["id".to_string()]);
        table.rows = vec![vec![Value::Number(1.0)]];
        let table_id = TableId::from_table(&table);
        let filter = FilterExpr::simple("id".to_string(), ">".to_string(), "0".to_string());
        
        // Проверяем, что кэш пуст
        assert!(cache.get_filter_result(&table_id, &filter).is_none());
        
        // Добавляем в кэш
        cache.cache_filter_result(table_id.clone(), filter.clone(), table.clone());
        
        // Проверяем, что результат есть в кэше
        let cached_result = cache.get_filter_result(&table_id, &filter);
        assert!(cached_result.is_some());
        
        // Проверяем статистику
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.filter_entries, 1);
    }
    
    #[test]
    fn test_cache_eviction() {
        let cache = OperationCache::new(2, Duration::from_secs(60));
        
        let table = Table::new(vec!["id".to_string()]);
        let table_id = TableId::from_table(&table);

        // Добавляем 3 записи в кэш с максимумом 2
        for i in 0..3 {
            let filter = FilterExpr::simple("id".to_string(), ">".to_string(), i.to_string());
            cache.cache_filter_result(table_id.clone(), filter, table.clone());
        }
        
        let stats = cache.get_stats();
        assert_eq!(stats.filter_entries, 2); // Должно быть только 2 записи
    }
    
    #[test]
    fn test_cache_expiration() {
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
}
