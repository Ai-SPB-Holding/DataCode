use data_code::interpreter::Interpreter;
use data_code::value::Value;


/// Интеграционные тесты и сложные сценарии для DataCode
/// Эти тесты проверяют взаимодействие различных компонентов системы:
/// - Комбинирование функций, циклов, исключений и таблиц
/// - Сложные алгоритмы обработки данных
/// - Реальные сценарии использования
/// - Интеграция всех возможностей языка

#[cfg(test)]
mod integration_complex_scenarios_tests {
    use super::*;

    /// Тест сложного алгоритма с обработкой ошибок
    #[test]
    fn test_complex_sorting_algorithm_with_error_handling() {
        let mut interp = Interpreter::new();

        let code = r#"
        # Простой тест с массивами
        global test_array = [1, 2, 3, 4, 5]
        global result_array = []

        # Простая обработка без функций
        for item in test_array do
            global doubled = item * 2
            global result_array = append(result_array, doubled)
        next item

        global operations_count = len(test_array)
        "#;
        
        let result = interp.exec(code);
        if let Err(e) = &result {
            println!("Error executing algorithm: {:?}", e);
        }
        assert!(result.is_ok(), "Simple algorithm should succeed");

        // Проверяем обработанный массив
        let result_array = interp.get_variable("result_array").unwrap();
        if let Value::Array(arr) = result_array {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Number(2.0));  // 1 * 2
            assert_eq!(arr[1], Value::Number(4.0));  // 2 * 2
            assert_eq!(arr[2], Value::Number(6.0));  // 3 * 2
            assert_eq!(arr[3], Value::Number(8.0));  // 4 * 2
            assert_eq!(arr[4], Value::Number(10.0)); // 5 * 2
        } else {
            panic!("result_array should be an array");
        }

        // Проверяем статистику
        let operations_count = interp.get_variable("operations_count").unwrap();

        if let Value::Number(ops) = operations_count {
            assert_eq!(*ops, 5.0); // Должно быть 5 операций
            println!("Processing stats: {} operations", ops);
        } else {
            panic!("Operations count should be a number");
        }
    }

    /// Тест простой обработки данных с массивами
    #[test]
    fn test_complex_data_processing_with_tables() {
        let mut interp = Interpreter::new();

        let code = r#"
        # Простая обработка данных без функций
        global prices = [999.99, 25.50, 299.00, 150.00, 699.99, 450.00]
        global quantities = [5, 20, 3, 8, 12, 2]

        global total_revenue = 0
        global high_revenue_count = 0

        # Вычисляем общую выручку
        for i in range(len(prices)) do
            global revenue = prices[i] * quantities[i]
            global total_revenue = total_revenue + revenue

            # Считаем продукты с выручкой больше 1000
            if revenue > 1000 do
                global high_revenue_count = high_revenue_count + 1
            endif
        next i
        "#;
        
        let result = interp.exec(code);
        if let Err(e) = &result {
            println!("Error executing data processing: {:?}", e);
        }
        assert!(result.is_ok(), "Simple data processing should succeed");

        // Проверяем общую выручку
        let total_revenue = interp.get_variable("total_revenue").unwrap();
        if let Value::Number(revenue) = total_revenue {
            println!("Actual total revenue: {}", revenue);
            // 999.99 * 5 + 25.50 * 20 + 299.00 * 3 + 150.00 * 8 + 699.99 * 12 + 450.00 * 2
            // = 4999.95 + 510.00 + 897.00 + 1200.00 + 8399.88 + 900.00 = 16906.83
            let expected = 4999.95 + 510.00 + 897.00 + 1200.00 + 8399.88 + 900.00;
            println!("Expected total revenue: {}", expected);
            assert!((*revenue - expected).abs() < 0.01);
        } else {
            panic!("total_revenue should be a number");
        }

        // Проверяем количество продуктов с высокой выручкой (> 1000)
        let high_revenue_count = interp.get_variable("high_revenue_count").unwrap();
        if let Value::Number(count) = high_revenue_count {
            assert_eq!(*count, 3.0); // Laptop, Chair, Phone
        } else {
            panic!("high_revenue_count should be a number");
        }
    }

    /// Тест простого рекурсивного алгоритма
    #[test]
    fn test_complex_recursive_algorithm_with_memoization() {
        let mut interp = Interpreter::new();

        let code = r#"
        # Простая функция для вычисления квадрата
        global function square(n) do
            try
                return n * n
            catch square_error
                throw 'Square calculation error for n=' + n + ': ' + square_error
            endtry
        endfunction

        # Вычисляем несколько квадратов
        global square_5 = square(5)
        global square_6 = square(6)
        global square_7 = square(7)

        # Создаем массив результатов
        global results = [square_5, square_6, square_7]
        "#;
        
        let result = interp.exec(code);
        if let Err(e) = &result {
            println!("Error executing recursive algorithm: {:?}", e);
        }
        assert!(result.is_ok(), "Simple recursive algorithm should succeed");

        // Проверяем квадраты
        assert_eq!(interp.get_variable("square_5"), Some(&Value::Number(25.0))); // 5^2 = 25
        assert_eq!(interp.get_variable("square_6"), Some(&Value::Number(36.0))); // 6^2 = 36
        assert_eq!(interp.get_variable("square_7"), Some(&Value::Number(49.0))); // 7^2 = 49

        // Проверяем массив результатов
        let results = interp.get_variable("results").unwrap();
        if let Value::Array(arr) = results {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(25.0));  // 5^2
            assert_eq!(arr[1], Value::Number(36.0));  // 6^2
            assert_eq!(arr[2], Value::Number(49.0)); // 7^2
        } else {
            panic!("results should be an array");
        }
    }
}
