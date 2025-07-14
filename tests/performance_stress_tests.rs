use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;
use std::time::{Duration, Instant};

/// Стресс-тесты и тесты производительности для DataCode интерпретатора
/// Эти тесты проверяют поведение интерпретатора под нагрузкой:
/// - Обработка больших объемов данных
/// - Глубокая рекурсия и ограничения стека
/// - Множественные вложенные циклы
/// - Интенсивные операции с памятью
/// - Производительность различных операций

#[cfg(test)]
mod performance_stress_tests {
    use super::*;

    /// Тест производительности арифметических операций
    #[test]
    fn test_arithmetic_performance() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global start_time = now()
        global result = 0
        
        for i in range(10000) do
            global result = result + i * 2 - 1
        forend
        
        global end_time = now()
        global final_result = result
        "#;
        
        let start = Instant::now();
        let result = interp.exec(code);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Arithmetic performance test should succeed");
        println!("Arithmetic operations (10,000 iterations) took: {:?}", duration);
        
        // Проверяем результат
        let final_result = interp.get_variable("final_result").unwrap();
        if let Value::Number(n) = final_result {
            // Ожидаемый результат: сумма (i * 2 - 1) для i от 0 до 9999
            // = сумма (2i - 1) = 2 * сумма(i) - 10000 = 2 * (9999 * 10000 / 2) - 10000 = 99980000
            assert_eq!(*n, 99980000.0);
        } else {
            panic!("final_result should be a number");
        }
        
        // Проверяем, что операция выполнилась за разумное время (менее 5 секунд)
        assert!(duration < Duration::from_secs(5), "Arithmetic operations took too long: {:?}", duration);
    }

    /// Тест производительности строковых операций
    #[test]
    fn test_string_performance() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global result = ''
        global separator = '_'
        
        for i in range(1000) do
            global result = result + 'item' + i + separator
        forend
        
        global final_length = len(result)
        "#;
        
        let start = Instant::now();
        let result = interp.exec(code);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "String performance test should succeed");
        println!("String operations (1,000 iterations) took: {:?}", duration);
        
        // Проверяем длину результата
        let final_length = interp.get_variable("final_length").unwrap();
        if let Value::Number(n) = final_length {
            // Каждая итерация добавляет примерно 'item' + число + '_' = минимум 6 символов
            assert!(*n > 6000.0);
        } else {
            panic!("final_length should be a number");
        }
        
        // Проверяем, что операция выполнилась за разумное время
        assert!(duration < Duration::from_secs(10), "String operations took too long: {:?}", duration);
    }

    /// Тест производительности операций с массивами
    #[test]
    fn test_array_performance() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global large_array = []
        
        # Создаем большой массив
        for i in range(5000) do
            global large_array = push(large_array, i)
        forend
        
        # Выполняем операции с массивом
        global sum = 0
        for item in large_array do
            global sum = sum + item
        forend
        
        global array_size = len(large_array)
        "#;
        
        let start = Instant::now();
        let result = interp.exec(code);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Array performance test should succeed");
        println!("Array operations (5,000 elements) took: {:?}", duration);
        
        // Проверяем размер массива
        assert_eq!(interp.get_variable("array_size"), Some(&Value::Number(5000.0)));
        
        // Проверяем сумму (0 + 1 + 2 + ... + 4999 = 4999 * 5000 / 2 = 12497500)
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(12497500.0)));
        
        // Проверяем производительность
        assert!(duration < Duration::from_secs(15), "Array operations took too long: {:?}", duration);
    }

    /// Стресс-тест с глубокой рекурсией
    #[test]
    fn test_deep_recursion_stress() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global function fibonacci(n) do
            if n <= 1 do
                return n
            endif
            return fibonacci(n - 1) + fibonacci(n - 2)
        endfunction
        
        # Вычисляем небольшое число Фибоначчи для проверки рекурсии
        global fib_result = fibonacci(20)
        "#;
        
        let start = Instant::now();
        let result = interp.exec(code);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Deep recursion test should succeed");
        println!("Fibonacci(20) calculation took: {:?}", duration);
        
        // Проверяем результат (20-е число Фибоначчи = 6765)
        assert_eq!(interp.get_variable("fib_result"), Some(&Value::Number(6765.0)));
        
        // Проверяем, что рекурсия не привела к переполнению стека
        assert_eq!(interp.recursion_depth, 0);
        
        // Проверяем производительность (рекурсивный Фибоначчи медленный, но должен завершиться)
        assert!(duration < Duration::from_secs(30), "Fibonacci calculation took too long: {:?}", duration);
    }

    /// Стресс-тест с множественными вложенными циклами
    #[test]
    fn test_nested_loops_stress() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global total_iterations = 0
        global result_matrix = []
        
        for i in range(50) do
            global row = []
            for j in range(50) do
                global value = i * j
                global row = push(row, value)
                global total_iterations = total_iterations + 1
            forend
            global result_matrix = push(result_matrix, row)
        forend
        
        global matrix_size = len(result_matrix)
        global first_row_size = len(result_matrix[0])
        global corner_value = result_matrix[49][49]
        "#;
        
        let start = Instant::now();
        let result = interp.exec(code);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Nested loops stress test should succeed");
        println!("Nested loops (50x50 matrix) took: {:?}", duration);
        
        // Проверяем результаты
        assert_eq!(interp.get_variable("total_iterations"), Some(&Value::Number(2500.0)));
        assert_eq!(interp.get_variable("matrix_size"), Some(&Value::Number(50.0)));
        assert_eq!(interp.get_variable("first_row_size"), Some(&Value::Number(50.0)));
        assert_eq!(interp.get_variable("corner_value"), Some(&Value::Number(2401.0))); // 49 * 49
        
        // Проверяем производительность
        assert!(duration < Duration::from_secs(20), "Nested loops took too long: {:?}", duration);
        
        // Проверяем состояние стеков
        assert_eq!(interp.variable_manager.loop_depth(), 0);
        assert_eq!(interp.variable_manager.function_depth(), 0);
    }

    /// Стресс-тест обработки исключений
    #[test]
    fn test_exception_handling_stress() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global exceptions_caught = 0
        global successful_operations = 0
        
        for i in range(1000) do
            try
                if i % 10 == 0 do
                    throw 'Error at iteration ' + i
                endif
                global successful_operations = successful_operations + 1
            catch error
                global exceptions_caught = exceptions_caught + 1
            endtry
        forend
        
        global total_processed = exceptions_caught + successful_operations
        "#;
        
        let start = Instant::now();
        let result = interp.exec(code);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Exception handling stress test should succeed");
        println!("Exception handling (1,000 iterations) took: {:?}", duration);
        
        // Проверяем результаты
        assert_eq!(interp.get_variable("exceptions_caught"), Some(&Value::Number(100.0))); // Каждый 10-й
        assert_eq!(interp.get_variable("successful_operations"), Some(&Value::Number(900.0))); // Остальные
        assert_eq!(interp.get_variable("total_processed"), Some(&Value::Number(1000.0)));
        
        // Проверяем, что стек исключений очищен
        assert_eq!(interp.exception_stack.len(), 0);
        
        // Проверяем производительность
        assert!(duration < Duration::from_secs(10), "Exception handling took too long: {:?}", duration);
    }

    /// Стресс-тест управления памятью
    #[test]
    fn test_memory_management_stress() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global large_structures = []
        
        # Создаем много больших структур данных
        for i in range(100) do
            global structure = {}
            global structure['id'] = i
            global structure['data'] = []
            
            for j in range(100) do
                global item = {}
                global item['index'] = j
                global item['value'] = i * j
                global item['text'] = 'item_' + i + '_' + j
                global structure['data'] = push(structure['data'], item)
            forend
            
            global large_structures = push(large_structures, structure)
        forend
        
        global structures_count = len(large_structures)
        global first_structure_data_count = len(large_structures[0]['data'])
        global last_item_value = large_structures[99]['data'][99]['value']
        "#;
        
        let start = Instant::now();
        let result = interp.exec(code);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Memory management stress test should succeed");
        println!("Memory management (100x100 structures) took: {:?}", duration);
        
        // Проверяем результаты
        assert_eq!(interp.get_variable("structures_count"), Some(&Value::Number(100.0)));
        assert_eq!(interp.get_variable("first_structure_data_count"), Some(&Value::Number(100.0)));
        assert_eq!(interp.get_variable("last_item_value"), Some(&Value::Number(9801.0))); // 99 * 99
        
        // Проверяем производительность
        assert!(duration < Duration::from_secs(30), "Memory management took too long: {:?}", duration);
        
        // Проверяем, что все переменные все еще доступны
        let all_vars = interp.get_all_variables();
        assert!(all_vars.contains_key("large_structures"));
        assert!(all_vars.len() > 5); // Должно быть много переменных
    }
}
