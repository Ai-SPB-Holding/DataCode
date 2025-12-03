// Comprehensive Performance Analysis for DataCode Interpreter
// This module provides detailed performance testing and bottleneck identification
// for the DataCode interpreter architecture and optimization planning

use data_code::interpreter::Interpreter;
use std::time::{Duration, Instant};

/// Performance metrics collection structure
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub execution_time: Duration,
    pub memory_usage_estimate: usize,
    pub iterations: usize,
    pub throughput: f64, // operations per second
}

impl PerformanceMetrics {
    pub fn new(name: &str, duration: Duration, iterations: usize) -> Self {
        let throughput = if duration.as_secs_f64() > 0.0 {
            iterations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        
        Self {
            operation_name: name.to_string(),
            execution_time: duration,
            memory_usage_estimate: 0, // To be implemented with proper memory profiling
            iterations,
            throughput,
        }
    }
}

/// Performance analysis suite for DataCode interpreter
pub struct PerformanceAnalyzer {
    pub metrics: Vec<PerformanceMetrics>,
    pub interpreter: Interpreter,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            interpreter: Interpreter::new(),
        }
    }
    
    /// Run a performance test and collect metrics
    pub fn run_test<F>(&mut self, test_name: &str, iterations: usize, test_fn: F) -> PerformanceMetrics
    where
        F: Fn(&mut Interpreter) -> Result<(), String>,
    {
        let start = Instant::now();
        
        for _ in 0..iterations {
            if let Err(e) = test_fn(&mut self.interpreter) {
                panic!("Test {} failed: {}", test_name, e);
            }
        }
        
        let duration = start.elapsed();
        let metrics = PerformanceMetrics::new(test_name, duration, iterations);
        
        println!("Test: {} | Duration: {:?} | Throughput: {:.2} ops/sec", 
                 test_name, duration, metrics.throughput);
        
        self.metrics.push(metrics.clone());
        metrics
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== DATACODE INTERPRETER PERFORMANCE ANALYSIS REPORT ===\n\n");
        
        for metric in &self.metrics {
            report.push_str(&format!(
                "Operation: {}\n  Execution Time: {:?}\n  Iterations: {}\n  Throughput: {:.2} ops/sec\n\n",
                metric.operation_name,
                metric.execution_time,
                metric.iterations,
                metric.throughput
            ));
        }
        
        // Identify bottlenecks
        report.push_str("=== PERFORMANCE BOTTLENECKS ===\n");
        let mut sorted_metrics = self.metrics.clone();
        sorted_metrics.sort_by(|a, b| b.execution_time.cmp(&a.execution_time));
        
        for (i, metric) in sorted_metrics.iter().take(5).enumerate() {
            report.push_str(&format!(
                "{}. {} - {:?} (Throughput: {:.2} ops/sec)\n",
                i + 1, metric.operation_name, metric.execution_time, metric.throughput
            ));
        }
        
        report
    }
}

#[cfg(test)]
mod comprehensive_performance_tests {
    use super::*;

    #[test]
    fn test_comprehensive_performance_analysis() {
        let mut analyzer = PerformanceAnalyzer::new();
        
        println!("Starting comprehensive performance analysis...\n");
        
        // Test 1: Basic arithmetic operations
        analyzer.run_test("arithmetic_operations", 10000, |interp| {
            interp.exec("global result = 10 + 5 * 3 - 2 / 4")
                .map_err(|e| format!("Arithmetic failed: {:?}", e))
        });
        
        // Test 2: String operations
        analyzer.run_test("string_operations", 5000, |interp| {
            interp.exec("global str_result = 'Hello' + ' ' + 'World' + '!'")
                .map_err(|e| format!("String operation failed: {:?}", e))
        });
        
        // Test 3: Array operations
        analyzer.run_test("array_operations", 1000, |interp| {
            interp.exec("global arr = [1, 2, 3, 4, 5]")
                .map_err(|e| format!("Array creation failed: {:?}", e))?;
            interp.exec("global arr_len = len(arr)")
                .map_err(|e| format!("Array operation failed: {:?}", e))
        });

        // Test 4: Variable assignment and retrieval
        analyzer.run_test("variable_operations", 10000, |interp| {
            interp.exec("global test_var = 42")
                .map_err(|e| format!("Variable assignment failed: {:?}", e))?;
            interp.exec("global retrieved = test_var")
                .map_err(|e| format!("Variable operation failed: {:?}", e))
        });
        
        // Test 5: Function calls
        analyzer.run_test("function_calls", 5000, |interp| {
            interp.exec("global func_result = len([1, 2, 3, 4, 5])")
                .map_err(|e| format!("Function call failed: {:?}", e))
        });
        
        // Test 6: Loop operations
        analyzer.run_test("loop_operations", 100, |interp| {
            let code = r#"
            global loop_result = 0
            for i in range(100) do
                global loop_result = loop_result + i
            next i
            "#;
            interp.exec(code)
                .map_err(|e| format!("Loop operation failed: {:?}", e))
        });
        
        // Test 7: Table creation and access
        analyzer.run_test("table_operations", 100, |interp| {
            let code = r#"
            global data = [[1, 'Alice'], [2, 'Bob'], [3, 'Charlie']]
            global headers = ['id', 'name']
            global table = table_create(data, headers)
            global names = table['name']
            "#;
            interp.exec(code)
                .map_err(|e| format!("Table operation failed: {:?}", e))
        });
        
        // Test 8: Complex expression parsing
        analyzer.run_test("complex_parsing", 1000, |interp| {
            let code = "global complex = (10 + 5) * (3 - 2) / (4 + 1) - (2 * 3)";
            interp.exec(code)
                .map_err(|e| format!("Complex parsing failed: {:?}", e))
        });
        
        // Generate and print report
        let report = analyzer.generate_report();
        println!("{}", report);
        
        // Verify all tests completed
        assert_eq!(analyzer.metrics.len(), 8, "Not all performance tests completed");
        
        // Check that no operation took excessively long
        for metric in &analyzer.metrics {
            assert!(
                metric.execution_time < Duration::from_secs(30),
                "Operation {} took too long: {:?}",
                metric.operation_name,
                metric.execution_time
            );
        }
    }

    #[test]
    fn test_large_dataset_bottleneck_analysis() {
        let mut analyzer = PerformanceAnalyzer::new();
        
        println!("=== LARGE DATASET BOTTLENECK ANALYSIS ===\n");
        
        // Test dataset creation performance
        analyzer.run_test("large_dataset_creation", 1, |interp| {
            let code = r#"
            global large_data = []
            for i in range(1000) do
                global category = i - (i / 100) * 100
                global row = [i, 'Item_' + i, i * 1.5, category]
                global large_data = push(large_data, row)
            next i
            global headers = ['id', 'name', 'value', 'category']
            global large_table = table_create(large_data, headers)
            "#;
            interp.exec(code)
                .map_err(|e| format!("Large dataset creation failed: {:?}", e))
        });
        
        // Test column access performance
        analyzer.run_test("column_access_performance", 100, |interp| {
            interp.exec("global col_data = large_table['name']")
                .map_err(|e| format!("Column access failed: {:?}", e))
        });
        
        // Test table slicing performance
        analyzer.run_test("table_slicing_performance", 100, |interp| {
            interp.exec("global head_data = table_head(large_table, 1000)")
                .map_err(|e| format!("Table slicing failed: {:?}", e))
        });
        
        let report = analyzer.generate_report();
        println!("{}", report);
        
        // Analyze bottlenecks
        let slowest_operation = analyzer.metrics.iter()
            .max_by_key(|m| m.execution_time)
            .unwrap();
        
        println!("Slowest operation identified: {} ({:?})", 
                 slowest_operation.operation_name, 
                 slowest_operation.execution_time);
        
        assert!(analyzer.metrics.len() == 3, "Dataset bottleneck analysis incomplete");
    }
}
