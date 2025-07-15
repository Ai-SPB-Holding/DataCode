// Модуль оптимизации AST для DataCode
// Реализует различные оптимизации для ускорения выполнения

pub mod ast;
pub mod cache;
pub mod static_analysis;

// Реэкспорт основных типов
pub use ast::ASTOptimizer;
pub use cache::ParseCache;
pub use static_analysis::StaticAnalyzer;

use crate::parser::Expr;
use crate::error::Result;

/// Главный оптимизатор, координирующий все виды оптимизаций
pub struct Optimizer {
    ast_optimizer: ASTOptimizer,
    parse_cache: ParseCache,
    static_analyzer: StaticAnalyzer,
}

impl Optimizer {
    /// Создать новый оптимизатор
    pub fn new() -> Self {
        Self {
            ast_optimizer: ASTOptimizer::new(),
            parse_cache: ParseCache::new(),
            static_analyzer: StaticAnalyzer::new(),
        }
    }
    
    /// Оптимизировать выражение
    pub fn optimize(&mut self, expr: Expr) -> Result<Expr> {
        // 1. Статический анализ
        self.static_analyzer.analyze(&expr)?;

        // 2. AST оптимизация
        let optimized = self.ast_optimizer.optimize(expr)?;

        Ok(optimized)
    }

    /// Оптимизировать выражение из строки с кэшированием
    pub fn optimize_expression(&mut self, expr_str: &str) -> Result<Expr> {
        // Проверяем кэш
        if let Some(cached_expr) = self.parse_cache.get(expr_str) {
            return Ok((*cached_expr).clone());
        }

        // Парсим выражение
        let mut parser = crate::parser::Parser::new(expr_str);
        let expr = parser.parse_expression()?;

        // Оптимизируем
        let optimized_expr = self.optimize(expr)?;

        // Кэшируем результат
        self.parse_cache.insert(expr_str, optimized_expr.clone());

        Ok(optimized_expr)
    }
    
    /// Получить кэш парсинга
    pub fn parse_cache(&mut self) -> &mut ParseCache {
        &mut self.parse_cache
    }
    
    /// Получить статистику оптимизаций
    pub fn get_stats(&self) -> OptimizerStats {
        OptimizerStats {
            ast_optimizations: self.ast_optimizer.get_optimization_count(),
            cache_hits: self.parse_cache.get_hit_count(),
            cache_misses: self.parse_cache.get_miss_count(),
            static_analysis_errors: self.static_analyzer.get_error_count(),
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика работы оптимизатора
#[derive(Debug, Clone)]
pub struct OptimizerStats {
    pub ast_optimizations: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub static_analysis_errors: usize,
}

impl OptimizerStats {
    /// Получить коэффициент попаданий в кэш
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
    
    /// Проверить, эффективен ли оптимизатор
    pub fn is_effective(&self) -> bool {
        self.cache_hit_ratio() > 0.5 && self.ast_optimizations > 0
    }
}
