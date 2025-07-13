use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod multiple_variables_for_tests {
    use super::*;

    #[test]
    fn test_for_with_two_variables_enum() {
        let mut interp = Interpreter::new();
        
        // Создаем массив для тестирования
        interp.exec("global data = ['a', 'b', 'c']").unwrap();
        
        // Используем синтаксис for i, value in enum(data)
        let loop_code = r#"
            global collected_indices = []
            global collected_values = []
            for i, value in enum(data) do
                global collected_indices = push(collected_indices, i)
                global collected_values = push(collected_values, value)
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        
        // Проверяем собранные индексы
        if let Some(Value::Array(indices)) = interp.get_variable("collected_indices") {
            assert_eq!(indices.len(), 3);
            assert_eq!(indices[0], Value::Number(0.0));
            assert_eq!(indices[1], Value::Number(1.0));
            assert_eq!(indices[2], Value::Number(2.0));
        } else {
            panic!("Expected indices array");
        }
        
        // Проверяем собранные значения
        if let Some(Value::Array(values)) = interp.get_variable("collected_values") {
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], Value::String("a".to_string()));
            assert_eq!(values[1], Value::String("b".to_string()));
            assert_eq!(values[2], Value::String("c".to_string()));
        } else {
            panic!("Expected values array");
        }
    }

    #[test]
    fn test_for_with_two_variables_custom_pairs() {
        let mut interp = Interpreter::new();
        
        // Создаем массив пар для тестирования
        interp.exec("global pairs = [[1, 'one'], [2, 'two'], [3, 'three']]").unwrap();
        
        // Используем синтаксис for num, word in pairs
        let loop_code = r#"
            global numbers = []
            global words = []
            for num, word in pairs do
                global numbers = push(numbers, num)
                global words = push(words, word)
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        
        // Проверяем собранные числа
        if let Some(Value::Array(numbers)) = interp.get_variable("numbers") {
            assert_eq!(numbers.len(), 3);
            assert_eq!(numbers[0], Value::Number(1.0));
            assert_eq!(numbers[1], Value::Number(2.0));
            assert_eq!(numbers[2], Value::Number(3.0));
        } else {
            panic!("Expected numbers array");
        }
        
        // Проверяем собранные слова
        if let Some(Value::Array(words)) = interp.get_variable("words") {
            assert_eq!(words.len(), 3);
            assert_eq!(words[0], Value::String("one".to_string()));
            assert_eq!(words[1], Value::String("two".to_string()));
            assert_eq!(words[2], Value::String("three".to_string()));
        } else {
            panic!("Expected words array");
        }
    }

    #[test]
    fn test_for_with_three_variables() {
        let mut interp = Interpreter::new();
        
        // Создаем массив троек для тестирования
        interp.exec("global triples = [[1, 'a', true], [2, 'b', false], [3, 'c', true]]").unwrap();
        
        // Используем синтаксис for num, letter, flag in triples
        let loop_code = r#"
            global sum = 0
            global letters = []
            global true_count = 0
            for num, letter, flag in triples do
                global sum = sum + num
                global letters = push(letters, letter)
                if flag do
                    global true_count = true_count + 1
                endif
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        
        // Проверяем результаты
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(6.0))); // 1 + 2 + 3
        assert_eq!(interp.get_variable("true_count"), Some(&Value::Number(2.0))); // true, false, true
        
        if let Some(Value::Array(letters)) = interp.get_variable("letters") {
            assert_eq!(letters.len(), 3);
            assert_eq!(letters[0], Value::String("a".to_string()));
            assert_eq!(letters[1], Value::String("b".to_string()));
            assert_eq!(letters[2], Value::String("c".to_string()));
        } else {
            panic!("Expected letters array");
        }
    }

    #[test]
    fn test_for_single_variable_still_works() {
        let mut interp = Interpreter::new();
        
        // Проверяем, что старый синтаксис с одной переменной все еще работает
        interp.exec("global numbers = [10, 20, 30]").unwrap();
        
        let loop_code = r#"
            global total = 0
            for num in numbers do
                global total = total + num
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        assert_eq!(interp.get_variable("total"), Some(&Value::Number(60.0)));
    }

    #[test]
    fn test_for_error_wrong_array_length() {
        let mut interp = Interpreter::new();
        
        // Создаем массив с неправильной длиной элементов
        interp.exec("global wrong_pairs = [[1, 'one'], [2], [3, 'three', 'extra']]").unwrap();
        
        // Пытаемся использовать синтаксис for a, b in wrong_pairs
        let loop_code = r#"
            for a, b in wrong_pairs do
                print(a, b)
            forend
        "#;
        
        let result = interp.exec(loop_code);
        assert!(result.is_err());
        
        // Проверяем, что ошибка содержит информацию о несоответствии длины
        if let Err(e) = result {
            assert!(e.to_string().contains("Cannot unpack array"));
        }
    }

    #[test]
    fn test_for_error_non_array_unpacking() {
        let mut interp = Interpreter::new();
        
        // Создаем массив с не-массивами
        interp.exec("global non_arrays = [42, 'hello', true]").unwrap();
        
        // Пытаемся использовать синтаксис for a, b in non_arrays
        let loop_code = r#"
            for a, b in non_arrays do
                print(a, b)
            forend
        "#;
        
        let result = interp.exec(loop_code);
        assert!(result.is_err());
        
        // Проверяем, что ошибка содержит информацию о невозможности распаковки
        if let Err(e) = result {
            assert!(e.to_string().contains("Cannot unpack non-array value"));
        }
    }

    #[test]
    fn test_for_with_enum_and_isinstance() {
        let mut interp = Interpreter::new();
        
        // Создаем массив смешанных типов
        interp.exec("global mixed_data = [42, 'hello', true, null]").unwrap();
        
        // Используем enum с isinstance для фильтрации
        let loop_code = r#"
            global string_positions = []
            for i, value in enum(mixed_data) do
                if isinstance(value, 'string') do
                    global string_positions = push(string_positions, i)
                endif
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        
        // Проверяем, что нашли строку на позиции 1
        if let Some(Value::Array(positions)) = interp.get_variable("string_positions") {
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0], Value::Number(1.0));
        } else {
            panic!("Expected string_positions array");
        }
    }

    #[test]
    fn test_for_nested_with_multiple_variables() {
        let mut interp = Interpreter::new();
        
        // Создаем вложенную структуру
        interp.exec("global matrix = [[[1, 'a'], [2, 'b']], [[3, 'c'], [4, 'd']]]").unwrap();
        
        // Используем вложенные циклы с множественными переменными
        let loop_code = r#"
            global result = []
            for row_idx, row in enum(matrix) do
                for col_idx, pair in enum(row) do
                    for num, letter in [pair] do
                        global combined = [row_idx, col_idx, num, letter]
                        global result = push(result, combined)
                    forend
                forend
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        
        // Проверяем результат
        if let Some(Value::Array(result)) = interp.get_variable("result") {
            assert_eq!(result.len(), 4); // 2x2 матрица
            
            // Проверяем первый элемент [0, 0, 1, 'a']
            if let Value::Array(first) = &result[0] {
                assert_eq!(first.len(), 4);
                assert_eq!(first[0], Value::Number(0.0)); // row_idx
                assert_eq!(first[1], Value::Number(0.0)); // col_idx
                assert_eq!(first[2], Value::Number(1.0)); // num
                assert_eq!(first[3], Value::String("a".to_string())); // letter
            } else {
                panic!("Expected array in result");
            }
        } else {
            panic!("Expected result array");
        }
    }

    #[test]
    fn test_for_empty_variable_name_error() {
        let mut interp = Interpreter::new();
        
        interp.exec("global data = [[1, 2], [3, 4]]").unwrap();
        
        // Пытаемся использовать пустое имя переменной
        let loop_code = r#"
            for , b in data do
                print(b)
            forend
        "#;
        
        let result = interp.exec(loop_code);
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("Empty variable name"));
        }
    }

    #[test]
    fn test_for_whitespace_handling() {
        let mut interp = Interpreter::new();
        
        // Создаем данные для тестирования
        interp.exec("global pairs = [[10, 20], [30, 40]]").unwrap();
        
        // Тестируем различные варианты пробелов
        let loop_code = r#"
            global sum1 = 0
            global sum2 = 0
            for  a  ,  b  in pairs do
                global sum1 = sum1 + a
                global sum2 = sum2 + b
            forend
        "#;
        
        interp.exec(loop_code).unwrap();
        
        assert_eq!(interp.get_variable("sum1"), Some(&Value::Number(40.0))); // 10 + 30
        assert_eq!(interp.get_variable("sum2"), Some(&Value::Number(60.0))); // 20 + 40
    }
}
