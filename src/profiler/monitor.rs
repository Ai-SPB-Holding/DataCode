// Система мониторинга оптимизаций DataCode
// Отслеживает эффективность различных оптимизаций и предоставляет рекомендации

use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::profiler::Profiler;
use crate::profiler::logger::{PerformanceLogger, LogLevel, PerformanceLogEntry};

/// Типы оптимизаций, которые мониторятся
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptimizationType {
    LazyEvaluation,
    ASTOptimization,
    ParseCaching,
    Vectorization,
    Parallelization,
    MemoryOptimization,
}

/// Метрики эффективности оптимизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    pub optimization_type: OptimizationType,
    pub enabled: bool,
    pub hit_count: u64,
    pub miss_count: u64,
    pub total_time_saved: Duration,
    pub memory_saved: usize,
    pub operations_optimized: u64,
    pub effectiveness_score: f64, // 0.0 - 1.0
}

impl OptimizationMetrics {
    pub fn new(optimization_type: OptimizationType) -> Self {
        Self {
            optimization_type,
            enabled: true,
            hit_count: 0,
            miss_count: 0,
            total_time_saved: Duration::ZERO,
            memory_saved: 0,
            operations_optimized: 0,
            effectiveness_score: 0.0,
        }
    }
    
    /// Записать успешное применение оптимизации
    pub fn record_hit(&mut self, time_saved: Duration, memory_saved: usize) {
        self.hit_count += 1;
        self.total_time_saved += time_saved;
        self.memory_saved += memory_saved;
        self.operations_optimized += 1;
        self.update_effectiveness_score();
    }
    
    /// Записать неуспешное применение оптимизации
    pub fn record_miss(&mut self) {
        self.miss_count += 1;
        self.update_effectiveness_score();
    }
    
    /// Обновить оценку эффективности
    fn update_effectiveness_score(&mut self) {
        let total_attempts = self.hit_count + self.miss_count;
        if total_attempts == 0 {
            self.effectiveness_score = 0.0;
            return;
        }
        
        let hit_rate = self.hit_count as f64 / total_attempts as f64;
        let time_factor = self.total_time_saved.as_millis() as f64 / 1000.0; // секунды
        let memory_factor = self.memory_saved as f64 / (1024.0 * 1024.0); // MB
        
        // Комбинированная оценка: hit rate * (time_saved + memory_saved)
        self.effectiveness_score = hit_rate * (time_factor + memory_factor).min(10.0) / 10.0;
    }
    
    /// Получить коэффициент попаданий
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

/// Монитор оптимизаций
pub struct OptimizationMonitor {
    metrics: HashMap<OptimizationType, OptimizationMetrics>,
    _profiler: Profiler,
    logger: PerformanceLogger,
    baseline_measurements: HashMap<String, Duration>,
}

impl OptimizationMonitor {
    /// Создать новый монитор
    pub fn new() -> Self {
        let mut metrics = HashMap::new();
        
        // Инициализируем метрики для всех типов оптимизаций
        metrics.insert(OptimizationType::LazyEvaluation, OptimizationMetrics::new(OptimizationType::LazyEvaluation));
        metrics.insert(OptimizationType::ASTOptimization, OptimizationMetrics::new(OptimizationType::ASTOptimization));
        metrics.insert(OptimizationType::ParseCaching, OptimizationMetrics::new(OptimizationType::ParseCaching));
        metrics.insert(OptimizationType::Vectorization, OptimizationMetrics::new(OptimizationType::Vectorization));
        metrics.insert(OptimizationType::Parallelization, OptimizationMetrics::new(OptimizationType::Parallelization));
        metrics.insert(OptimizationType::MemoryOptimization, OptimizationMetrics::new(OptimizationType::MemoryOptimization));
        
        Self {
            metrics,
            _profiler: Profiler::new(),
            logger: PerformanceLogger::new(),
            baseline_measurements: HashMap::new(),
        }
    }
    
    /// Записать базовое измерение (без оптимизаций)
    pub fn record_baseline(&mut self, operation: &str, duration: Duration) {
        self.baseline_measurements.insert(operation.to_string(), duration);
    }
    
    /// Записать успешное применение оптимизации
    pub fn record_optimization_hit(&mut self, 
        optimization_type: OptimizationType, 
        operation: &str, 
        optimized_duration: Duration,
        memory_saved: usize) {
        
        if let Some(baseline) = self.baseline_measurements.get(operation) {
            let time_saved = baseline.saturating_sub(optimized_duration);
            
            if let Some(metrics) = self.metrics.get_mut(&optimization_type) {
                metrics.record_hit(time_saved, memory_saved);
                
                // Логируем успешную оптимизацию
                let entry = PerformanceLogEntry::new(
                    LogLevel::Info,
                    format!("optimization_{:?}", optimization_type),
                    format!("Optimization applied to {}", operation)
                )
                .with_duration(time_saved)
                .with_memory_usage(memory_saved)
                .with_metadata("operation".to_string(), operation.to_string())
                .with_metadata("baseline_ms".to_string(), baseline.as_millis().to_string())
                .with_metadata("optimized_ms".to_string(), optimized_duration.as_millis().to_string());
                
                self.logger.log(entry);
            }
        }
    }
    
    /// Записать неуспешное применение оптимизации
    pub fn record_optimization_miss(&mut self, optimization_type: OptimizationType, operation: &str) {
        if let Some(metrics) = self.metrics.get_mut(&optimization_type) {
            metrics.record_miss();
            
            let entry = PerformanceLogEntry::new(
                LogLevel::Debug,
                format!("optimization_{:?}", optimization_type),
                format!("Optimization not applied to {}", operation)
            )
            .with_metadata("operation".to_string(), operation.to_string());
            
            self.logger.log(entry);
        }
    }
    
    /// Включить/выключить оптимизацию
    pub fn set_optimization_enabled(&mut self, optimization_type: OptimizationType, enabled: bool) {
        if let Some(metrics) = self.metrics.get_mut(&optimization_type) {
            metrics.enabled = enabled;
        }
    }
    
    /// Проверить, включена ли оптимизация
    pub fn is_optimization_enabled(&self, optimization_type: &OptimizationType) -> bool {
        self.metrics.get(optimization_type)
            .map(|m| m.enabled)
            .unwrap_or(false)
    }
    
    /// Получить метрики оптимизации
    pub fn get_optimization_metrics(&self, optimization_type: &OptimizationType) -> Option<&OptimizationMetrics> {
        self.metrics.get(optimization_type)
    }
    
    /// Получить все метрики
    pub fn get_all_metrics(&self) -> &HashMap<OptimizationType, OptimizationMetrics> {
        &self.metrics
    }
    
    /// Получить рекомендации по оптимизации
    pub fn get_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        for (opt_type, metrics) in &self.metrics {
            // Рекомендация отключить неэффективные оптимизации
            if metrics.enabled && metrics.effectiveness_score < 0.1 && metrics.hit_count + metrics.miss_count > 100 {
                recommendations.push(OptimizationRecommendation {
                    optimization_type: opt_type.clone(),
                    action: RecommendationAction::Disable,
                    reason: format!("Low effectiveness score: {:.2}", metrics.effectiveness_score),
                    priority: RecommendationPriority::Medium,
                });
            }
            
            // Рекомендация включить эффективные оптимизации
            if !metrics.enabled && metrics.effectiveness_score > 0.7 {
                recommendations.push(OptimizationRecommendation {
                    optimization_type: opt_type.clone(),
                    action: RecommendationAction::Enable,
                    reason: format!("High effectiveness score: {:.2}", metrics.effectiveness_score),
                    priority: RecommendationPriority::High,
                });
            }
            
            // Рекомендация настроить параметры
            if metrics.hit_rate() < 0.5 && metrics.hit_count + metrics.miss_count > 50 {
                recommendations.push(OptimizationRecommendation {
                    optimization_type: opt_type.clone(),
                    action: RecommendationAction::Tune,
                    reason: format!("Low hit rate: {:.2}", metrics.hit_rate()),
                    priority: RecommendationPriority::Low,
                });
            }
        }
        
        // Сортируем по приоритету
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations
    }
    
    /// Сгенерировать отчет о производительности
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let mut total_time_saved = Duration::ZERO;
        let mut total_memory_saved = 0;
        let mut total_operations_optimized = 0;
        
        for metrics in self.metrics.values() {
            total_time_saved += metrics.total_time_saved;
            total_memory_saved += metrics.memory_saved;
            total_operations_optimized += metrics.operations_optimized;
        }
        
        PerformanceReport {
            total_time_saved,
            total_memory_saved,
            total_operations_optimized,
            optimization_metrics: self.metrics.clone(),
            recommendations: self.get_recommendations(),
        }
    }
    
    /// Экспортировать данные в JSON
    pub fn export_json(&self) -> String {
        let report = self.generate_performance_report();
        serde_json::to_string_pretty(&report).unwrap_or_default()
    }
}

impl Default for OptimizationMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Рекомендация по оптимизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub optimization_type: OptimizationType,
    pub action: RecommendationAction,
    pub reason: String,
    pub priority: RecommendationPriority,
}

/// Действие рекомендации
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationAction {
    Enable,
    Disable,
    Tune,
}

/// Приоритет рекомендации
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
}

/// Отчет о производительности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub total_time_saved: Duration,
    pub total_memory_saved: usize,
    pub total_operations_optimized: u64,
    pub optimization_metrics: HashMap<OptimizationType, OptimizationMetrics>,
    pub recommendations: Vec<OptimizationRecommendation>,
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_OPTIMIZATION_MONITOR: std::sync::Mutex<OptimizationMonitor> = 
        std::sync::Mutex::new(OptimizationMonitor::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimization_metrics() {
        let mut metrics = OptimizationMetrics::new(OptimizationType::LazyEvaluation);
        
        // Тест записи попадания
        metrics.record_hit(Duration::from_millis(100), 1024);
        assert_eq!(metrics.hit_count, 1);
        assert_eq!(metrics.total_time_saved, Duration::from_millis(100));
        assert_eq!(metrics.memory_saved, 1024);
        assert!(metrics.effectiveness_score > 0.0);
        
        // Тест записи промаха
        metrics.record_miss();
        assert_eq!(metrics.miss_count, 1);
        assert_eq!(metrics.hit_rate(), 0.5);
    }
    
    #[test]
    fn test_optimization_monitor() {
        let mut monitor = OptimizationMonitor::new();
        
        // Записываем базовое измерение
        monitor.record_baseline("test_operation", Duration::from_millis(200));
        
        // Записываем успешную оптимизацию
        monitor.record_optimization_hit(
            OptimizationType::LazyEvaluation,
            "test_operation",
            Duration::from_millis(100),
            512
        );
        
        let metrics = monitor.get_optimization_metrics(&OptimizationType::LazyEvaluation).unwrap();
        assert_eq!(metrics.hit_count, 1);
        assert_eq!(metrics.total_time_saved, Duration::from_millis(100));
        assert_eq!(metrics.memory_saved, 512);
    }
}
