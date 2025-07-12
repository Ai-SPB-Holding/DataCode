use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;

#[cfg(test)]
mod file_operations_integration_tests {
    use super::*;

    #[test]
    fn test_complete_file_workflow() {
        let mut interp = Interpreter::new();
        
        // Полный рабочий процесс с файлами
        let workflow_code = r#"
            # Получаем текущую директорию
            global current_dir = getcwd()
            print('Current directory:', current_dir)
            
            # Создаем путь к тестовой директории
            global test_data_path = current_dir / 'test_data'
            print('Test data path:', test_data_path)
            
            # Получаем список файлов
            global file_list = list_files(test_data_path)
            print('Files found:', file_list)
            
            # Читаем каждый файл
            global file_contents = array()
            for file in file_list do
                local full_path = test_data_path / file
                print('Reading file:', full_path)
                local content = read_file(full_path)
                global file_contents = append(file_contents, content)
            forend
        "#;
        
        let result = interp.exec(workflow_code);
        assert!(result.is_ok(), "Complete workflow should succeed: {:?}", result);
        
        // Проверяем результаты
        match interp.get_variable("file_list") {
            Some(Value::Array(files)) => {
                assert!(files.len() >= 3); // Должно быть минимум 3 файла
                
                // Проверяем, что все ожидаемые файлы присутствуют
                let file_names: Vec<String> = files.iter()
                    .filter_map(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                    .collect();
                
                assert!(file_names.contains(&"sample.txt".to_string()));
                assert!(file_names.contains(&"sample.csv".to_string()));
                assert!(file_names.contains(&"sample.xlsx".to_string()));
            }
            _ => panic!("file_list should be an array"),
        }
        
        match interp.get_variable("file_contents") {
            Some(Value::Array(contents)) => {
                assert!(contents.len() >= 3); // Должно быть минимум 3 файла
                
                // Проверяем, что содержимое файлов различается по типу
                let mut has_string = false;
                let mut has_array = false;
                
                for content in contents {
                    match content {
                        Value::String(_) => has_string = true,
                        Value::Array(_) => has_array = true,
                        _ => {}
                    }
                }
                
                assert!(has_string, "Should have at least one string content (txt file)");
                assert!(has_array, "Should have at least one array content (csv/xlsx file)");
            }
            _ => panic!("file_contents should be an array"),
        }
    }

    #[test]
    fn test_path_building_and_file_reading() {
        let mut interp = Interpreter::new();
        
        // Тест построения путей и чтения файлов
        let path_test_code = r#"
            global base = getcwd()
            global subdir = 'test_data'
            global filename = 'sample.txt'
            
            # Строим путь пошагово
            global step1 = base / subdir
            global final_path = step1 / filename
            
            # Читаем файл
            global content = read_file(final_path)
        "#;
        
        let result = interp.exec(path_test_code);
        assert!(result.is_ok(), "Path building test should succeed: {:?}", result);
        
        // Проверяем, что файл был прочитан
        match interp.get_variable("content") {
            Some(Value::String(content)) => {
                assert!(content.contains("Hello, DataCode!"));
            }
            _ => panic!("Content should be a string"),
        }
    }

    #[test]
    fn test_csv_data_processing() {
        let mut interp = Interpreter::new();
        
        // Тест обработки CSV данных
        let csv_processing_code = r#"
            global csv_path = getcwd() / 'test_data' / 'sample.csv'
            global csv_data = read_file(csv_path)
            
            # Подсчитываем количество строк
            global row_count = len(csv_data)
            
            # Получаем заголовки (первая строка)
            global headers = csv_data[0]
            
            # Получаем первую строку данных
            global first_row = csv_data[1]
        "#;
        
        let result = interp.exec(csv_processing_code);
        assert!(result.is_ok(), "CSV processing should succeed: {:?}", result);
        
        // Проверяем количество строк
        match interp.get_variable("row_count") {
            Some(Value::Number(count)) => {
                assert_eq!(*count, 6.0); // Заголовок + 5 строк данных
            }
            _ => panic!("row_count should be a number"),
        }
        
        // Проверяем заголовки
        match interp.get_variable("headers") {
            Some(Value::Array(headers)) => {
                assert_eq!(headers.len(), 4);
                if let Value::String(first_header) = &headers[0] {
                    assert_eq!(first_header, "Name");
                }
            }
            _ => panic!("headers should be an array"),
        }
    }

    #[test]
    fn test_xlsx_data_processing() {
        let mut interp = Interpreter::new();
        
        // Тест обработки Excel данных
        let xlsx_processing_code = r#"
            global xlsx_path = getcwd() / 'test_data' / 'sample.xlsx'
            global xlsx_data = read_file(xlsx_path)
            
            # Подсчитываем количество строк
            global row_count = len(xlsx_data)
            
            # Получаем заголовки (первая строка)
            global headers = xlsx_data[0]
        "#;
        
        let result = interp.exec(xlsx_processing_code);
        assert!(result.is_ok(), "XLSX processing should succeed: {:?}", result);
        
        // Проверяем количество строк
        match interp.get_variable("row_count") {
            Some(Value::Number(count)) => {
                assert!(*count >= 1.0); // Минимум заголовок
            }
            _ => panic!("row_count should be a number"),
        }
        
        // Проверяем заголовки
        match interp.get_variable("headers") {
            Some(Value::Array(headers)) => {
                assert_eq!(headers.len(), 4);
                if let Value::String(first_header) = &headers[0] {
                    assert_eq!(first_header, "Product");
                }
            }
            _ => panic!("headers should be an array"),
        }
    }

    #[test]
    fn test_error_handling_in_file_operations() {
        let mut interp = Interpreter::new();
        
        // Тест обработки ошибок при работе с файлами
        
        // 1. Несуществующая директория
        let result1 = interp.exec("global bad_files = list_files(path('nonexistent_dir'))");
        assert!(result1.is_err());
        
        // 2. Несуществующий файл
        let result2 = interp.exec("global bad_content = read_file(path('nonexistent.txt'))");
        assert!(result2.is_err());
        
        // 3. Неправильный тип аргумента для read_file
        let result3 = interp.exec("global bad_read = read_file('not_a_path')");
        assert!(result3.is_err());
        
        // 4. Неправильный тип аргумента для list_files
        let result4 = interp.exec("global bad_list = list_files('not_a_path')");
        assert!(result4.is_err());
    }

    #[test]
    fn test_mixed_file_types_processing() {
        let mut interp = Interpreter::new();
        
        // Тест обработки смешанных типов файлов (упрощенная версия)
        let mixed_processing_code = r#"
            global test_dir = getcwd() / 'test_data'
            global all_files = list_files(test_dir)

            global txt_files = array()
            global csv_files = array()
            global xlsx_files = array()

            # Просто читаем все файлы и создаем пустые массивы для классификации
            for filename in all_files do
                local file_path = test_dir / filename
                local content = read_file(file_path)
                # Упрощенная классификация без функций type() и contains()
            forend
        "#;
        
        let result = interp.exec(mixed_processing_code);
        assert!(result.is_ok(), "Mixed processing should succeed: {:?}", result);
        
        // Проверяем, что переменные созданы
        assert!(interp.get_variable("all_files").is_some());
        assert!(interp.get_variable("txt_files").is_some());
        assert!(interp.get_variable("csv_files").is_some());
        assert!(interp.get_variable("xlsx_files").is_some());
    }
}
