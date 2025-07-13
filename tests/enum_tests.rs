use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod enum_tests {
    use super::*;

    #[test]
    fn test_enum_with_array() {
        let mut interp = Interpreter::new();
        
        // Создаем массив для тестирования
        interp.exec("global test_array = [10, 20, 30]").unwrap();
        
        // Используем enum для получения пар [индекс, значение]
        interp.exec("global enumerated = enum(test_array)").unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("enumerated") {
            assert_eq!(result.len(), 3);
            
            // Проверяем первую пару [0, 10]
            if let Value::Array(first_pair) = &result[0] {
                assert_eq!(first_pair.len(), 2);
                assert_eq!(first_pair[0], Value::Number(0.0));
                assert_eq!(first_pair[1], Value::Number(10.0));
            } else {
                panic!("Expected array pair");
            }
            
            // Проверяем вторую пару [1, 20]
            if let Value::Array(second_pair) = &result[1] {
                assert_eq!(second_pair.len(), 2);
                assert_eq!(second_pair[0], Value::Number(1.0));
                assert_eq!(second_pair[1], Value::Number(20.0));
            } else {
                panic!("Expected array pair");
            }
            
            // Проверяем третью пару [2, 30]
            if let Value::Array(third_pair) = &result[2] {
                assert_eq!(third_pair.len(), 2);
                assert_eq!(third_pair[0], Value::Number(2.0));
                assert_eq!(third_pair[1], Value::Number(30.0));
            } else {
                panic!("Expected array pair");
            }
        } else {
            panic!("Expected array result from enum");
        }
    }

    #[test]
    fn test_enum_with_string() {
        let mut interp = Interpreter::new();
        
        // Создаем строку для тестирования
        interp.exec("global test_string = 'abc'").unwrap();
        
        // Используем enum для получения пар [индекс, символ]
        interp.exec("global enumerated = enum(test_string)").unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("enumerated") {
            assert_eq!(result.len(), 3);
            
            // Проверяем первую пару [0, 'a']
            if let Value::Array(first_pair) = &result[0] {
                assert_eq!(first_pair.len(), 2);
                assert_eq!(first_pair[0], Value::Number(0.0));
                assert_eq!(first_pair[1], Value::String("a".to_string()));
            } else {
                panic!("Expected array pair");
            }
            
            // Проверяем вторую пару [1, 'b']
            if let Value::Array(second_pair) = &result[1] {
                assert_eq!(second_pair.len(), 2);
                assert_eq!(second_pair[0], Value::Number(1.0));
                assert_eq!(second_pair[1], Value::String("b".to_string()));
            } else {
                panic!("Expected array pair");
            }
            
            // Проверяем третью пару [2, 'c']
            if let Value::Array(third_pair) = &result[2] {
                assert_eq!(third_pair.len(), 2);
                assert_eq!(third_pair[0], Value::Number(2.0));
                assert_eq!(third_pair[1], Value::String("c".to_string()));
            } else {
                panic!("Expected array pair");
            }
        } else {
            panic!("Expected array result from enum");
        }
    }

    #[test]
    fn test_enum_with_empty_array() {
        let mut interp = Interpreter::new();
        
        // Создаем пустой массив
        interp.exec("global empty_array = []").unwrap();
        
        // Используем enum с пустым массивом
        interp.exec("global enumerated = enum(empty_array)").unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("enumerated") {
            assert_eq!(result.len(), 0);
        } else {
            panic!("Expected array result from enum");
        }
    }

    #[test]
    fn test_enum_with_empty_string() {
        let mut interp = Interpreter::new();
        
        // Создаем пустую строку
        interp.exec("global empty_string = ''").unwrap();
        
        // Используем enum с пустой строкой
        interp.exec("global enumerated = enum(empty_string)").unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("enumerated") {
            assert_eq!(result.len(), 0);
        } else {
            panic!("Expected array result from enum");
        }
    }

    #[test]
    fn test_enum_with_mixed_array() {
        let mut interp = Interpreter::new();
        
        // Создаем массив смешанных типов
        interp.exec("global mixed_array = [42, 'hello', true, null]").unwrap();
        
        // Используем enum
        interp.exec("global enumerated = enum(mixed_array)").unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("enumerated") {
            assert_eq!(result.len(), 4);
            
            // Проверяем первую пару [0, 42]
            if let Value::Array(first_pair) = &result[0] {
                assert_eq!(first_pair[0], Value::Number(0.0));
                assert_eq!(first_pair[1], Value::Number(42.0));
            } else {
                panic!("Expected array pair");
            }
            
            // Проверяем вторую пару [1, 'hello']
            if let Value::Array(second_pair) = &result[1] {
                assert_eq!(second_pair[0], Value::Number(1.0));
                assert_eq!(second_pair[1], Value::String("hello".to_string()));
            } else {
                panic!("Expected array pair");
            }
            
            // Проверяем третью пару [2, true]
            if let Value::Array(third_pair) = &result[2] {
                assert_eq!(third_pair[0], Value::Number(2.0));
                assert_eq!(third_pair[1], Value::Bool(true));
            } else {
                panic!("Expected array pair");
            }
            
            // Проверяем четвертую пару [3, null]
            if let Value::Array(fourth_pair) = &result[3] {
                assert_eq!(fourth_pair[0], Value::Number(3.0));
                assert_eq!(fourth_pair[1], Value::Null);
            } else {
                panic!("Expected array pair");
            }
        } else {
            panic!("Expected array result from enum");
        }
    }

    #[test]
    fn test_enum_in_for_loop() {
        let mut interp = Interpreter::new();
        
        // Создаем массив для тестирования
        interp.exec("global data = ['a', 'b', 'c']").unwrap();
        
        // Используем enum в цикле для сбора индексов
        let loop_code = r#"
            global indices = []
            global values = []
            for pair in enum(data) do
                global index = pair[0]
                global value = pair[1]
                global indices = push(indices, index)
                global values = push(values, value)
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        
        // Проверяем собранные индексы
        if let Some(Value::Array(indices)) = interp.get_variable("indices") {
            assert_eq!(indices.len(), 3);
            assert_eq!(indices[0], Value::Number(0.0));
            assert_eq!(indices[1], Value::Number(1.0));
            assert_eq!(indices[2], Value::Number(2.0));
        } else {
            panic!("Expected indices array");
        }
        
        // Проверяем собранные значения
        if let Some(Value::Array(values)) = interp.get_variable("values") {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], Value::String("a".to_string()));
            assert_eq!(values[1], Value::String("b".to_string()));
            assert_eq!(values[2], Value::String("c".to_string()));
        } else {
            panic!("Expected values array");
        }
    }

    #[test]
    fn test_enum_wrong_argument_count() {
        let mut interp = Interpreter::new();
        
        // Тест с недостаточным количеством аргументов
        let result = interp.exec("global test1 = enum()");
        assert!(result.is_err());
        
        // Тест с избыточным количеством аргументов
        let result = interp.exec("global test2 = enum([1, 2], 'extra')");
        assert!(result.is_err());
    }

    #[test]
    fn test_enum_invalid_type() {
        let mut interp = Interpreter::new();
        
        // Тест с числом (неподдерживаемый тип)
        let result = interp.exec("global test = enum(42)");
        assert!(result.is_err());
        
        // Тест с булевым значением
        let result = interp.exec("global test = enum(true)");
        assert!(result.is_err());
    }

    #[test]
    fn test_enum_with_unicode_string() {
        let mut interp = Interpreter::new();
        
        // Создаем строку с Unicode символами
        interp.exec("global unicode_string = 'привет'").unwrap();
        
        // Используем enum
        interp.exec("global enumerated = enum(unicode_string)").unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("enumerated") {
            assert_eq!(result.len(), 6); // 'привет' содержит 6 символов
            
            // Проверяем первую пару [0, 'п']
            if let Value::Array(first_pair) = &result[0] {
                assert_eq!(first_pair[0], Value::Number(0.0));
                assert_eq!(first_pair[1], Value::String("п".to_string()));
            } else {
                panic!("Expected array pair");
            }
        } else {
            panic!("Expected array result from enum");
        }
    }

    #[test]
    fn test_enum_with_nested_arrays() {
        let mut interp = Interpreter::new();
        
        // Создаем массив с вложенными массивами
        interp.exec("global nested = [[1, 2], [3, 4], [5, 6]]").unwrap();
        
        // Используем enum
        interp.exec("global enumerated = enum(nested)").unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("enumerated") {
            assert_eq!(result.len(), 3);
            
            // Проверяем первую пару [0, [1, 2]]
            if let Value::Array(first_pair) = &result[0] {
                assert_eq!(first_pair[0], Value::Number(0.0));
                if let Value::Array(nested_array) = &first_pair[1] {
                    assert_eq!(nested_array.len(), 2);
                    assert_eq!(nested_array[0], Value::Number(1.0));
                    assert_eq!(nested_array[1], Value::Number(2.0));
                } else {
                    panic!("Expected nested array");
                }
            } else {
                panic!("Expected array pair");
            }
        } else {
            panic!("Expected array result from enum");
        }
    }
}
