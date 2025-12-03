use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::{DataCodeError, VariableErrorType};


/// Сложные тесты для проверки внутренних структур Rust интерпретатора DataCode
/// Эти тесты проверяют глубокие аспекты работы интерпретатора, включая:
/// - Управление памятью и стеком вызовов
/// - Состояние внутренних переменных Rust
/// - Обработка исключений и стек исключений
/// - Рекурсивные вызовы и ограничения глубины
/// - Сложные сценарии с множественными областями видимости

#[cfg(test)]
mod advanced_rust_internals_tests {
    use super::*;

    /// Тест проверки внутреннего состояния VariableManager
    #[test]
    fn test_variable_manager_internal_state() {
        let mut interp = Interpreter::new();
        
        // Проверяем начальное состояние
        assert_eq!(interp.variable_manager.function_depth(), 0);
        assert_eq!(interp.variable_manager.loop_depth(), 0);
        assert!(!interp.variable_manager.is_in_function());
        assert!(!interp.variable_manager.is_in_loop());
        
        // Устанавливаем глобальные переменные
        interp.set_variable("global_var".to_string(), Value::Number(42.0), true);
        
        // Проверяем, что переменная попала в глобальные
        let globals = interp.get_all_variables();
        assert_eq!(globals.len(), 1);
        assert!(globals.contains_key("global_var"));
        
        // Симулируем вход в функцию
        interp.variable_manager.enter_function_scope();
        assert_eq!(interp.variable_manager.function_depth(), 1);
        assert!(interp.variable_manager.is_in_function());
        
        // Устанавливаем локальную переменную
        interp.set_variable("local_var".to_string(), Value::String("test".to_string()), false);
        
        // Проверяем, что локальная переменная видна, а глобальная тоже
        assert_eq!(interp.get_variable("local_var"), Some(&Value::String("test".to_string())));
        assert_eq!(interp.get_variable("global_var"), Some(&Value::Number(42.0)));
        
        // Симулируем вход в цикл
        interp.variable_manager.enter_loop_scope();
        assert_eq!(interp.variable_manager.loop_depth(), 1);
        assert!(interp.variable_manager.is_in_loop());
        
        // Устанавливаем переменную цикла
        interp.variable_manager.set_loop_variable("loop_var".to_string(), Value::Bool(true));
        assert_eq!(interp.get_variable("loop_var"), Some(&Value::Bool(true)));
        
        // Выходим из цикла
        interp.variable_manager.exit_loop_scope();
        assert_eq!(interp.variable_manager.loop_depth(), 0);
        assert!(!interp.variable_manager.is_in_loop());
        assert_eq!(interp.get_variable("loop_var"), None);
        
        // Выходим из функции
        interp.variable_manager.exit_function_scope();
        assert_eq!(interp.variable_manager.function_depth(), 0);
        assert!(!interp.variable_manager.is_in_function());
        assert_eq!(interp.get_variable("local_var"), None);
        assert_eq!(interp.get_variable("global_var"), Some(&Value::Number(42.0)));
    }

    /// Тест проверки стека исключений
    #[test]
    fn test_exception_stack_internal_state() {
        let mut interp = Interpreter::new();

        let code = r#"
        try
            global level1 = true
            try
                global level2 = true
                try
                    global level3 = true
                    throw 'Deep error'
                catch deep_error
                    global deep_caught = true
                    global deep_error_msg = deep_error
                endtry
                global after_inner_try = true
            catch middle_error
                global middle_caught = true
            endtry
            global after_middle_try = true
        catch outer_error
            global outer_caught = true
        endtry
        global after_outer_try = true
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");


        // Проверяем начальное состояние стека исключений
        assert_eq!(interp.exception_stack.len(), 0);
        assert_eq!(interp.get_try_nesting_level(), 0);

        // Сначала тестируем код без try/catch
        let no_try_code = "global before = true\nglobal after = true";

        let result = interp.exec(no_try_code);
        assert!(result.is_ok(), "No try code should execute successfully: {:?}", result);

        // Проверяем код без try/catch
        assert_eq!(interp.get_variable("before"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("after"), Some(&Value::Bool(true)));

        // Теперь тестируем очень простой случай без исключений
        let very_simple_code = "global before_try = true\ntry\nglobal inside_try = true\ncatch error\nglobal caught_error = true\nendtry\nglobal after_try = true";

        println!("Executing code: {}", very_simple_code);
        let result = interp.exec(very_simple_code);
        assert!(result.is_ok(), "Very simple code should execute successfully: {:?}", result);

        // Проверяем очень простой случай
        println!("before_try: {:?}", interp.get_variable("before_try"));
        println!("inside_try: {:?}", interp.get_variable("inside_try"));
        println!("after_try: {:?}", interp.get_variable("after_try"));
        println!("caught_error: {:?}", interp.get_variable("caught_error"));

        assert_eq!(interp.get_variable("before_try"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("inside_try"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("after_try"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("caught_error"), None); // Не должно быть исключения

        // Теперь тестируем простой случай с исключением
        let simple_exception_code = "global before_exception = true\ntry\nglobal inside_exception = true\nthrow 'Test error'\nglobal after_throw = true\ncatch error\nglobal caught_exception = true\nglobal error_message = error\nendtry\nglobal after_exception = true";

        println!("Executing exception code: {}", simple_exception_code);
        let result = interp.exec(simple_exception_code);
        assert!(result.is_ok(), "Exception code should execute successfully: {:?}", result);

        // Проверяем случай с исключением
        println!("before_exception: {:?}", interp.get_variable("before_exception"));
        println!("inside_exception: {:?}", interp.get_variable("inside_exception"));
        println!("after_throw: {:?}", interp.get_variable("after_throw"));
        println!("caught_exception: {:?}", interp.get_variable("caught_exception"));
        println!("error_message: {:?}", interp.get_variable("error_message"));
        println!("after_exception: {:?}", interp.get_variable("after_exception"));

        assert_eq!(interp.get_variable("before_exception"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("inside_exception"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("after_throw"), None); // Не должно выполниться после throw
        assert_eq!(interp.get_variable("caught_exception"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("error_message"), Some(&Value::String("Test error".to_string())));
        assert_eq!(interp.get_variable("after_exception"), Some(&Value::Bool(true)));

        // Проверяем, что стек исключений очищен после выполнения
        assert_eq!(interp.exception_stack.len(), 0);
        assert_eq!(interp.get_try_nesting_level(), 0);
    }

    /// Тест проверки глубины рекурсии
    #[test]
    fn test_recursion_depth_tracking() {
        let mut interp = Interpreter::new();
        
        // Проверяем начальную глубину рекурсии
        assert_eq!(interp.recursion_depth, 0);
        
        // Создаем рекурсивную функцию с ограниченной глубиной
        let code = r#"
        global function factorial(n) do
            if n <= 1 do
                return 1
            endif
            return n * factorial(n - 1)
        endfunction
        
        global result = factorial(5)
        "#;
        
        let result = interp.exec(code);
        if let Err(e) = &result {
            println!("Error executing factorial: {:?}", e);
        }
        assert!(result.is_ok(), "Factorial should execute successfully");
        
        // Проверяем результат
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(120.0)));
        
        // Проверяем, что глубина рекурсии сброшена
        assert_eq!(interp.recursion_depth, 0);
    }

    /// Тест проверки состояния return_value
    #[test]
    fn test_return_value_state_management() {
        let mut interp = Interpreter::new();
        
        // Проверяем начальное состояние
        assert!(interp.return_value.is_none());
        
        let code = r#"
        global function test_return() do
            global inside_function = true
            return 42
            global after_return = true
        endfunction
        
        global result = test_return()
        global after_call = true
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Function call should succeed");
        
        // Проверяем, что return_value очищен после выполнения
        assert!(interp.return_value.is_none());
        
        // Проверяем правильность выполнения
        assert_eq!(interp.get_variable("result"), Some(&Value::Number(42.0)));
        assert_eq!(interp.get_variable("inside_function"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("after_call"), Some(&Value::Bool(true)));
        
        // Код после return не должен был выполниться
        assert_eq!(interp.get_variable("after_return"), None);
    }

    /// Тест проверки current_line для отслеживания ошибок
    #[test]
    fn test_current_line_tracking() {
        let mut interp = Interpreter::new();
        
        // Проверяем начальное значение
        assert_eq!(interp.current_line, 1);
        
        let code = r#"
        global line1 = true
        global line2 = true
        global line3 = true
        global undefined_var = nonexistent_variable
        global line5 = true
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_err(), "Should fail due to undefined variable");
        
        // Проверяем тип ошибки и информацию о строке
        match result.unwrap_err() {
            DataCodeError::VariableError { name, error_type, line } => {
                assert_eq!(name, "nonexistent_variable");
                assert_eq!(error_type, VariableErrorType::NotFound);
                // Строка должна быть больше 1 (где произошла ошибка)
                assert!(line > 1);
            }
            other => panic!("Expected VariableError, got {:?}", other),
        }
        
        // Проверяем, что переменные до ошибки были установлены
        assert_eq!(interp.get_variable("line1"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("line2"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("line3"), Some(&Value::Bool(true)));
        
        // Переменные после ошибки не должны были быть установлены
        assert_eq!(interp.get_variable("line5"), None);
    }

    /// Тест проверки сложных сценариев с множественными областями видимости
    #[test]
    fn test_complex_scope_management() {
        let mut interp = Interpreter::new();

        let code = r#"
        global outer_var = 'outer'
        global test_value = 'test'

        global function simple_function(param1) do
            global inner_var = 'inner'
            global processed_value = param1 + '_processed'
            return processed_value
        endfunction

        global final_result = simple_function(test_value)
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Complex scope code should execute successfully");

        // Проверяем глобальные переменные
        assert_eq!(interp.get_variable("outer_var"), Some(&Value::String("outer".to_string())));
        assert_eq!(interp.get_variable("inner_var"), Some(&Value::String("inner".to_string())));
        assert_eq!(interp.get_variable("final_result"), Some(&Value::String("test_processed".to_string())));
        assert_eq!(interp.get_variable("processed_value"), Some(&Value::String("test_processed".to_string())));
    }

    /// Тест проверки обработки ошибок в сложных сценариях
    #[test]
    fn test_complex_error_handling_with_state_inspection() {
        let mut interp = Interpreter::new();

        let code = r#"
        global errors_caught = []
        global execution_steps = []

        global function risky_operation(step) do
            global execution_steps = push(execution_steps, 'step_' + step)

            if step == '2' do
                throw 'Error in step 2'
            endif



            return 'success_' + step
        endfunction

        for step in ['1', '2', '3', '4', '5'] do
            try
                global result = risky_operation(step)
                global execution_steps = push(execution_steps, 'success_' + step)
            catch error
                global errors_caught = push(errors_caught, error)
                global execution_steps = push(execution_steps, 'caught_' + step)
            endtry
        next step
        "#;

        let result = interp.exec(code);

        // Код должен выполниться успешно с обработкой ошибок через try/catch
        assert!(result.is_ok(), "Code should execute successfully with try/catch handling errors");

        // Проверяем, что ошибки были правильно обработаны до критической ошибки
        let errors_caught = interp.get_variable("errors_caught").unwrap();
        if let Value::Array(errors) = errors_caught {
            assert_eq!(errors.len(), 1); // Только ошибка из шага 2 должна была быть поймана
            assert_eq!(errors[0], Value::String("Error in step 2".to_string()));
        } else {
            panic!("errors_caught should be an array");
        }

        // Проверяем шаги выполнения
        let execution_steps = interp.get_variable("execution_steps").unwrap();
        if let Value::Array(steps) = execution_steps {
            // Должны быть шаги: step_1, success_1, step_2, caught_2, step_3, success_3, step_4, success_4, step_5, success_5
            assert!(steps.len() >= 8);
            assert_eq!(steps[0], Value::String("step_1".to_string()));
            assert_eq!(steps[1], Value::String("success_1".to_string()));
            assert_eq!(steps[2], Value::String("step_2".to_string()));
            assert_eq!(steps[3], Value::String("caught_2".to_string()));
            assert_eq!(steps[4], Value::String("step_3".to_string()));
            assert_eq!(steps[5], Value::String("success_3".to_string()));
            assert_eq!(steps[6], Value::String("step_4".to_string()));
            assert_eq!(steps[7], Value::String("success_4".to_string()));
        } else {
            panic!("execution_steps should be an array");
        }

        // Проверяем состояние интерпретатора после ошибки
        assert_eq!(interp.variable_manager.function_depth(), 0);
        assert_eq!(interp.variable_manager.loop_depth(), 0);
        assert_eq!(interp.exception_stack.len(), 0);
    }
}

    /// Тест проверки управления памятью при работе с большими структурами данных
    #[test]
    fn test_memory_management_with_large_structures() {
        let mut interp = Interpreter::new();

        let code = r#"
        global large_array = []
        global nested_arrays = []

        # Создаем большой массив
        for i in range(100) do
            global large_array = push(large_array, i * 2)
        next i

        # Создаем вложенные массивы
        for i in range(10) do
            global data_array = []

            for j in range(10) do
                global data_array = push(data_array, 'item_' + str(i) + '_' + str(j))
            next j

            global nested_arrays = push(nested_arrays, data_array)
        next i

        # Проверяем размеры
        global array_size = length(large_array)
        global arrays_count = length(nested_arrays)
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Large structure creation should succeed");

        // Проверяем размеры созданных структур
        assert_eq!(interp.get_variable("array_size"), Some(&Value::Number(100.0)));
        assert_eq!(interp.get_variable("arrays_count"), Some(&Value::Number(10.0)));

        // Проверяем содержимое массива
        let large_array = interp.get_variable("large_array").unwrap();
        if let Value::Array(arr) = large_array {
            assert_eq!(arr.len(), 100);
            assert_eq!(arr[0], Value::Number(0.0));
            assert_eq!(arr[99], Value::Number(198.0));
        } else {
            panic!("large_array should be an array");
        }

        // Проверяем вложенные массивы
        let nested_arrays = interp.get_variable("nested_arrays").unwrap();
        if let Value::Array(arrays) = nested_arrays {
            assert_eq!(arrays.len(), 10);

            // Проверяем первый массив
            if let Value::Array(first_array) = &arrays[0] {
                assert_eq!(first_array.len(), 10);
                assert_eq!(first_array[0], Value::String("item_0_0".to_string()));
            } else {
                panic!("First element should be an array");
            }
        } else {
            panic!("nested_arrays should be an array");
        }

        // Проверяем, что все переменные все еще доступны
        let all_vars = interp.get_all_variables();
        assert!(all_vars.contains_key("large_array"));
        assert!(all_vars.contains_key("nested_arrays"));
        assert!(all_vars.contains_key("array_size"));
        assert!(all_vars.contains_key("arrays_count"));
    }

    /// Тест проверки состояния интерпретатора при рекурсивных исключениях
    #[test]
    fn test_recursive_exception_handling() {
        let mut interp = Interpreter::new();

        let code = r#"
        global max_depth_reached = 0
        global final_caught = false
        global final_error_msg = ''

        global function simple_thrower(depth) do
            global max_depth_reached = depth

            if depth >= 3 do
                throw 'Max depth reached: ' + str(depth)
            endif

            return simple_thrower(depth + 1)
        endfunction

        try
            global result = simple_thrower(0)
        catch final_error
            global final_error_msg = final_error
            global final_caught = true
        endtry
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Recursive exception handling should succeed");

        // Проверяем, что исключение было правильно обработано
        assert_eq!(interp.get_variable("final_caught"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("max_depth_reached"), Some(&Value::Number(3.0)));

        // Проверяем сообщение об ошибке
        let final_error_msg = interp.get_variable("final_error_msg").unwrap();
        if let Value::String(msg) = final_error_msg {
            assert!(msg.contains("Max depth reached: 3"));
        } else {
            panic!("final_error_msg should be a string");
        }

        // Проверяем, что стек исключений очищен
        assert_eq!(interp.exception_stack.len(), 0);
        assert_eq!(interp.recursion_depth, 0);
    }
