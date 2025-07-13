use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;
use std::path::PathBuf;

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
                assert_eq!(table.rows.len(), 5);

                // Проверяем заголовки
                assert_eq!(table.column_names.len(), 4); // Name, Age, City, Salary
                assert_eq!(table.column_names[0], "Name");
                assert_eq!(table.column_names[1], "Age");
                assert_eq!(table.column_names[2], "City");
                assert_eq!(table.column_names[3], "Salary");

                // Проверяем первую строку данных
                let first_data_row = &table.rows[0];
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
                assert!(table.rows.len() > 0);

                // Проверяем заголовки
                assert_eq!(table.column_names.len(), 4); // Product, Price, Quantity, Category
                assert_eq!(table.column_names[0], "Product");
                assert_eq!(table.column_names[1], "Price");
                assert_eq!(table.column_names[2], "Quantity");
                assert_eq!(table.column_names[3], "Category");

                // Проверяем первую строку данных
                if table.rows.len() > 0 {
                    let first_data_row = &table.rows[0];
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
        match result.unwrap_err() {
            DataCodeError::FunctionError { name, .. } => {
                assert_eq!(name, "read_file");
            }
            _ => panic!("Expected FunctionError for wrong argument count"),
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
}
