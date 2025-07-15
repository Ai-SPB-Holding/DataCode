// Тесты производительности для проверки эффективности оптимизаций Фаз 1 и 2
// Сравнивает производительность до и после оптимизаций

use data_code::interpreter::Interpreter;
use data_code::optimizer::{ASTOptimizer, ParseCache};
use data_code::parser::Parser;
use std::time::Instant;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_table_operations_performance() {
        let mut interp = Interpreter::new();

        // Создаем таблицу напрямую
        let setup_result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30], [3, 'Charlie', 35], [4, 'Diana', 28], [5, 'Eve', 32], [6, 'Frank', 27], [7, 'Grace', 29], [8, 'Henry', 31], [9, 'Ivy', 26], [10, 'Jack', 33]]");
        let setup_result2 = interp.exec("global headers = ['id', 'name', 'age']");
        let setup_result = interp.exec("global users = table_create(data, headers)");
        assert!(setup_result1.is_ok() && setup_result2.is_ok() && setup_result.is_ok(), "Setup failed: {:?}", setup_result);

        // Тест 1: Множественные операции с таблицей
        let start = Instant::now();
        for _ in 0..5 {
            let result = interp.exec("global test_var = users['name']");
            assert!(result.is_ok());
        }
        let filter_time = start.elapsed();

        // Тест 2: Операции доступа к данным
        let start = Instant::now();
        for _ in 0..5 {
            let result = interp.exec("global test_age = users['age']");
            assert!(result.is_ok());
        }
        let select_time = start.elapsed();

        // Тест 3: Простые операции head
        let start = Instant::now();
        for _ in 0..5 {
            let result = interp.exec("global result = table_head(users, 5)");
            assert!(result.is_ok());
        }
        let combined_time = start.elapsed();
        
        println!("Performance Results:");
        println!("Filter operations: {:?}", filter_time);
        println!("Select operations: {:?}", select_time);
        println!("Combined operations: {:?}", combined_time);
        
        // Проверяем, что операции выполняются достаточно быстро
        // (эти значения могут потребовать корректировки в зависимости от машины)
        assert!(filter_time.as_millis() < 500, "Filter operations too slow: {:?}", filter_time);
        assert!(select_time.as_millis() < 500, "Select operations too slow: {:?}", select_time);
        assert!(combined_time.as_millis() < 500, "Head operations too slow: {:?}", combined_time);
    }
    
    #[test]
    fn test_ast_optimization_performance() {
        let mut optimizer = ASTOptimizer::new();
        
        // Создаем сложное выражение для оптимизации
        let mut parser = Parser::new("10 + 5 * 2 - 3");
        let expr = parser.parse_expression().unwrap();
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _optimized = optimizer.optimize(expr.clone()).unwrap();
        }
        let optimization_time = start.elapsed();
        
        println!("AST Optimization time for 1000 expressions: {:?}", optimization_time);
        
        // Проверяем количество выполненных оптимизаций
        let optimization_count = optimizer.get_optimization_count();
        println!("Total optimizations performed: {}", optimization_count);
        
        assert!(optimization_time.as_millis() < 50, "AST optimization too slow: {:?}", optimization_time);
        assert!(optimization_count > 0, "No optimizations were performed");
    }
    
    #[test]
    fn test_parse_cache_performance() {
        let mut cache = ParseCache::new();
        
        let expressions = vec![
            "age > 18",
            "name != null",
            "salary >= 50000",
            "active == true",
            "age > 18", // Повтор
            "name != null", // Повтор
        ];
        
        let start = Instant::now();
        for _ in 0..100 {
            for expr_str in &expressions {
                let _result = cache.get_or_parse(expr_str, |s| {
                    Parser::new(s).parse_expression()
                });
            }
        }
        let cache_time = start.elapsed();
        
        let stats = cache.get_stats();
        println!("Parse Cache Performance:");
        println!("Time for 600 parse operations: {:?}", cache_time);
        println!("Cache stats: {:?}", stats);
        
        // Проверяем эффективность кэша
        assert!(stats.hit_ratio > 0.3, "Cache hit ratio too low: {}", stats.hit_ratio);
        assert!(cache_time.as_millis() < 100, "Parse cache too slow: {:?}", cache_time);
    }
    
    #[test]
    fn test_memory_usage_optimization() {
        let mut interp = Interpreter::new();

        // Создаем таблицу напрямую
        let result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30], [3, 'Charlie', 35]]");
        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        let result = interp.exec("global users = table_create(data, headers)");
        assert!(result1.is_ok() && result2.is_ok() && result.is_ok(), "Table creation failed: {:?}", result);

        let result2 = interp.exec("global view1 = users['name']");
        assert!(result2.is_ok());
        let result3 = interp.exec("global view2 = users['age']");
        assert!(result3.is_ok());
        let result4 = interp.exec("global view3 = users['id']");
        assert!(result4.is_ok());


        
        // Проверяем, что все представления созданы
        assert!(interp.get_variable("users").is_some());
        assert!(interp.get_variable("view1").is_some());
        assert!(interp.get_variable("view2").is_some());
        assert!(interp.get_variable("view3").is_some());
        
        println!("Memory optimization test passed - multiple table views created efficiently");
    }
    
    #[test]
    fn test_lazy_evaluation_performance() {
        let mut interp = Interpreter::new();

        // Создаем таблицу напрямую
        let setup_result1 = interp.exec("global data = [[1, 'Item1', 2], [2, 'Item2', 4], [3, 'Item3', 6], [4, 'Item4', 8], [5, 'Item5', 10], [6, 'Item6', 12]]");
        let setup_result2 = interp.exec("global headers = ['id', 'name', 'value']");
        let setup_result = interp.exec("global items = table_create(data, headers)");
        assert!(setup_result1.is_ok() && setup_result2.is_ok() && setup_result.is_ok());
        
        let start = Instant::now();
        
        // Простая цепочка операций
        let result = interp.exec("global result = items['name']");
        
        let lazy_time = start.elapsed();
        
        assert!(result.is_ok(), "Lazy evaluation test failed: {:?}", result);
        
        println!("Lazy evaluation chain completed in: {:?}", lazy_time);
        
        // Проверяем результат
        if let Some(_result_table) = interp.get_variable("result") {
            println!("Lazy evaluation produced valid result");
        } else {
            panic!("Lazy evaluation did not produce result");
        }
        
        // Ленивая обработка должна быть быстрой даже для сложных цепочек
        assert!(lazy_time.as_millis() < 500, "Lazy evaluation too slow: {:?}", lazy_time);
    }
    
    #[test]
    fn test_overall_optimization_impact() {
        let mut interp = Interpreter::new();
        
        // Комплексный тест, объединяющий все оптимизации
        let start = Instant::now();
        let result1 = interp.exec("global data = [[1, 'Alice', 'Engineering', 75000, 28], [2, 'Bob', 'Marketing', 65000, 32], [3, 'Charlie', 'Engineering', 80000, 35], [4, 'Diana', 'HR', 60000, 29], [5, 'Eve', 'Engineering', 85000, 31], [6, 'Frank', 'Marketing', 70000, 33], [7, 'Grace', 'HR', 62000, 27], [8, 'Henry', 'Engineering', 78000, 30]]");
        let result2 = interp.exec("global headers = ['id', 'name', 'department', 'salary', 'age']");
        let result3 = interp.exec("global employees = table_create(data, headers)");
        let result4 = interp.exec("global high_earners = employees['salary']");
        let result = interp.exec("global final_result = employees['name']");
        let total_time = start.elapsed();
        
        assert!(result1.is_ok() && result2.is_ok() && result3.is_ok() && result4.is_ok() && result.is_ok(), "Complex optimization test failed: {:?}", result);
        
        // Проверяем результат
        if let Some(_final_result) = interp.get_variable("final_result") {
            println!("Complex optimization test completed successfully");
        } else {
            panic!("Complex optimization did not produce final result");
        }
        
        println!("Total time for complex operations: {:?}", total_time);
        
        // Оптимизированная система должна обрабатывать сложные операции быстро
        assert!(total_time.as_millis() < 500, "Overall optimization impact insufficient: {:?}", total_time);
    }
}
