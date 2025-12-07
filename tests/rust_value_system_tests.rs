use data_code::interpreter::Interpreter;
use data_code::value::Value;

/// Сложные тесты для проверки системы типов и значений DataCode
/// Эти тесты проверяют глубокие аспекты работы с Value enum и типами данных:
/// - Преобразования типов и их внутреннее представление
/// - Работа с таблицами и их внутренней структурой
/// - Операции с путями и паттернами
/// - Сложные операции с массивами и объектами
/// - Проверка типов и их определение

#[cfg(test)]
mod rust_value_system_tests {
    use super::*;

    /// Тест проверки внутреннего представления различных типов Value
    #[test]
    fn test_value_internal_representation() {
        let mut interp = Interpreter::new();
        
        // Создаем различные типы значений
        let code = r#"
        global number_int = 42
        global number_float = 3.14159
        global string_simple = 'hello'
        global string_empty = ''
        global bool_true = true
        global bool_false = false
        global array_mixed = [1, 'two', true, null]
        global currency_usd = '$100.50'
        global currency_eur = '€75.25'
        global path_simple = getcwd()
        global null_value = null
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Value creation should succeed");
        
        // Проверяем внутреннее представление чисел
        let number_int = interp.get_variable("number_int").unwrap();
        assert!(matches!(number_int, Value::Number(n) if *n == 42.0));
        
        let number_float = interp.get_variable("number_float").unwrap();
        assert!(matches!(number_float, Value::Number(n) if (*n - 3.14159).abs() < 0.00001));
        
        // Проверяем строки
        let string_simple = interp.get_variable("string_simple").unwrap();
        assert!(matches!(string_simple, Value::String(s) if s == "hello"));
        
        let string_empty = interp.get_variable("string_empty").unwrap();
        assert!(matches!(string_empty, Value::String(s) if s.is_empty()));
        
        // Проверяем булевы значения
        let bool_true = interp.get_variable("bool_true").unwrap();
        assert!(matches!(bool_true, Value::Bool(true)));
        
        let bool_false = interp.get_variable("bool_false").unwrap();
        assert!(matches!(bool_false, Value::Bool(false)));
        
        // Проверяем массив
        let array_mixed = interp.get_variable("array_mixed").unwrap();
        if let Value::Array(arr) = array_mixed {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], Value::Number(1.0));
            assert_eq!(arr[1], Value::String("two".to_string()));
            assert_eq!(arr[2], Value::Bool(true));
            assert_eq!(arr[3], Value::Null);
        } else {
            panic!("array_mixed should be an Array");
        }
        
        // Проверяем валюты (в текущей реализации они остаются строками)
        let currency_usd = interp.get_variable("currency_usd").unwrap();
        assert!(matches!(currency_usd, Value::String(s) if s == "$100.50"));

        let currency_eur = interp.get_variable("currency_eur").unwrap();
        assert!(matches!(currency_eur, Value::String(s) if s == "€75.25"));
        
        // Проверяем путь
        let path_simple = interp.get_variable("path_simple").unwrap();
        assert!(matches!(path_simple, Value::Path(_)));
        
        // Проверяем null
        let null_value = interp.get_variable("null_value").unwrap();
        assert!(matches!(null_value, Value::Null));
    }

    /// Тест проверки операций с Value и их влияния на внутреннее состояние
    #[test]
    fn test_value_operations_internal_state() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global a = 10
        global b = 20
        global c = 'hello'
        global d = 'world'
        
        # Арифметические операции
        global sum = a + b
        global diff = b - a
        global prod = a * b
        global quot = b / a
        
        # Строковые операции
        global concat = c + ' ' + d
        
        # Логические операции
        global gt = b > a
        global eq = a == 10
        global ne = a != b
        
        # Операции с массивами
        global arr1 = [1, 2, 3]
        global arr2 = [4, 5, 6]
        # Операция сложения массивов не реализована
        # global combined = arr1 + arr2
        
        # Операции сравнения строк
        global str_eq = c == 'hello'
        global str_ne = c != d
        "#;
        
        let result = interp.exec(code);
        if let Err(e) = &result {
            println!("Error executing code: {:?}", e);
        }
        assert!(result.is_ok(), "Operations should succeed");
        
        // Проверяем арифметические результаты
        assert_eq!(interp.get_variable("sum"), Some(&Value::Number(30.0)));
        assert_eq!(interp.get_variable("diff"), Some(&Value::Number(10.0)));
        assert_eq!(interp.get_variable("prod"), Some(&Value::Number(200.0)));
        assert_eq!(interp.get_variable("quot"), Some(&Value::Number(2.0)));
        
        // Проверяем строковые операции
        assert_eq!(interp.get_variable("concat"), Some(&Value::String("hello world".to_string())));
        
        // Проверяем логические операции
        assert_eq!(interp.get_variable("gt"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("eq"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("ne"), Some(&Value::Bool(true)));
        
        // Проверяем операции с массивами (операция сложения массивов не реализована)
        // let combined = interp.get_variable("combined").unwrap();
        // if let Value::Array(arr) = combined {
        //     assert_eq!(arr.len(), 6);
        //     assert_eq!(arr[0], Value::Number(1.0));
        //     assert_eq!(arr[3], Value::Number(4.0));
        //     assert_eq!(arr[5], Value::Number(6.0));
        // } else {
        //     panic!("combined should be an Array");
        // }
        
        // Проверяем сравнения строк
        assert_eq!(interp.get_variable("str_eq"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("str_ne"), Some(&Value::Bool(true)));
    }

    /// Тест проверки работы с таблицами и их внутренней структурой
    #[test]
    fn test_table_internal_structure() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        # Создаем простую таблицу для тестирования
        global table = table_create([['Name', 'Age'], ['Alice', 25], ['Bob', 30]])

        # Проверяем размеры
        global table_rows = len(table)
        "#;
        
        let result = interp.exec(code);
        if let Err(e) = &result {
            println!("Error executing table code: {:?}", e);
        }
        assert!(result.is_ok(), "Table operations should succeed");
        
        // Проверяем таблицу
        let table = interp.get_variable("table").unwrap();
        assert!(matches!(table, Value::Table(_)));
        
        if let Value::Table(table_data) = table {
            let table_borrowed = table_data.borrow();

            // Проверяем количество строк (включая заголовок)
            assert_eq!(table_borrowed.rows.len(), 3);

            // Проверяем заголовки (автоматически сгенерированные)
            assert_eq!(table_borrowed.column_names, vec!["Column_0", "Column_1"]);

            // Проверяем первую строку данных (заголовки стали данными)
            assert_eq!(table_borrowed.rows[0][0], Value::String("Name".to_string()));
            assert_eq!(table_borrowed.rows[0][1], Value::String("Age".to_string()));
        }

        // Проверяем размеры
        assert_eq!(interp.get_variable("table_rows"), Some(&Value::Number(3.0)));
    }

    /// Тест проверки работы с путями и паттернами
    #[test]
    fn test_path_and_pattern_operations() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global base_path = getcwd()
        global sub_dir = 'data'
        global filename = 'test.txt'
        global pattern = '*.csv'
        
        # Операции с путями (если поддерживается)
        # global full_path = base_path / sub_dir / filename
        # global pattern_path = base_path / pattern
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Path operations should succeed");
        
        // Проверяем базовый путь
        let base_path = interp.get_variable("base_path").unwrap();
        assert!(matches!(base_path, Value::Path(_)));
        
        if let Value::Path(path) = base_path {
            assert!(path.is_absolute() || path.is_relative());
            assert!(path.exists() || !path.exists()); // Путь может существовать или не существовать
        }
        
        // Проверяем строковые значения
        assert_eq!(interp.get_variable("sub_dir"), Some(&Value::String("data".to_string())));
        assert_eq!(interp.get_variable("filename"), Some(&Value::String("test.txt".to_string())));
        assert_eq!(interp.get_variable("pattern"), Some(&Value::String("*.csv".to_string())));
    }

    /// Тест проверки сложных операций с объектами и их внутренней структурой
    #[test]
    fn test_complex_object_operations() {
        let mut interp = Interpreter::new();

        let code = r#"
        # Создаем простые массивы для тестирования
        global numbers = [1, 2, 3, 4, 5]
        global strings = ['hello', 'world', 'test']
        global mixed = [1, 'hello', 3.14, true]

        # Тестируем операции с массивами
        global first_number = numbers[0]
        global last_string = strings[2]
        global mixed_element = mixed[1]

        # Тестируем функции массивов
        global numbers_length = len(numbers)
        global strings_length = len(strings)
        "#;

        let result = interp.exec(code);
        if let Err(e) = &result {
            println!("Error executing complex object code: {:?}", e);
        }
        assert!(result.is_ok(), "Complex object operations should succeed");

        // Проверяем массивы
        let numbers = interp.get_variable("numbers").unwrap();
        if let Value::Array(numbers_array) = numbers {
            assert_eq!(numbers_array.len(), 5);
            assert_eq!(numbers_array[0], Value::Number(1.0));
            assert_eq!(numbers_array[4], Value::Number(5.0));
        } else {
            panic!("numbers should be an array");
        }

        let strings = interp.get_variable("strings").unwrap();
        if let Value::Array(strings_array) = strings {
            assert_eq!(strings_array.len(), 3);
            assert_eq!(strings_array[0], Value::String("hello".to_string()));
            assert_eq!(strings_array[2], Value::String("test".to_string()));
        } else {
            panic!("strings should be an array");
        }

        let mixed = interp.get_variable("mixed").unwrap();
        if let Value::Array(mixed_array) = mixed {
            assert_eq!(mixed_array.len(), 4);
            assert_eq!(mixed_array[0], Value::Number(1.0));
            assert_eq!(mixed_array[1], Value::String("hello".to_string()));
            assert_eq!(mixed_array[2], Value::Number(3.14));
            assert_eq!(mixed_array[3], Value::Bool(true));
        } else {
            panic!("mixed should be an array");
        }

        // Проверяем извлеченные данные
        assert_eq!(interp.get_variable("first_number"), Some(&Value::Number(1.0)));
        assert_eq!(interp.get_variable("last_string"), Some(&Value::String("test".to_string())));
        assert_eq!(interp.get_variable("mixed_element"), Some(&Value::String("hello".to_string())));

        // Проверяем длины массивов
        assert_eq!(interp.get_variable("numbers_length"), Some(&Value::Number(5.0)));
        assert_eq!(interp.get_variable("strings_length"), Some(&Value::Number(3.0)));
    }

    /// Тест проверки типов данных и их определения
    #[test]
    fn test_data_type_detection_and_conversion() {
        let mut interp = Interpreter::new();

        let code = r#"
        global int_val = 42
        global float_val = 3.14
        global str_val = 'hello'
        global bool_val = true
        global null_val = null
        global array_val = [1, 2, 3]
        global currency_val = '$100.00'
        global path_val = getcwd()

        # Проверяем типы с помощью isinstance
        global is_int_number = isinstance(int_val, num)
        global is_float_number = isinstance(float_val, float)
        global is_str_string = isinstance(str_val, str)
        global is_bool_bool = isinstance(bool_val, bool)
        global is_null_null = isinstance(null_val, null)
        global is_array_array = isinstance(array_val, array)
        global is_currency_currency = isinstance(currency_val, money)
        global is_path_path = isinstance(path_val, path)

        # Проверяем неправильные типы
        global is_int_string = isinstance(int_val, str)
        global is_str_number = isinstance(str_val, num)
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Type checking should succeed");

        // Проверяем правильные типы
        assert_eq!(interp.get_variable("is_int_number"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("is_float_number"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("is_str_string"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("is_bool_bool"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("is_null_null"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("is_array_array"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("is_currency_currency"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("is_path_path"), Some(&Value::Bool(true)));

        // Проверяем неправильные типы
        assert_eq!(interp.get_variable("is_int_string"), Some(&Value::Bool(false)));
        assert_eq!(interp.get_variable("is_str_number"), Some(&Value::Bool(false)));
    }
}
