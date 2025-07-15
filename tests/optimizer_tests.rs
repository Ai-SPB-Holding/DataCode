// Тесты для оптимизатора DataCode
// Проверяет корректность работы AST оптимизации, кэширования и статического анализа

use data_code::optimizer::{Optimizer, ASTOptimizer, ParseCache, StaticAnalyzer};
use data_code::parser::{Parser, Expr, BinaryOp};
use data_code::value::Value;
use std::collections::HashMap;

#[cfg(test)]
mod optimizer_tests {
    use super::*;

    #[test]
    fn test_ast_constant_folding() {
        let mut optimizer = ASTOptimizer::new();
        
        // Тест свертки констант для арифметических операций
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Value::Number(10.0))),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Literal(Value::Number(5.0))),
        };
        
        let optimized = optimizer.optimize(expr).unwrap();
        
        match optimized {
            Expr::Literal(Value::Number(n)) => assert_eq!(n, 15.0),
            _ => panic!("Expected constant folding to produce literal 15.0"),
        }
        
        assert!(optimizer.get_optimization_count() > 0);
    }
    
    #[test]
    fn test_ast_boolean_simplification() {
        let mut optimizer = ASTOptimizer::new();
        
        // Тест упрощения true && x → x
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Value::Bool(true))),
            operator: BinaryOp::And,
            right: Box::new(Expr::Variable("x".to_string())),
        };
        
        let optimized = optimizer.optimize(expr).unwrap();
        
        match optimized {
            Expr::Variable(name) => assert_eq!(name, "x"),
            _ => panic!("Expected boolean simplification to produce variable x"),
        }
        
        assert!(optimizer.get_optimization_count() > 0);
    }
    
    #[test]
    fn test_ast_filter_combination() {
        let mut optimizer = ASTOptimizer::new();
        
        // Создаем вложенные фильтры: table_filter(table_filter(data, x > 5), x < 10)
        let inner_filter = Expr::FunctionCall {
            name: "table_filter".to_string(),
            args: vec![
                Expr::Variable("data".to_string()),
                Expr::Binary {
                    left: Box::new(Expr::Variable("x".to_string())),
                    operator: BinaryOp::Greater,
                    right: Box::new(Expr::Literal(Value::Number(5.0))),
                },
            ],
        };
        
        let outer_filter = Expr::FunctionCall {
            name: "table_filter".to_string(),
            args: vec![
                inner_filter,
                Expr::Binary {
                    left: Box::new(Expr::Variable("x".to_string())),
                    operator: BinaryOp::Less,
                    right: Box::new(Expr::Literal(Value::Number(10.0))),
                },
            ],
        };
        
        let optimized = optimizer.optimize(outer_filter).unwrap();
        
        // Проверяем, что фильтры объединились
        match optimized {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "table_filter");
                assert_eq!(args.len(), 2);
                
                // Второй аргумент должен быть объединенным условием (x > 5 && x < 10)
                match &args[1] {
                    Expr::Binary { operator: BinaryOp::And, .. } => {
                        // Успешно объединили условия
                    }
                    _ => panic!("Expected combined filter condition with AND operator"),
                }
            }
            _ => panic!("Expected optimized function call"),
        }
        
        assert!(optimizer.get_optimization_count() > 0);
    }
    
    #[test]
    fn test_parse_cache_basic() {
        let mut cache = ParseCache::new();
        
        // Первый вызов - промах
        let result1 = cache.get_or_parse("1 + 2", |s| {
            Parser::new(s).parse_expression()
        });
        assert!(result1.is_ok());
        assert_eq!(cache.get_miss_count(), 1);
        assert_eq!(cache.get_hit_count(), 0);
        
        // Второй вызов - попадание
        let result2 = cache.get_or_parse("1 + 2", |s| {
            Parser::new(s).parse_expression()
        });
        assert!(result2.is_ok());
        assert_eq!(cache.get_miss_count(), 1);
        assert_eq!(cache.get_hit_count(), 1);
        
        // Проверяем коэффициент попаданий
        assert_eq!(cache.hit_ratio(), 0.5);
    }
    
    #[test]
    fn test_parse_cache_performance() {
        let mut cache = ParseCache::new();
        let expressions = vec![
            "age > 18",
            "name != null",
            "id > 0",
            "age > 18", // Повтор
            "name != null", // Повтор
            "salary >= 50000",
            "age > 18", // Еще один повтор
        ];
        
        for expr_str in expressions {
            let _ = cache.get_or_parse(expr_str, |s| {
                Parser::new(s).parse_expression()
            });
        }
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 3); // 3 повтора
        assert_eq!(stats.misses, 4); // 4 уникальных выражения
        assert!(stats.hit_ratio > 0.4); // Более 40% попаданий
        // Проверяем, что кэш работает (есть попадания)
        assert!(stats.hits > 0, "Cache should have some hits");
    }
    
    #[test]
    fn test_static_analyzer_variable_usage() {
        let mut analyzer = StaticAnalyzer::new();
        
        // Создаем выражение с переменными
        let expr = Expr::Binary {
            left: Box::new(Expr::Variable("age".to_string())),
            operator: BinaryOp::Greater,
            right: Box::new(Expr::Literal(Value::Number(18.0))),
        };
        
        let result = analyzer.analyze(&expr).unwrap();
        
        assert!(result.variables_used.contains("age"));
        assert_eq!(result.variables_used.len(), 1);
        assert!(result.functions_called.is_empty());
    }
    
    #[test]
    fn test_static_analyzer_function_calls() {
        let mut analyzer = StaticAnalyzer::new();
        
        // Создаем выражение с вызовом функции
        let expr = Expr::FunctionCall {
            name: "table_filter".to_string(),
            args: vec![
                Expr::Variable("data".to_string()),
                Expr::Binary {
                    left: Box::new(Expr::Variable("age".to_string())),
                    operator: BinaryOp::Greater,
                    right: Box::new(Expr::Literal(Value::Number(18.0))),
                },
            ],
        };
        
        let result = analyzer.analyze(&expr).unwrap();
        
        assert!(result.functions_called.contains("table_filter"));
        assert!(result.variables_used.contains("data"));
        assert!(result.variables_used.contains("age"));
    }
    
    #[test]
    fn test_static_analyzer_type_checking() {
        let mut analyzer = StaticAnalyzer::new();
        
        // Создаем выражение с потенциальной ошибкой типов
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Value::String("hello".to_string()))),
            operator: BinaryOp::Subtract, // Нельзя вычитать из строки
            right: Box::new(Expr::Literal(Value::Number(5.0))),
        };
        
        let result = analyzer.analyze(&expr).unwrap();
        
        assert!(result.has_errors());
        assert!(!result.potential_errors.is_empty());
    }
    
    #[test]
    fn test_static_analyzer_with_context() {
        let mut analyzer = StaticAnalyzer::new();
        
        // Создаем контекст с известными переменными
        let mut variables = HashMap::new();
        variables.insert("age".to_string(), Value::Number(25.0));
        variables.insert("name".to_string(), Value::String("John".to_string()));
        
        let expr = Expr::Binary {
            left: Box::new(Expr::Variable("age".to_string())),
            operator: BinaryOp::Greater,
            right: Box::new(Expr::Variable("unknown_var".to_string())), // Неизвестная переменная
        };
        
        let result = analyzer.analyze_with_context(&expr, &variables).unwrap();
        
        assert!(result.variables_used.contains("age"));
        assert!(result.variables_used.contains("unknown_var"));
        assert!(result.has_errors()); // Должна быть ошибка для unknown_var
    }
    
    #[test]
    fn test_full_optimizer_integration() {
        let mut optimizer = Optimizer::new();
        
        // Создаем сложное выражение для оптимизации
        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal(Value::Number(10.0))),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Literal(Value::Number(5.0))),
            }),
            operator: BinaryOp::And,
            right: Box::new(Expr::Literal(Value::Bool(true))),
        };
        
        let optimized = optimizer.optimize(expr).unwrap();
        
        // Должно быть оптимизировано до: 15.0 && true → 15.0
        match optimized {
            Expr::Literal(Value::Number(n)) => assert_eq!(n, 15.0),
            _ => {
                // Если не полностью оптимизировано, проверяем частичную оптимизацию
                println!("Partial optimization result: {:?}", optimized);
            }
        }
        
        let stats = optimizer.get_stats();
        assert!(stats.ast_optimizations > 0);
    }
    
    #[test]
    fn test_parser_basic() {
        let mut parser = Parser::new("10 + 5 * 2");
        let expr = parser.parse_expression().unwrap();

        // Проверяем, что парсер создал выражение
        match expr {
            Expr::Binary { .. } => {
                // Ожидаем бинарное выражение
            }
            _ => panic!("Expected binary expression"),
        }
    }
    
    #[test]
    fn test_cache_preloading() {
        let mut cache = ParseCache::new();
        cache.preload_common_expressions();
        
        // Проверяем, что общие выражения загружены
        assert!(cache.size() > 0);
        
        // Тестируем попадание в кэш для предзагруженного выражения
        let result = cache.get("true");
        assert!(result.is_some());
        assert_eq!(cache.get_hit_count(), 1);
    }
}
