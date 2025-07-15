// Система логирования производительности для DataCode
// Предоставляет детальные логи операций с метриками производительности

use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// Уровни логирования производительности
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// Запись лога производительности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub operation: String,
    pub duration: Option<u64>, // в микросекундах
    pub input_size: Option<usize>,
    pub output_size: Option<usize>,
    pub memory_usage: Option<usize>,
    pub message: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl PerformanceLogEntry {
    /// Создать новую запись лога
    pub fn new(level: LogLevel, operation: String, message: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            timestamp,
            level,
            operation,
            duration: None,
            input_size: None,
            output_size: None,
            memory_usage: None,
            message,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Установить длительность операции
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration.as_micros() as u64);
        self
    }
    
    /// Установить размер входных данных
    pub fn with_input_size(mut self, size: usize) -> Self {
        self.input_size = Some(size);
        self
    }
    
    /// Установить размер выходных данных
    pub fn with_output_size(mut self, size: usize) -> Self {
        self.output_size = Some(size);
        self
    }
    
    /// Установить использование памяти
    pub fn with_memory_usage(mut self, bytes: usize) -> Self {
        self.memory_usage = Some(bytes);
        self
    }
    
    /// Добавить метаданные
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Форматировать запись для вывода
    pub fn format(&self) -> String {
        let level_str = match self.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        };
        
        let mut parts = vec![
            format!("[{}]", level_str),
            format!("{}", self.operation),
        ];
        
        if let Some(duration) = self.duration {
            parts.push(format!("{}μs", duration));
        }
        
        if let Some(input_size) = self.input_size {
            parts.push(format!("in:{}", format_size(input_size)));
        }
        
        if let Some(output_size) = self.output_size {
            parts.push(format!("out:{}", format_size(output_size)));
        }
        
        if let Some(memory) = self.memory_usage {
            parts.push(format!("mem:{}", format_size(memory)));
        }
        
        parts.push(self.message.clone());
        
        if !self.metadata.is_empty() {
            let metadata_str = self.metadata.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            parts.push(format!("[{}]", metadata_str));
        }
        
        parts.join(" ")
    }
}

/// Логгер производительности
pub struct PerformanceLogger {
    file_writer: Option<Arc<Mutex<BufWriter<File>>>>,
    console_enabled: bool,
    min_level: LogLevel,
    buffer: Arc<Mutex<Vec<PerformanceLogEntry>>>,
}

impl PerformanceLogger {
    /// Создать новый логгер
    pub fn new() -> Self {
        Self {
            file_writer: None,
            console_enabled: true,
            min_level: LogLevel::Info,
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Включить запись в файл
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        
        self.file_writer = Some(Arc::new(Mutex::new(BufWriter::new(file))));
        Ok(self)
    }
    
    /// Установить минимальный уровень логирования
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }
    
    /// Включить/выключить вывод в консоль
    pub fn with_console(mut self, enabled: bool) -> Self {
        self.console_enabled = enabled;
        self
    }
    
    /// Записать лог
    pub fn log(&self, entry: PerformanceLogEntry) {
        if !self.should_log(entry.level) {
            return;
        }
        
        // Добавляем в буфер
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(entry.clone());
            
            // Ограничиваем размер буфера
            if buffer.len() > 10000 {
                buffer.drain(0..5000);
            }
        }
        
        let formatted = entry.format();
        
        // Вывод в консоль
        if self.console_enabled {
            println!("{}", formatted);
        }
        
        // Запись в файл
        if let Some(ref writer) = self.file_writer {
            if let Ok(mut writer) = writer.lock() {
                let _ = writeln!(writer, "{}", formatted);
                let _ = writer.flush();
            }
        }
    }
    
    /// Проверить, нужно ли логировать данный уровень
    fn should_log(&self, level: LogLevel) -> bool {
        match (self.min_level, level) {
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Debug) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Warning, LogLevel::Debug | LogLevel::Info) => false,
            (LogLevel::Warning, _) => true,
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Error, _) => false,
        }
    }
    
    /// Получить записи из буфера
    pub fn get_buffer(&self) -> Vec<PerformanceLogEntry> {
        self.buffer.lock().unwrap().clone()
    }
    
    /// Очистить буфер
    pub fn clear_buffer(&self) {
        self.buffer.lock().unwrap().clear();
    }
    
    /// Экспортировать буфер в JSON
    pub fn export_buffer_json(&self) -> String {
        let buffer = self.get_buffer();
        serde_json::to_string_pretty(&buffer).unwrap_or_default()
    }
    
    /// Получить статистику по операциям из буфера
    pub fn get_operation_stats(&self) -> std::collections::HashMap<String, OperationSummary> {
        let buffer = self.get_buffer();
        let mut stats = std::collections::HashMap::new();
        
        for entry in buffer {
            let summary = stats.entry(entry.operation.clone())
                .or_insert_with(|| OperationSummary::new(entry.operation.clone()));
            
            summary.count += 1;
            
            if let Some(duration) = entry.duration {
                summary.total_duration += duration;
                summary.min_duration = summary.min_duration.min(duration);
                summary.max_duration = summary.max_duration.max(duration);
            }
            
            if let Some(memory) = entry.memory_usage {
                summary.total_memory += memory;
                summary.max_memory = summary.max_memory.max(memory);
            }
        }
        
        // Вычисляем средние значения
        for summary in stats.values_mut() {
            if summary.count > 0 {
                summary.avg_duration = summary.total_duration / summary.count as u64;
                summary.avg_memory = summary.total_memory / summary.count;
            }
        }
        
        stats
    }
}

impl Default for PerformanceLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Сводка по операции
#[derive(Debug, Clone)]
pub struct OperationSummary {
    pub operation: String,
    pub count: usize,
    pub total_duration: u64,
    pub avg_duration: u64,
    pub min_duration: u64,
    pub max_duration: u64,
    pub total_memory: usize,
    pub avg_memory: usize,
    pub max_memory: usize,
}

impl OperationSummary {
    fn new(operation: String) -> Self {
        Self {
            operation,
            count: 0,
            total_duration: 0,
            avg_duration: 0,
            min_duration: u64::MAX,
            max_duration: 0,
            total_memory: 0,
            avg_memory: 0,
            max_memory: 0,
        }
    }
}

/// Форматировать размер в человекочитаемом виде
fn format_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{}{}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}

/// Глобальный логгер производительности
lazy_static::lazy_static! {
    pub static ref GLOBAL_PERFORMANCE_LOGGER: PerformanceLogger = {
        PerformanceLogger::new()
            .with_console(true)
            .with_min_level(LogLevel::Info)
    };
}

/// Макросы для удобного логирования
#[macro_export]
macro_rules! perf_log {
    ($level:expr, $operation:expr, $message:expr) => {
        $crate::profiler::logger::GLOBAL_PERFORMANCE_LOGGER.log(
            $crate::profiler::logger::PerformanceLogEntry::new($level, $operation.to_string(), $message.to_string())
        );
    };
    
    ($level:expr, $operation:expr, $message:expr, $($key:expr => $value:expr),*) => {
        let mut entry = $crate::profiler::logger::PerformanceLogEntry::new($level, $operation.to_string(), $message.to_string());
        $(
            entry = entry.with_metadata($key.to_string(), $value.to_string());
        )*
        $crate::profiler::logger::GLOBAL_PERFORMANCE_LOGGER.log(entry);
    };
}

#[macro_export]
macro_rules! perf_info {
    ($operation:expr, $message:expr) => {
        perf_log!($crate::profiler::logger::LogLevel::Info, $operation, $message);
    };
    ($operation:expr, $message:expr, $($key:expr => $value:expr),*) => {
        perf_log!($crate::profiler::logger::LogLevel::Info, $operation, $message, $($key => $value),*);
    };
}

#[macro_export]
macro_rules! perf_debug {
    ($operation:expr, $message:expr) => {
        perf_log!($crate::profiler::logger::LogLevel::Debug, $operation, $message);
    };
    ($operation:expr, $message:expr, $($key:expr => $value:expr),*) => {
        perf_log!($crate::profiler::logger::LogLevel::Debug, $operation, $message, $($key => $value),*);
    };
}
