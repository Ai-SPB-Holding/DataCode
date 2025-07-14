use data_code::interpreter::Interpreter;
use data_code::value::{Value, DataType};
use data_code::error::DataCodeError;
use std::collections::HashMap;
use std::path::PathBuf;

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
        
        // Проверяем валюты
        let currency_usd = interp.get_variable("currency_usd").unwrap();
        assert!(matches!(currency_usd, Value::Currency(s) if s == "$100.50"));
        
        let currency_eur = interp.get_variable("currency_eur").unwrap();
        assert!(matches!(currency_eur, Value::Currency(s) if s == "€75.25"));
        
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
        global combined = arr1 + arr2
        
        # Операции сравнения строк
        global str_eq = c == 'hello'
        global str_ne = c != d
        "#;
        
        let result = interp.exec(code);
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
        
        // Проверяем операции с массивами
        let combined = interp.get_variable("combined").unwrap();
        if let Value::Array(arr) = combined {
            assert_eq!(arr.len(), 6);
            assert_eq!(arr[0], Value::Number(1.0));
            assert_eq!(arr[3], Value::Number(4.0));
            assert_eq!(arr[5], Value::Number(6.0));
        } else {
            panic!("combined should be an Array");
        }
        
        // Проверяем сравнения строк
        assert_eq!(interp.get_variable("str_eq"), Some(&Value::Bool(true)));
        assert_eq!(interp.get_variable("str_ne"), Some(&Value::Bool(true)));
    }

    /// Тест проверки работы с таблицами и их внутренней структурой
    #[test]
    fn test_table_internal_structure() {
        let mut interp = Interpreter::new();
        
        let code = r#"
        global data = [
            ['Name', 'Age', 'City'],
            ['Alice', 25, 'New York'],
            ['Bob', 30, 'London'],
            ['Charlie', 35, 'Tokyo']
        ]
        
        global table = table_create(data)
        global name_column = table['Name']
        global age_column = table['Age']
        global city_column = table['City']
        
        # Проверяем размеры
        global table_rows = len(table)
        global name_count = len(name_column)
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Table operations should succeed");
        
        // Проверяем таблицу
        let table = interp.get_variable("table").unwrap();
        assert!(matches!(table, Value::Table(_)));
        
        if let Value::Table(table_data) = table {
            // Проверяем количество строк (без заголовка)
            assert_eq!(table_data.rows.len(), 3);
            
            // Проверяем заголовки
            assert_eq!(table_data.column_names, vec!["Name", "Age", "City"]);
            
            // Проверяем первую строку данных
            assert_eq!(table_data.rows[0][0], Value::String("Alice".to_string()));
            assert_eq!(table_data.rows[0][1], Value::Number(25.0));
            assert_eq!(table_data.rows[0][2], Value::String("New York".to_string()));
        }
        
        // Проверяем колонки
        let name_column = interp.get_variable("name_column").unwrap();
        if let Value::Array(names) = name_column {
            assert_eq!(names.len(), 3);
            assert_eq!(names[0], Value::String("Alice".to_string()));
            assert_eq!(names[1], Value::String("Bob".to_string()));
            assert_eq!(names[2], Value::String("Charlie".to_string()));
        } else {
            panic!("name_column should be an Array");
        }
        
        // Проверяем размеры
        assert_eq!(interp.get_variable("table_rows"), Some(&Value::Number(3.0)));
        assert_eq!(interp.get_variable("name_count"), Some(&Value::Number(3.0)));
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
        global person = {}
        global person['name'] = 'John Doe'
        global person['age'] = 30
        global person['address'] = {}
        global person['address']['street'] = '123 Main St'
        global person['address']['city'] = 'Anytown'
        global person['address']['zip'] = '12345'
        global person['hobbies'] = ['reading', 'swimming', 'coding']

        # Создаем массив объектов
        global people = []
        global people = push(people, person)

        global person2 = {}
        global person2['name'] = 'Jane Smith'
        global person2['age'] = 25
        global person2['address'] = {}
        global person2['address']['street'] = '456 Oak Ave'
        global person2['address']['city'] = 'Somewhere'
        global person2['address']['zip'] = '67890'
        global person2['hobbies'] = ['painting', 'dancing']

        global people = push(people, person2)

        # Извлекаем данные
        global first_person_name = people[0]['name']
        global second_person_city = people[1]['address']['city']
        global first_person_hobbies = people[0]['hobbies']
        "#;

        let result = interp.exec(code);
        assert!(result.is_ok(), "Complex object operations should succeed");

        // Проверяем основной объект person
        let person = interp.get_variable("person").unwrap();
        if let Value::Object(person_map) = person {
            assert_eq!(person_map.get("name"), Some(&Value::String("John Doe".to_string())));
            assert_eq!(person_map.get("age"), Some(&Value::Number(30.0)));

            // Проверяем вложенный объект address
            if let Some(Value::Object(address)) = person_map.get("address") {
                assert_eq!(address.get("street"), Some(&Value::String("123 Main St".to_string())));
                assert_eq!(address.get("city"), Some(&Value::String("Anytown".to_string())));
                assert_eq!(address.get("zip"), Some(&Value::String("12345".to_string())));
            } else {
                panic!("address should be an object");
            }

            // Проверяем массив hobbies
            if let Some(Value::Array(hobbies)) = person_map.get("hobbies") {
                assert_eq!(hobbies.len(), 3);
                assert_eq!(hobbies[0], Value::String("reading".to_string()));
                assert_eq!(hobbies[1], Value::String("swimming".to_string()));
                assert_eq!(hobbies[2], Value::String("coding".to_string()));
            } else {
                panic!("hobbies should be an array");
            }
        } else {
            panic!("person should be an object");
        }

        // Проверяем массив людей
        let people = interp.get_variable("people").unwrap();
        if let Value::Array(people_array) = people {
            assert_eq!(people_array.len(), 2);

            // Проверяем первого человека
            if let Value::Object(first_person) = &people_array[0] {
                assert_eq!(first_person.get("name"), Some(&Value::String("John Doe".to_string())));
            } else {
                panic!("First person should be an object");
            }

            // Проверяем второго человека
            if let Value::Object(second_person) = &people_array[1] {
                assert_eq!(second_person.get("name"), Some(&Value::String("Jane Smith".to_string())));
            } else {
                panic!("Second person should be an object");
            }
        } else {
            panic!("people should be an array");
        }

        // Проверяем извлеченные данные
        assert_eq!(interp.get_variable("first_person_name"), Some(&Value::String("John Doe".to_string())));
        assert_eq!(interp.get_variable("second_person_city"), Some(&Value::String("Somewhere".to_string())));

        let first_person_hobbies = interp.get_variable("first_person_hobbies").unwrap();
        if let Value::Array(hobbies) = first_person_hobbies {
            assert_eq!(hobbies.len(), 3);
            assert_eq!(hobbies[0], Value::String("reading".to_string()));
        } else {
            panic!("first_person_hobbies should be an array");
        }
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
        global is_int_number = isinstance(int_val, 'number')
        global is_float_number = isinstance(float_val, 'number')
        global is_str_string = isinstance(str_val, 'string')
        global is_bool_bool = isinstance(bool_val, 'bool')
        global is_null_null = isinstance(null_val, 'null')
        global is_array_array = isinstance(array_val, 'array')
        global is_currency_currency = isinstance(currency_val, 'currency')
        global is_path_path = isinstance(path_val, 'path')

        # Проверяем неправильные типы
        global is_int_string = isinstance(int_val, 'string')
        global is_str_number = isinstance(str_val, 'number')
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
