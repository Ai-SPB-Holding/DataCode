use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::{DataCodeError, VariableErrorType, FunctionErrorType};
use std::collections::HashMap;

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
        
        // Проверяем начальное состояние стека исключений
        assert_eq!(interp.exception_stack.len(), 0);
        assert_eq!(interp.get_try_nesting_level(), 0);
        
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
        
        // Проверяем, что стек исключений очищен после выполнения
        assert_eq!(interp.exception_stack.len(), 0);
        assert_eq!(interp.get_try_nesting_level(), 0);
        
        // Проверяем правильность выполнения
        assert_eq!(interp.get_variable("level1"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("level2"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("level3"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("deep_caught"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("deep_error_msg"), Some(&Value::String("Deep error".to_string())));
        assert_eq!(interp.get_variable("after_inner_try"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("after_middle_try"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("after_outer_try"), Some(&Value::Bool(true)));
        
        // Внешние catch блоки не должны были выполниться
        assert_eq!(interp.get_variable("middle_caught"), None);
        assert_eq!(interp.get_variable("outer_caught"), None);
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

        global function outer_function(param1) do
            global inner_var = 'inner'
            local local_var = param1

            global function nested_function(param2) do
                local nested_local = param2
                global nested_global = 'nested_global'

                for i in [1, 2, 3] do
                    local loop_local = i * 2
                    global loop_global = 'loop_' + i

                    for j in ['a', 'b'] do
                        local inner_loop_local = j + '_' + i
                        global inner_loop_global = 'inner_' + j + '_' + i
                    forend
                forend

                return nested_local + '_processed'
            endfunction

            local result = nested_function(local_var)
            return result
        endfunction

        global final_result = outer_function('test')
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Complex scope code should execute successfully");

        // Проверяем глобальные переменные
        assert_eq!(interp.get_variable("outer_var"), Some(&Value::String("outer".to_string())));
        assert_eq!(interp.get_variable("inner_var"), Some(&Value::String("inner".to_string())));
        assert_eq!(interp.get_variable("nested_global"), Some(&Value::String("nested_global".to_string())));
        assert_eq!(interp.get_variable("final_result"), Some(&Value::String("test_processed".to_string())));

        // Проверяем переменные циклов (должны быть глобальными)
        assert_eq!(interp.get_variable("loop_global"), Some(&Value::String("loop_3".to_string())));
        assert_eq!(interp.get_variable("inner_loop_global"), Some(&Value::String("inner_b_3".to_string())));

        // Проверяем, что локальные переменные не видны после выхода из функций
        assert_eq!(interp.get_variable("local_var"), None);
        assert_eq!(interp.get_variable("nested_local"), None);
        assert_eq!(interp.get_variable("loop_local"), None);
        assert_eq!(interp.get_variable("inner_loop_local"), None);

        // Проверяем состояние менеджера переменных
        assert_eq!(interp.variable_manager.function_depth(), 0);
        assert_eq!(interp.variable_manager.loop_depth(), 0);
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

            if step == '4' do
                global undefined_result = nonexistent_var
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
        forend
        "#;

        let result = interp.exec(code);

        // Код должен частично выполниться, но упасть на шаге 4 из-за неопределенной переменной
        assert!(result.is_err(), "Should fail on step 4 due to undefined variable");

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
            // Должны быть шаги: step_1, success_1, step_2, caught_2, step_3, success_3, step_4
            assert!(steps.len() >= 6);
            assert_eq!(steps[0], Value::String("step_1".to_string()));
            assert_eq!(steps[1], Value::String("success_1".to_string()));
            assert_eq!(steps[2], Value::String("step_2".to_string()));
            assert_eq!(steps[3], Value::String("caught_2".to_string()));
            assert_eq!(steps[4], Value::String("step_3".to_string()));
            assert_eq!(steps[5], Value::String("success_3".to_string()));
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
        global nested_objects = []

        # Создаем большой массив
        for i in range(1000) do
            global large_array = push(large_array, i * 2)
        forend

        # Создаем вложенные структуры
        for i in range(100) do
            global obj = {}
            global obj['id'] = i
            global obj['data'] = []

            for j in range(50) do
                global obj['data'] = push(obj['data'], 'item_' + i + '_' + j)
            forend

            global nested_objects = push(nested_objects, obj)
        forend

        # Проверяем размеры
        global array_size = len(large_array)
        global objects_count = len(nested_objects)
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Large structure creation should succeed");

        // Проверяем размеры созданных структур
        assert_eq!(interp.get_variable("array_size"), Some(&Value::Number(1000.0)));
        assert_eq!(interp.get_variable("objects_count"), Some(&Value::Number(100.0)));

        // Проверяем содержимое массива
        let large_array = interp.get_variable("large_array").unwrap();
        if let Value::Array(arr) = large_array {
            assert_eq!(arr.len(), 1000);
            assert_eq!(arr[0], Value::Number(0.0));
            assert_eq!(arr[999], Value::Number(1998.0));
        } else {
            panic!("large_array should be an array");
        }

        // Проверяем вложенные объекты
        let nested_objects = interp.get_variable("nested_objects").unwrap();
        if let Value::Array(objects) = nested_objects {
            assert_eq!(objects.len(), 100);

            // Проверяем первый объект
            if let Value::Object(first_obj) = &objects[0] {
                assert_eq!(first_obj.get("id"), Some(&Value::Number(0.0)));
                if let Some(Value::Array(data)) = first_obj.get("data") {
                    assert_eq!(data.len(), 50);
                    assert_eq!(data[0], Value::String("item_0_0".to_string()));
                } else {
                    panic!("Object data should be an array");
                }
            } else {
                panic!("First element should be an object");
            }
        } else {
            panic!("nested_objects should be an array");
        }

        // Проверяем, что все переменные все еще доступны
        let all_vars = interp.get_all_variables();
        assert!(all_vars.contains_key("large_array"));
        assert!(all_vars.contains_key("nested_objects"));
        assert!(all_vars.contains_key("array_size"));
        assert!(all_vars.contains_key("objects_count"));
    }

    /// Тест проверки состояния интерпретатора при рекурсивных исключениях
    #[test]
    fn test_recursive_exception_handling() {
        let mut interp = Interpreter::new();

        let code = r#"
        global exception_depth = 0
        global max_depth_reached = 0

        global function recursive_thrower(depth) do
            global exception_depth = depth
            if depth > global max_depth_reached do
                global max_depth_reached = depth
            endif

            if depth >= 5 do
                throw 'Max depth reached: ' + depth
            endif

            try
                return recursive_thrower(depth + 1)
            catch error
                global caught_at_depth = depth
                throw 'Propagated from depth ' + depth + ': ' + error
            endtry
        endfunction

        try
            global result = recursive_thrower(0)
        catch final_error
            global final_error_msg = final_error
            global final_caught = true
        endtry
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Recursive exception handling should succeed");

        // Проверяем, что исключение было правильно обработано
        assert_eq!(interp.get_variable("final_caught"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("max_depth_reached"), Some(&Value::Number(5.0)));
        assert_eq!(interp.get_variable("caught_at_depth"), Some(&Value::Number(4.0)));

        // Проверяем сообщение об ошибке
        let final_error_msg = interp.get_variable("final_error_msg").unwrap();
        if let Value::String(msg) = final_error_msg {
            assert!(msg.contains("Propagated from depth 4"));
            assert!(msg.contains("Max depth reached: 5"));
        } else {
            panic!("final_error_msg should be a string");
        }

        // Проверяем, что стек исключений очищен
        assert_eq!(interp.exception_stack.len(), 0);
        assert_eq!(interp.recursion_depth, 0);
    }
