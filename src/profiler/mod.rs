// Система профилирования для DataCode
// Отслеживает производительность операций и предоставляет детальную статистику

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use serde::Serializer;

/// Профилировщик операций DataCode
pub struct Profiler {
    timings: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
    counters: Arc<Mutex<HashMap<String, u64>>>,
    memory_usage: Arc<Mutex<HashMap<String, usize>>>,
    active_timers: Arc<Mutex<HashMap<String, Instant>>>,
}

impl Profiler {
    /// Создать новый профилировщик
    pub fn new() -> Self {
        Self {
            timings: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            memory_usage: Arc::new(Mutex::new(HashMap::new())),
            active_timers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Начать измерение времени операции
    pub fn start_timer(&self, operation: &str) -> TimerGuard {
        let mut timers = self.active_timers.lock().unwrap();
        timers.insert(operation.to_string(), Instant::now());
        
        TimerGuard {
            operation: operation.to_string(),
            profiler: self,
        }
    }
    
    /// Завершить измерение времени операции
    pub fn end_timer(&self, operation: &str) {
        let mut timers = self.active_timers.lock().unwrap();
        if let Some(start_time) = timers.remove(operation) {
            let duration = start_time.elapsed();
            
            let mut timings = self.timings.lock().unwrap();
            timings.entry(operation.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }
    
    /// Увеличить счетчик операции
    pub fn increment_counter(&self, operation: &str) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(operation.to_string()).or_insert(0) += 1;
    }
    
    /// Записать использование памяти
    pub fn record_memory_usage(&self, operation: &str, bytes: usize) {
        let mut memory = self.memory_usage.lock().unwrap();
        memory.insert(operation.to_string(), bytes);
    }
    
    /// Получить статистику по операции
    pub fn get_operation_stats(&self, operation: &str) -> OperationStats {
        let timings = self.timings.lock().unwrap();
        let counters = self.counters.lock().unwrap();
        let memory = self.memory_usage.lock().unwrap();
        
        let durations = timings.get(operation).cloned().unwrap_or_default();
        let count = counters.get(operation).copied().unwrap_or(0);
        let memory_bytes = memory.get(operation).copied().unwrap_or(0);
        
        OperationStats::new(operation.to_string(), durations, count, memory_bytes)
    }
    
    /// Получить общую статистику
    pub fn get_summary(&self) -> ProfilerSummary {
        // Собираем все ключи операций без удержания блокировок
        let timing_keys: Vec<String> = {
            let timings = self.timings.lock().unwrap();
            timings.keys().cloned().collect()
        };

        let counter_keys: Vec<String> = {
            let counters = self.counters.lock().unwrap();
            counters.keys().cloned().collect()
        };

        let total_operations: u64 = {
            let counters = self.counters.lock().unwrap();
            counters.values().sum()
        };

        let total_memory_usage: usize = {
            let memory = self.memory_usage.lock().unwrap();
            memory.values().sum()
        };

        let mut operations = Vec::new();
        let mut processed_operations = std::collections::HashSet::new();

        // Добавляем операции из timings
        for operation in timing_keys {
            operations.push(self.get_operation_stats(&operation));
            processed_operations.insert(operation);
        }

        // Добавляем операции, которые есть только в счетчиках
        for operation in counter_keys {
            if !processed_operations.contains(&operation) {
                operations.push(self.get_operation_stats(&operation));
            }
        }

        ProfilerSummary {
            operations,
            total_operations,
            total_memory_usage,
        }
    }
    
    /// Очистить все данные профилирования
    pub fn clear(&self) {
        self.timings.lock().unwrap().clear();
        self.counters.lock().unwrap().clear();
        self.memory_usage.lock().unwrap().clear();
        self.active_timers.lock().unwrap().clear();
    }
    
    /// Экспортировать данные в JSON формат
    pub fn export_json(&self) -> String {
        let summary = self.get_summary();
        serde_json::to_string_pretty(&summary).unwrap_or_default()
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard для автоматического завершения таймера
pub struct TimerGuard<'a> {
    operation: String,
    profiler: &'a Profiler,
}

impl<'a> Drop for TimerGuard<'a> {
    fn drop(&mut self) {
        self.profiler.end_timer(&self.operation);
    }
}

/// Статистика по отдельной операции
#[derive(Debug, Clone, serde::Serialize)]
pub struct OperationStats {
    pub operation: String,
    pub count: u64,
    #[serde(serialize_with = "serialize_duration")]
    pub total_time: Duration,
    #[serde(serialize_with = "serialize_duration")]
    pub avg_time: Duration,
    #[serde(serialize_with = "serialize_duration")]
    pub min_time: Duration,
    #[serde(serialize_with = "serialize_duration")]
    pub max_time: Duration,
    pub memory_usage: usize,
}

impl OperationStats {
    fn new(operation: String, durations: Vec<Duration>, count: u64, memory_usage: usize) -> Self {
        let total_time = durations.iter().sum();
        let avg_time = if durations.is_empty() {
            Duration::ZERO
        } else {
            total_time / durations.len() as u32
        };
        let min_time = durations.iter().min().copied().unwrap_or(Duration::ZERO);
        let max_time = durations.iter().max().copied().unwrap_or(Duration::ZERO);
        
        Self {
            operation,
            count,
            total_time,
            avg_time,
            min_time,
            max_time,
            memory_usage,
        }
    }
}

/// Общая сводка профилирования
#[derive(Debug, serde::Serialize)]
pub struct ProfilerSummary {
    pub operations: Vec<OperationStats>,
    pub total_operations: u64,
    pub total_memory_usage: usize,
}

impl ProfilerSummary {
    /// Получить топ операций по времени выполнения
    pub fn top_by_time(&self, limit: usize) -> Vec<&OperationStats> {
        let mut ops = self.operations.iter().collect::<Vec<_>>();
        ops.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        ops.into_iter().take(limit).collect()
    }
    
    /// Получить топ операций по количеству вызовов
    pub fn top_by_count(&self, limit: usize) -> Vec<&OperationStats> {
        let mut ops = self.operations.iter().collect::<Vec<_>>();
        ops.sort_by(|a, b| b.count.cmp(&a.count));
        ops.into_iter().take(limit).collect()
    }
    
    /// Получить топ операций по использованию памяти
    pub fn top_by_memory(&self, limit: usize) -> Vec<&OperationStats> {
        let mut ops = self.operations.iter().collect::<Vec<_>>();
        ops.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
        ops.into_iter().take(limit).collect()
    }
}

pub mod logger;
pub mod monitor;

/// Сериализация Duration в микросекундах
fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_micros() as u64)
}

/// Глобальный профилировщик
lazy_static::lazy_static! {
    pub static ref GLOBAL_PROFILER: Profiler = Profiler::new();
}

/// Макрос для удобного профилирования
#[macro_export]
macro_rules! profile {
    ($operation:expr, $code:block) => {{
        let _timer = $crate::profiler::GLOBAL_PROFILER.start_timer($operation);
        $crate::profiler::GLOBAL_PROFILER.increment_counter($operation);
        $code
    }};
}

/// Макрос для профилирования с записью памяти
#[macro_export]
macro_rules! profile_memory {
    ($operation:expr, $memory:expr, $code:block) => {{
        let _timer = $crate::profiler::GLOBAL_PROFILER.start_timer($operation);
        $crate::profiler::GLOBAL_PROFILER.increment_counter($operation);
        $crate::profiler::GLOBAL_PROFILER.record_memory_usage($operation, $memory);
        $code
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_profiler_basic() {
        let profiler = Profiler::new();
        
        // Тест таймера
        {
            let _timer = profiler.start_timer("test_operation");
            thread::sleep(Duration::from_millis(10));
        }
        
        // Тест счетчика
        profiler.increment_counter("test_counter");
        profiler.increment_counter("test_counter");
        
        // Тест памяти
        profiler.record_memory_usage("test_memory", 1024);
        
        let stats = profiler.get_operation_stats("test_operation");
        assert_eq!(stats.operation, "test_operation");
        assert!(stats.total_time > Duration::ZERO);
        
        let counter_stats = profiler.get_operation_stats("test_counter");
        assert_eq!(counter_stats.count, 2);
        
        let memory_stats = profiler.get_operation_stats("test_memory");
        assert_eq!(memory_stats.memory_usage, 1024);
    }
    
    #[test]
    fn test_profiler_summary() {
        let profiler = Profiler::new();
        
        profiler.increment_counter("op1");
        profiler.increment_counter("op2");
        profiler.increment_counter("op2");
        
        let summary = profiler.get_summary();
        assert_eq!(summary.total_operations, 3);
        
        let top_by_count = summary.top_by_count(1);
        assert_eq!(top_by_count[0].operation, "op2");
        assert_eq!(top_by_count[0].count, 2);
    }
}
