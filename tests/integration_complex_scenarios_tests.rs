use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;
use std::collections::HashMap;

/// Интеграционные тесты и сложные сценарии для DataCode
/// Эти тесты проверяют взаимодействие различных компонентов системы:
/// - Комбинирование функций, циклов, исключений и таблиц
/// - Сложные алгоритмы обработки данных
/// - Реальные сценарии использования
/// - Интеграция всех возможностей языка

#[cfg(test)]
mod integration_complex_scenarios_tests {
    use super::*;

    /// Тест сложного алгоритма сортировки с обработкой ошибок
    #[test]
    fn test_complex_sorting_algorithm_with_error_handling() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        # Реализация пузырьковой сортировки с обработкой ошибок
        global function bubble_sort(arr) do
            try
                if not isinstance(arr, 'array') do
                    throw 'Input must be an array'
                endif
                
                global n = len(arr)
                global sorted_arr = arr
                global comparisons = 0
                global swaps = 0
                
                for i in range(n) do
                    for j in range(n - i - 1) do
                        global comparisons = comparisons + 1
                        
                        try
                            if sorted_arr[j] > sorted_arr[j + 1] do
                                # Swap elements
                                global temp = sorted_arr[j]
                                global sorted_arr[j] = sorted_arr[j + 1]
                                global sorted_arr[j + 1] = temp
                                global swaps = swaps + 1
                            endif
                        catch swap_error
                            throw 'Error during swap at position ' + j + ': ' + swap_error
                        endtry
                    forend
                forend
                
                global result = {}
                global result['sorted'] = sorted_arr
                global result['comparisons'] = comparisons
                global result['swaps'] = swaps
                return result
                
            catch sort_error
                global error_result = {}
                global error_result['error'] = sort_error
                global error_result['sorted'] = null
                return error_result
            endtry
        endfunction
        
        # Тестируем сортировку
        global test_array = [64, 34, 25, 12, 22, 11, 90]
        global sort_result = bubble_sort(test_array)
        
        global sorted_array = sort_result['sorted']
        global total_comparisons = sort_result['comparisons']
        global total_swaps = sort_result['swaps']
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Complex sorting algorithm should succeed");
        
        // Проверяем отсортированный массив
        let sorted_array = interp.get_variable("sorted_array").unwrap();
        if let Value::Array(arr) = sorted_array {
            assert_eq!(arr.len(), 7);
            assert_eq!(arr[0], Value::Number(11.0));
            assert_eq!(arr[1], Value::Number(12.0));
            assert_eq!(arr[2], Value::Number(22.0));
            assert_eq!(arr[3], Value::Number(25.0));
            assert_eq!(arr[4], Value::Number(34.0));
            assert_eq!(arr[5], Value::Number(64.0));
            assert_eq!(arr[6], Value::Number(90.0));
        } else {
            panic!("sorted_array should be an array");
        }
        
        // Проверяем статистику
        let total_comparisons = interp.get_variable("total_comparisons").unwrap();
        let total_swaps = interp.get_variable("total_swaps").unwrap();
        
        if let (Value::Number(comp), Value::Number(swaps)) = (total_comparisons, total_swaps) {
            assert!(*comp > 0.0); // Должны были быть сравнения
            assert!(*swaps > 0.0); // Должны были быть перестановки
            println!("Sorting stats: {} comparisons, {} swaps", comp, swaps);
        } else {
            panic!("Statistics should be numbers");
        }
    }

    /// Тест сложной обработки данных с таблицами и фильтрацией
    #[test]
    fn test_complex_data_processing_with_tables() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        # Создаем таблицу с данными о продажах
        global sales_data = [
            ['Product', 'Category', 'Price', 'Quantity', 'Date'],
            ['Laptop', 'Electronics', 999.99, 5, '2023-01-15'],
            ['Mouse', 'Electronics', 25.50, 20, '2023-01-16'],
            ['Desk', 'Furniture', 299.00, 3, '2023-01-17'],
            ['Chair', 'Furniture', 150.00, 8, '2023-01-18'],
            ['Phone', 'Electronics', 699.99, 12, '2023-01-19'],
            ['Table', 'Furniture', 450.00, 2, '2023-01-20']
        ]
        
        global sales_table = table_create(sales_data)
        
        # Функция для анализа продаж
        global function analyze_sales(table) do
            try
                global products = table['Product']
                global categories = table['Category']
                global prices = table['Price']
                global quantities = table['Quantity']
                
                global analysis = {}
                global analysis['total_revenue'] = 0
                global analysis['category_stats'] = {}
                global analysis['top_products'] = []
                
                # Вычисляем общую выручку и статистику по категориям
                for i in range(len(products)) do
                    global revenue = prices[i] * quantities[i]
                    global analysis['total_revenue'] = analysis['total_revenue'] + revenue
                    
                    global category = categories[i]
                    if not isset(analysis['category_stats'][category]) do
                        global analysis['category_stats'][category] = {}
                        global analysis['category_stats'][category]['revenue'] = 0
                        global analysis['category_stats'][category]['count'] = 0
                    endif
                    
                    global analysis['category_stats'][category]['revenue'] = 
                        analysis['category_stats'][category]['revenue'] + revenue
                    global analysis['category_stats'][category]['count'] = 
                        analysis['category_stats'][category]['count'] + 1
                forend
                
                # Находим продукты с выручкой больше 1000
                for i in range(len(products)) do
                    global revenue = prices[i] * quantities[i]
                    if revenue > 1000 do
                        global product_info = {}
                        global product_info['name'] = products[i]
                        global product_info['revenue'] = revenue
                        global analysis['top_products'] = push(analysis['top_products'], product_info)
                    endif
                forend
                
                return analysis
                
            catch analysis_error
                throw 'Analysis failed: ' + analysis_error
            endtry
        endfunction
        
        global sales_analysis = analyze_sales(sales_table)
        global total_revenue = sales_analysis['total_revenue']
        global electronics_revenue = sales_analysis['category_stats']['Electronics']['revenue']
        global furniture_count = sales_analysis['category_stats']['Furniture']['count']
        global top_products_count = len(sales_analysis['top_products'])
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Complex data processing should succeed");
        
        // Проверяем общую выручку
        let total_revenue = interp.get_variable("total_revenue").unwrap();
        if let Value::Number(revenue) = total_revenue {
            // Laptop: 999.99 * 5 = 4999.95
            // Mouse: 25.50 * 20 = 510.00
            // Desk: 299.00 * 3 = 897.00
            // Chair: 150.00 * 8 = 1200.00
            // Phone: 699.99 * 12 = 8399.88
            // Table: 450.00 * 2 = 900.00
            // Total: 16906.83
            assert!((*revenue - 16906.83).abs() < 0.01);
        } else {
            panic!("total_revenue should be a number");
        }
        
        // Проверяем выручку по электронике
        let electronics_revenue = interp.get_variable("electronics_revenue").unwrap();
        if let Value::Number(revenue) = electronics_revenue {
            // Laptop + Mouse + Phone = 4999.95 + 510.00 + 8399.88 = 13909.83
            assert!((*revenue - 13909.83).abs() < 0.01);
        } else {
            panic!("electronics_revenue should be a number");
        }
        
        // Проверяем количество мебели
        assert_eq!(interp.get_variable("furniture_count"), Some(&Value::Number(3.0)));
        
        // Проверяем количество топ-продуктов (с выручкой > 1000)
        assert_eq!(interp.get_variable("top_products_count"), Some(&Value::Number(3.0))); // Laptop, Chair, Phone
    }

    /// Тест сложного алгоритма с рекурсией и мемоизацией
    #[test]
    fn test_complex_recursive_algorithm_with_memoization() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        # Реализация чисел Фибоначчи с мемоизацией
        global memo = {}
        global calculation_count = 0
        
        global function fibonacci_memo(n) do
            global calculation_count = calculation_count + 1
            
            try
                if n < 0 do
                    throw 'Fibonacci not defined for negative numbers'
                endif
                
                if n <= 1 do
                    return n
                endif
                
                # Проверяем мемо
                if isset(memo[n]) do
                    return memo[n]
                endif
                
                # Вычисляем и сохраняем в мемо
                global result = fibonacci_memo(n - 1) + fibonacci_memo(n - 2)
                global memo[n] = result
                return result
                
            catch fib_error
                throw 'Fibonacci calculation error for n=' + n + ': ' + fib_error
            endtry
        endfunction
        
        # Функция для вычисления последовательности Фибоначчи
        global function fibonacci_sequence(count) do
            global sequence = []
            global stats = {}
            global stats['calculations'] = 0
            
            try
                for i in range(count) do
                    global calculation_count = 0
                    global fib_value = fibonacci_memo(i)
                    global sequence = push(sequence, fib_value)
                    global stats['calculations'] = stats['calculations'] + calculation_count
                forend
                
                global result = {}
                global result['sequence'] = sequence
                global result['stats'] = stats
                global result['memo_size'] = len(memo)
                return result
                
            catch seq_error
                throw 'Sequence generation failed: ' + seq_error
            endtry
        endfunction
        
        # Генерируем последовательность
        global fib_result = fibonacci_sequence(15)
        global fib_sequence = fib_result['sequence']
        global total_calculations = fib_result['stats']['calculations']
        global memo_size = fib_result['memo_size']
        
        # Проверяем конкретные значения
        global fib_10 = fib_sequence[10]
        global fib_14 = fib_sequence[14]
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Complex recursive algorithm should succeed");
        
        // Проверяем последовательность Фибоначчи
        let fib_sequence = interp.get_variable("fib_sequence").unwrap();
        if let Value::Array(seq) = fib_sequence {
            assert_eq!(seq.len(), 15);
            assert_eq!(seq[0], Value::Number(0.0)); // F(0) = 0
            assert_eq!(seq[1], Value::Number(1.0)); // F(1) = 1
            assert_eq!(seq[2], Value::Number(1.0)); // F(2) = 1
            assert_eq!(seq[3], Value::Number(2.0)); // F(3) = 2
            assert_eq!(seq[4], Value::Number(3.0)); // F(4) = 3
            assert_eq!(seq[5], Value::Number(5.0)); // F(5) = 5
        } else {
            panic!("fib_sequence should be an array");
        }
        
        // Проверяем конкретные значения
        assert_eq!(interp.get_variable("fib_10"), Some(&Value::Number(55.0))); // F(10) = 55
        assert_eq!(interp.get_variable("fib_14"), Some(&Value::Number(377.0))); // F(14) = 377
        
        // Проверяем эффективность мемоизации
        let total_calculations = interp.get_variable("total_calculations").unwrap();
        let memo_size = interp.get_variable("memo_size").unwrap();
        
        if let (Value::Number(calc), Value::Number(memo)) = (total_calculations, memo_size) {
            // Благодаря мемоизации количество вычислений должно быть значительно меньше
            // чем при наивной рекурсии
            assert!(*calc < 100.0); // Должно быть намного меньше без мемоизации
            assert!(*memo > 10.0); // Должны быть сохранены промежуточные результаты
            println!("Memoization efficiency: {} calculations, {} memoized values", calc, memo);
        } else {
            panic!("Statistics should be numbers");
        }
    }
}
