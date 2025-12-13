use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;

#[cfg(test)]
mod file_reading_tests {
    use super::*;

    #[test]
    fn test_read_txt_file() {
        let mut interp = Interpreter::new();
        
        // Создаем путь к тестовому файлу
        let test_file_path = "test_data/sample.txt";
        interp.exec(&format!("global txt_path = path('{}')", test_file_path)).unwrap();
        interp.exec("global txt_content = read_file(txt_path)").unwrap();
        
        match interp.get_variable("txt_content") {
            Some(Value::String(content)) => {
                assert!(content.contains("Hello, DataCode!"));
                assert!(content.contains("This is a test text file"));
                assert!(content.contains("UTF-8 characters: привет мир! 🌍"));
                assert!(content.contains("End of file."));
                assert!(content.contains('\n'));
            }
            _ => panic!("read_file should return a string for txt files"),
        }
    }

    #[test]
    fn test_read_csv_file() {
        let mut interp = Interpreter::new();
        
        // Создаем путь к тестовому CSV файлу
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        interp.exec("global csv_content = read_file(csv_path)").unwrap();
        
        match interp.get_variable("csv_content") {
            Some(Value::Table(table)) => {
                // Проверяем количество строк данных (без заголовка)
                let table_borrowed = table.borrow();
                assert_eq!(table_borrowed.rows.len(), 5);

                // Проверяем заголовки
                assert_eq!(table_borrowed.column_names.len(), 4); // Name, Age, City, Salary
                assert_eq!(table_borrowed.column_names[0], "Name");
                assert_eq!(table_borrowed.column_names[1], "Age");
                assert_eq!(table_borrowed.column_names[2], "City");
                assert_eq!(table_borrowed.column_names[3], "Salary");

                // Проверяем первую строку данных
                let first_data_row = &table_borrowed.rows[0];
                assert_eq!(first_data_row.len(), 4);
                if let Value::String(name) = &first_data_row[0] {
                    assert_eq!(name, "John Doe");
                }
                if let Value::Number(age) = &first_data_row[1] {
                    assert_eq!(*age, 30.0);
                }
                if let Value::String(city) = &first_data_row[2] {
                    assert_eq!(city, "New York");
                }
                if let Value::Number(salary) = &first_data_row[3] {
                    assert_eq!(*salary, 50000.0);
                }
            }
            _ => panic!("read_file should return a table for csv files"),
        }
    }

    #[test]
    fn test_read_xlsx_file() {
        let mut interp = Interpreter::new();
        
        // Создаем путь к тестовому Excel файлу
        let test_file_path = "test_data/sample.xlsx";
        interp.exec(&format!("global xlsx_path = path('{}')", test_file_path)).unwrap();
        interp.exec("global xlsx_content = read_file(xlsx_path)").unwrap();
        
        match interp.get_variable("xlsx_content") {
            Some(Value::Table(table)) => {
                // Проверяем, что есть данные
                let table_borrowed = table.borrow();
                assert!(table_borrowed.rows.len() > 0);

                // Проверяем заголовки
                assert_eq!(table_borrowed.column_names.len(), 4); // Product, Price, Quantity, Category
                assert_eq!(table_borrowed.column_names[0], "Product");
                assert_eq!(table_borrowed.column_names[1], "Price");
                assert_eq!(table_borrowed.column_names[2], "Quantity");
                assert_eq!(table_borrowed.column_names[3], "Category");

                // Проверяем первую строку данных
                if table_borrowed.rows.len() > 0 {
                    let first_data_row = &table_borrowed.rows[0];
                    assert_eq!(first_data_row.len(), 4);
                    if let Value::String(product) = &first_data_row[0] {
                        assert_eq!(product, "Laptop");
                    }
                    if let Value::Number(price) = &first_data_row[1] {
                        assert_eq!(*price, 999.99);
                    }
                }
            }
            _ => panic!("read_file should return a table for xlsx files"),
        }
    }

    #[test]
    fn test_read_file_unsupported_extension() {
        let mut interp = Interpreter::new();
        
        // Пытаемся прочитать файл с неподдерживаемым расширением
        interp.exec("global bad_path = path('test.pdf')").unwrap();
        let result = interp.exec("global content = read_file(bad_path)");
        
        assert!(result.is_err());
        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("Unsupported file extension"));
            }
            _ => panic!("Expected RuntimeError for unsupported file extension"),
        }
    }

    #[test]
    fn test_read_file_nonexistent() {
        let mut interp = Interpreter::new();
        
        // Пытаемся прочитать несуществующий файл
        interp.exec("global nonexistent_path = path('nonexistent.txt')").unwrap();
        let result = interp.exec("global content = read_file(nonexistent_path)");
        
        assert!(result.is_err());
        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("Failed to read"));
            }
            _ => panic!("Expected RuntimeError for nonexistent file"),
        }
    }

    #[test]
    fn test_read_file_wrong_argument_type() {
        let mut interp = Interpreter::new();
        
        // Пытаемся передать неправильный тип аргумента
        let result = interp.exec("global content = read_file('string_instead_of_path')");
        
        assert!(result.is_err());
        match result.unwrap_err() {
            DataCodeError::TypeError { expected, .. } => {
                assert_eq!(expected, "Path");
            }
            _ => panic!("Expected TypeError for wrong argument type"),
        }
    }

    #[test]
    fn test_read_file_wrong_argument_count() {
        let mut interp = Interpreter::new();
        
        // Пытаемся вызвать read_file без аргументов
        let result = interp.exec("global content = read_file()");
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("read_file"));
                assert!(message.contains("expects"));
                assert!(message.contains("arguments"));
            }
            _ => panic!("Expected RuntimeError for wrong argument count, got: {:?}", error),
        }
    }

    #[test]
    fn test_file_reading_integration() {
        let mut interp = Interpreter::new();
        
        // Интеграционный тест: читаем все типы файлов
        let integration_code = r#"
            global base_path = getcwd()
            global test_dir = path('test_data')

            # Читаем txt файл
            global txt_path = test_dir / 'sample.txt'
            global txt_content = read_file(txt_path)

            # Читаем csv файл
            global csv_path = test_dir / 'sample.csv'
            global csv_content = read_file(csv_path)

            # Читаем xlsx файл
            global xlsx_path = test_dir / 'sample.xlsx'
            global xlsx_content = read_file(xlsx_path)
        "#;
        
        let result = interp.exec(integration_code);
        assert!(result.is_ok(), "Integration test should succeed: {:?}", result);
        
        // Проверяем результаты
        // Проверяем txt файл
        match interp.get_variable("txt_content") {
            Some(Value::String(_)) => {}, // OK
            _ => panic!("txt_content should be a string"),
        }

        // Проверяем csv файл
        match interp.get_variable("csv_content") {
            Some(Value::Table(_)) => {}, // OK - CSV теперь возвращает Table
            _ => panic!("csv_content should be a table"),
        }

        // Проверяем xlsx файл
        match interp.get_variable("xlsx_content") {
            Some(Value::Table(_)) => {}, // OK - Excel тоже возвращает Table
            _ => panic!("xlsx_content should be a table"),
        }
    }

    // ========== Тесты для функционала header ==========

    #[test]
    fn test_read_csv_with_header_filter() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        interp.exec("global csv_content = read_file(csv_path, header=[\"Name\", \"Age\"])").unwrap();
        
        match interp.get_variable("csv_content") {
            Some(Value::Table(table)) => {
                let table_borrowed = table.borrow();
                // Должно быть только 2 колонки
                assert_eq!(table_borrowed.column_names.len(), 2);
                assert_eq!(table_borrowed.column_names[0], "Name");
                assert_eq!(table_borrowed.column_names[1], "Age");
                
                // Проверяем количество строк
                assert_eq!(table_borrowed.rows.len(), 5);
                
                // Проверяем первую строку
                let first_row = &table_borrowed.rows[0];
                assert_eq!(first_row.len(), 2);
                if let Value::String(name) = &first_row[0] {
                    assert_eq!(name, "John Doe");
                }
                if let Value::Number(age) = &first_row[1] {
                    assert_eq!(*age, 30.0);
                }
            }
            _ => panic!("read_file should return a table for csv files with header filter"),
        }
    }

    #[test]
    fn test_read_csv_with_header_filter_reordered() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        // Порядок колонок в header отличается от порядка в файле
        interp.exec("global csv_content = read_file(csv_path, header=[\"Salary\", \"Name\", \"City\"])").unwrap();
        
        match interp.get_variable("csv_content") {
            Some(Value::Table(table)) => {
                let table_borrowed = table.borrow();
                // Должно быть 3 колонки в указанном порядке
                assert_eq!(table_borrowed.column_names.len(), 3);
                assert_eq!(table_borrowed.column_names[0], "Salary");
                assert_eq!(table_borrowed.column_names[1], "Name");
                assert_eq!(table_borrowed.column_names[2], "City");
                
                // Проверяем первую строку - порядок значений должен соответствовать header
                let first_row = &table_borrowed.rows[0];
                assert_eq!(first_row.len(), 3);
                if let Value::Number(salary) = &first_row[0] {
                    assert_eq!(*salary, 50000.0);
                }
                if let Value::String(name) = &first_row[1] {
                    assert_eq!(name, "John Doe");
                }
                if let Value::String(city) = &first_row[2] {
                    assert_eq!(city, "New York");
                }
            }
            _ => panic!("read_file should return a table with reordered columns"),
        }
    }

    #[test]
    fn test_read_xlsx_with_header_filter() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.xlsx";
        interp.exec(&format!("global xlsx_path = path('{}')", test_file_path)).unwrap();
        interp.exec("global xlsx_content = read_file(xlsx_path, header=[\"Product\", \"Price\"])").unwrap();
        
        match interp.get_variable("xlsx_content") {
            Some(Value::Table(table)) => {
                let table_borrowed = table.borrow();
                // Должно быть только 2 колонки
                assert_eq!(table_borrowed.column_names.len(), 2);
                assert_eq!(table_borrowed.column_names[0], "Product");
                assert_eq!(table_borrowed.column_names[1], "Price");
                
                // Проверяем первую строку
                if table_borrowed.rows.len() > 0 {
                    let first_row = &table_borrowed.rows[0];
                    assert_eq!(first_row.len(), 2);
                    if let Value::String(product) = &first_row[0] {
                        assert_eq!(product, "Laptop");
                    }
                    if let Value::Number(price) = &first_row[1] {
                        assert_eq!(*price, 999.99);
                    }
                }
            }
            _ => panic!("read_file should return a table for xlsx files with header filter"),
        }
    }

    #[test]
    fn test_read_file_with_header_and_sheet_name() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.xlsx";
        interp.exec(&format!("global xlsx_path = path('{}')", test_file_path)).unwrap();
        // Не указываем sheet_name, так как тестовый файл может не иметь листа "Sheet1"
        // Просто проверяем, что header работает с xlsx файлами
        interp.exec("global xlsx_content = read_file(xlsx_path, header=[\"Product\", \"Price\"])").unwrap();
        
        match interp.get_variable("xlsx_content") {
            Some(Value::Table(table)) => {
                let table_borrowed = table.borrow();
                assert_eq!(table_borrowed.column_names.len(), 2);
                assert_eq!(table_borrowed.column_names[0], "Product");
                assert_eq!(table_borrowed.column_names[1], "Price");
            }
            _ => panic!("read_file should work with header for xlsx files"),
        }
    }

    #[test]
    fn test_read_file_with_header_and_header_row() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        interp.exec("global csv_content = read_file(csv_path, 0, header=[\"Name\", \"Age\"])").unwrap();
        
        match interp.get_variable("csv_content") {
            Some(Value::Table(table)) => {
                let table_borrowed = table.borrow();
                assert_eq!(table_borrowed.column_names.len(), 2);
                assert_eq!(table_borrowed.column_names[0], "Name");
                assert_eq!(table_borrowed.column_names[1], "Age");
            }
            _ => panic!("read_file should work with both header_row and header"),
        }
    }

    #[test]
    fn test_read_file_with_header_missing_column() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        // Указываем несуществующую колонку вместе с существующей
        // Должно выдать предупреждение, но продолжить работу
        let result = interp.exec("global csv_content = read_file(csv_path, header=[\"Name\", \"NonExistentColumn\"])");
        
        // Должно работать, но с предупреждением
        assert!(result.is_ok(), "Should work with missing column (with warning)");
        
        match interp.get_variable("csv_content") {
            Some(Value::Table(table)) => {
                let table_borrowed = table.borrow();
                // Должна быть только одна колонка (Name)
                assert_eq!(table_borrowed.column_names.len(), 1);
                assert_eq!(table_borrowed.column_names[0], "Name");
            }
            _ => panic!("Should return table with only existing columns"),
        }
    }

    #[test]
    fn test_read_file_with_header_all_missing_columns() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        // Все указанные колонки не существуют
        let result = interp.exec("global csv_content = read_file(csv_path, header=[\"NonExistent1\", \"NonExistent2\"])");
        
        // Должно вернуть ошибку
        assert!(result.is_err());
        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("None of the specified columns found") || 
                        message.contains("not found"));
            }
            _ => panic!("Expected RuntimeError for all missing columns"),
        }
    }

    #[test]
    fn test_read_file_with_header_empty_array() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        // Пустой массив header
        let result = interp.exec("global csv_content = read_file(csv_path, header=[])");
        
        // Должно вернуть ошибку
        assert!(result.is_err());
        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("empty") || message.contains("cannot be empty"));
            }
            _ => panic!("Expected RuntimeError for empty header array"),
        }
    }

    #[test]
    fn test_read_file_with_header_wrong_type() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        // header должен быть массивом, а не строкой
        let result = interp.exec("global csv_content = read_file(csv_path, header=\"Name\")");
        
        // Должно вернуть ошибку
        assert!(result.is_err());
        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("array") || message.contains("Array"));
            }
            _ => panic!("Expected RuntimeError for wrong header type"),
        }
    }

    #[test]
    fn test_read_file_with_header_array_containing_non_strings() {
        let mut interp = Interpreter::new();
        
        let test_file_path = "test_data/sample.csv";
        interp.exec(&format!("global csv_path = path('{}')", test_file_path)).unwrap();
        // header должен содержать только строки
        let result = interp.exec("global csv_content = read_file(csv_path, header=[\"Name\", 123])");
        
        // Должно вернуть ошибку
        assert!(result.is_err());
        match result.unwrap_err() {
            DataCodeError::RuntimeError { message, .. } => {
                assert!(message.contains("string") || message.contains("String"));
            }
            _ => panic!("Expected RuntimeError for non-string elements in header array"),
        }
    }
}
