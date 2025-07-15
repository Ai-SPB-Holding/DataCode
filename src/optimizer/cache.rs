// Кэш парсинга для DataCode
// Кэширует часто используемые выражения для ускорения парсинга

use crate::parser::Expr;
use std::collections::HashMap;
use std::rc::Rc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Кэш парсинга выражений
pub struct ParseCache {
    cache: HashMap<String, Rc<Expr>>,
    hit_count: usize,
    miss_count: usize,
    max_size: usize,
}

impl ParseCache {
    /// Создать новый кэш парсинга
    pub fn new() -> Self {
        Self::with_capacity(1000) // По умолчанию кэшируем 1000 выражений
    }
    
    /// Создать кэш с заданной емкостью
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            hit_count: 0,
            miss_count: 0,
            max_size,
        }
    }
    
    /// Получить выражение из кэша или распарсить новое
    pub fn get_or_parse<F>(&mut self, expr_str: &str, parser_fn: F) -> crate::error::Result<Rc<Expr>>
    where
        F: FnOnce(&str) -> crate::error::Result<Expr>,
    {
        // Создаем ключ для кэша
        let cache_key = self.create_cache_key(expr_str);
        
        // Проверяем кэш
        if let Some(cached_expr) = self.cache.get(&cache_key) {
            self.hit_count += 1;
            return Ok(cached_expr.clone());
        }
        
        // Парсим новое выражение
        self.miss_count += 1;
        let parsed_expr = parser_fn(expr_str)?;
        let rc_expr = Rc::new(parsed_expr);
        
        // Добавляем в кэш, если есть место
        if self.cache.len() < self.max_size {
            self.cache.insert(cache_key, rc_expr.clone());
        } else {
            // Если кэш полон, удаляем случайный элемент (простая стратегия)
            if let Some(key_to_remove) = self.cache.keys().next().cloned() {
                self.cache.remove(&key_to_remove);
                self.cache.insert(cache_key, rc_expr.clone());
            }
        }
        
        Ok(rc_expr)
    }
    
    /// Получить выражение из кэша
    pub fn get(&mut self, expr_str: &str) -> Option<Rc<Expr>> {
        let cache_key = self.create_cache_key(expr_str);
        if let Some(cached_expr) = self.cache.get(&cache_key) {
            self.hit_count += 1;
            Some(cached_expr.clone())
        } else {
            self.miss_count += 1;
            None
        }
    }
    
    /// Добавить выражение в кэш
    pub fn insert(&mut self, expr_str: &str, expr: Expr) {
        let cache_key = self.create_cache_key(expr_str);
        
        if self.cache.len() < self.max_size {
            self.cache.insert(cache_key, Rc::new(expr));
        } else {
            // Простая стратегия замещения - удаляем первый элемент
            if let Some(key_to_remove) = self.cache.keys().next().cloned() {
                self.cache.remove(&key_to_remove);
                self.cache.insert(cache_key, Rc::new(expr));
            }
        }
    }
    
    /// Очистить кэш
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }
    
    /// Получить количество попаданий в кэш
    pub fn get_hit_count(&self) -> usize {
        self.hit_count
    }
    
    /// Получить количество промахов кэша
    pub fn get_miss_count(&self) -> usize {
        self.miss_count
    }
    
    /// Получить коэффициент попаданий в кэш
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
    
    /// Получить размер кэша
    pub fn size(&self) -> usize {
        self.cache.len()
    }
    
    /// Получить максимальный размер кэша
    pub fn max_size(&self) -> usize {
        self.max_size
    }
    
    /// Проверить, заполнен ли кэш
    pub fn is_full(&self) -> bool {
        self.cache.len() >= self.max_size
    }
    
    /// Получить статистику кэша
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hit_count,
            misses: self.miss_count,
            size: self.cache.len(),
            max_size: self.max_size,
            hit_ratio: self.hit_ratio(),
        }
    }
    
    /// Создать ключ для кэша на основе строки выражения
    fn create_cache_key(&self, expr_str: &str) -> String {
        // Нормализуем строку (убираем лишние пробелы)
        let normalized = expr_str.trim().replace("  ", " ");
        
        // Для длинных выражений используем хэш
        if normalized.len() > 100 {
            let mut hasher = DefaultHasher::new();
            normalized.hash(&mut hasher);
            format!("hash_{}", hasher.finish())
        } else {
            normalized
        }
    }
    
    /// Предварительно заполнить кэш часто используемыми выражениями
    pub fn preload_common_expressions(&mut self) {
        let common_expressions = vec![
            "true",
            "false",
            "null",
            "0",
            "1",
            "-1",
            "''",
            "[]",
            "{}",
            "age > 0",
            "age >= 18",
            "age < 100",
            "name != null",
            "name != ''",
            "id > 0",
        ];
        
        for expr_str in common_expressions {
            // Парсим и добавляем в кэш
            if let Ok(expr) = crate::parser::Parser::new(expr_str).parse_expression() {
                self.insert(expr_str, expr);
            }
        }
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика кэша парсинга
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub size: usize,
    pub max_size: usize,
    pub hit_ratio: f64,
}

impl CacheStats {
    /// Проверить, эффективен ли кэш
    pub fn is_effective(&self) -> bool {
        self.hit_ratio > 0.3 && self.hits > 10
    }
    
    /// Получить строковое представление статистики
    pub fn to_string(&self) -> String {
        format!(
            "Cache Stats: {}/{} hits ({:.1}%), size: {}/{}",
            self.hits,
            self.hits + self.misses,
            self.hit_ratio * 100.0,
            self.size,
            self.max_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_cache_basic_operations() {
        let mut cache = ParseCache::new();
        
        // Тест промаха
        let result1 = cache.get_or_parse("1 + 2", |s| {
            Parser::new(s).parse_expression()
        });
        assert!(result1.is_ok());
        assert_eq!(cache.get_miss_count(), 1);
        assert_eq!(cache.get_hit_count(), 0);
        
        // Тест попадания
        let result2 = cache.get_or_parse("1 + 2", |s| {
            Parser::new(s).parse_expression()
        });
        assert!(result2.is_ok());
        assert_eq!(cache.get_miss_count(), 1);
        assert_eq!(cache.get_hit_count(), 1);
    }
    
    #[test]
    fn test_cache_stats() {
        let mut cache = ParseCache::new();
        
        // Добавляем несколько выражений
        for i in 0..5 {
            let expr_str = format!("{} + {}", i, i + 1);
            let _ = cache.get_or_parse(&expr_str, |s| {
                Parser::new(s).parse_expression()
            });
        }
        
        // Повторяем первые 3
        for i in 0..3 {
            let expr_str = format!("{} + {}", i, i + 1);
            let _ = cache.get_or_parse(&expr_str, |s| {
                Parser::new(s).parse_expression()
            });
        }
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 5);
        assert_eq!(stats.size, 5);
        assert!(stats.hit_ratio > 0.0);
    }
    
    #[test]
    fn test_cache_key_normalization() {
        let cache = ParseCache::new();
        
        let key1 = cache.create_cache_key("  1 + 2  ");
        let key2 = cache.create_cache_key("1 + 2");
        let key3 = cache.create_cache_key("1  +  2");
        
        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
    }
}
