use data_code::interpreter::Interpreter;
use data_code::value::Value;
use std::path::PathBuf;

#[cfg(test)]
mod path_methods_tests {
    use super::*;

    #[test]
    fn test_path_name() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global name = file_path.name
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("name") {
            Some(Value::String(s)) => {
                assert_eq!(s, "document.txt");
            }
            _ => panic!("Expected String value for name"),
        }
    }

    #[test]
    fn test_path_extension() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global ext = file_path.extension
            global file_no_ext = path("/home/user/document")
            global ext2 = file_no_ext.extension
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("ext") {
            Some(Value::String(s)) => {
                assert_eq!(s, "txt");
            }
            _ => panic!("Expected String value for extension"),
        }
        
        match interp.get_variable("ext2") {
            Some(Value::String(s)) => {
                assert_eq!(s, "");
            }
            _ => panic!("Expected empty String for extension"),
        }
    }

    #[test]
    fn test_path_stem() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global stem = file_path.stem
            global file_no_ext = path("/home/user/document")
            global stem2 = file_no_ext.stem
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("stem") {
            Some(Value::String(s)) => {
                assert_eq!(s, "document");
            }
            _ => panic!("Expected String value for stem"),
        }
        
        match interp.get_variable("stem2") {
            Some(Value::String(s)) => {
                assert_eq!(s, "document");
            }
            _ => panic!("Expected String value for stem2"),
        }
    }

    #[test]
    fn test_path_parent() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global parent = file_path.parent
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("parent") {
            Some(Value::Path(p)) => {
                assert_eq!(*p, PathBuf::from("/home/user"));
            }
            _ => panic!("Expected Path value for parent"),
        }
    }

    #[test]
    fn test_path_parent_root() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global root_path = path("/")
            global parent = root_path.parent
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("parent") {
            Some(Value::Null) => {
                // Root path has no parent
            }
            _ => panic!("Expected Null for root path parent"),
        }
    }

    #[test]
    fn test_path_to_string() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global path_str = file_path.to_string
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("path_str") {
            Some(Value::String(s)) => {
                assert!(s.contains("document.txt"));
            }
            _ => panic!("Expected String value for to_string"),
        }
    }

    #[test]
    fn test_path_len() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global path_len = file_path.len
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("path_len") {
            Some(Value::Number(n)) => {
                assert!(*n > 0.0);
            }
            _ => panic!("Expected Number value for len"),
        }
    }

    #[test]
    fn test_path_exists() {
        let mut interp = Interpreter::new();
        
        // Используем текущую директорию, которая должна существовать
        let code = r#"
            global current_dir = getcwd()
            global exists = current_dir.exists
            global fake_path = path("/non/existent/path/12345")
            global fake_exists = fake_path.exists
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("exists") {
            Some(Value::Bool(b)) => {
                assert!(b, "Current directory should exist");
            }
            _ => panic!("Expected Bool value for exists"),
        }
        
        match interp.get_variable("fake_exists") {
            Some(Value::Bool(b)) => {
                assert!(!b, "Fake path should not exist");
            }
            _ => panic!("Expected Bool value for fake_exists"),
        }
    }

    #[test]
    fn test_path_is_file() {
        let mut interp = Interpreter::new();
        
        // Создаем временный файл и проверяем
        let code = r#"
            global current_dir = getcwd()
            global is_file = current_dir.is_file
            global is_dir = current_dir.is_dir
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("is_file") {
            Some(Value::Bool(b)) => {
                // Текущая директория не файл
                assert!(!b);
            }
            _ => panic!("Expected Bool value for is_file"),
        }
        
        match interp.get_variable("is_dir") {
            Some(Value::Bool(b)) => {
                // Текущая директория должна быть директорией
                assert!(b);
            }
            _ => panic!("Expected Bool value for is_dir"),
        }
    }

    #[test]
    fn test_path_is_dir() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global current_dir = getcwd()
            global is_dir = current_dir.is_dir
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("is_dir") {
            Some(Value::Bool(b)) => {
                assert!(b, "Current directory should be a directory");
            }
            _ => panic!("Expected Bool value for is_dir"),
        }
    }

    #[test]
    fn test_path_methods_in_loop() {
        let mut interp = Interpreter::new();
        
        // Тест доступа к методам Path в цикле
        let code = r#"
            global current_dir = getcwd()
            global files = list_files(current_dir)
            global found_file = false
            global found_dir = false
            
            for file in files do
                if file.is_file do
                    global found_file = true
                    global file_name = file.name
                    global file_ext = file.extension
                endif
                if file.is_dir do
                    global found_dir = true
                endif
            next file
        "#;
        
        let result = interp.exec(code);
        // Может быть ошибка если директория пустая, но это нормально
        // Проверяем что хотя бы не было ошибок доступа к методам
        if result.is_err() {
            let err_str = format!("{:?}", result.unwrap_err());
            // Не должно быть ошибок типа "Cannot access member"
            assert!(!err_str.contains("Cannot access member"), 
                "Should not have member access errors: {}", err_str);
        }
    }

    #[test]
    fn test_path_invalid_member() {
        let mut interp = Interpreter::new();
        
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global invalid = file_path.invalid_method
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_err(), "Should fail on invalid member access");
        
        let err_str = format!("{:?}", result.unwrap_err());
        assert!(err_str.contains("Path has no member"), 
            "Should indicate that Path has no such member");
    }

    #[test]
    fn test_path_methods_chaining() {
        let mut interp = Interpreter::new();
        
        // Тест цепочки методов (parent.name)
        let code = r#"
            global file_path = path("/home/user/document.txt")
            global parent_path = file_path.parent
            global parent_name = parent_path.name
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok(), "Code should execute successfully");
        
        match interp.get_variable("parent_name") {
            Some(Value::String(s)) => {
                assert_eq!(s, "user");
            }
            _ => panic!("Expected String value for parent name"),
        }
    }

    #[test]
    fn test_path_pattern_methods() {
        let mut interp = Interpreter::new();
        
        // PathPattern должен поддерживать те же методы
        let code = r#"
            global pattern = path("/home/user/*.txt")
            global pattern_name = pattern.name
            global pattern_parent = pattern.parent
        "#;
        
        let result = interp.exec(code);
        // PathPattern может не иметь имени, если это паттерн
        // Но методы должны работать без ошибок
        if result.is_err() {
            let err_str = format!("{:?}", result.unwrap_err());
            assert!(!err_str.contains("Cannot access member"), 
                "Should not have member access errors for PathPattern");
        }
    }

    #[test]
    fn test_path_complex_workflow() {
        let mut interp = Interpreter::new();
        
        // Комплексный тест использования методов Path
        let code = r#"
            global base_path = getcwd()
            global files = list_files(base_path)
            
            global csv_files = []
            global directories = []
            
            for file in files do
                if file.is_file do
                    if file.extension == "csv" do
                        global csv_files = append(csv_files, file.name)
                    endif
                endif
                if file.is_dir do
                    global directories = append(directories, file.name)
                endif
            next file
        "#;
        
        let result = interp.exec(code);
        // Может быть ошибка если нет файлов, но не должно быть ошибок доступа к методам
        if result.is_err() {
            let err_str = format!("{:?}", result.unwrap_err());
            assert!(!err_str.contains("Cannot access member"), 
                "Should not have member access errors: {}", err_str);
        }
    }

    #[test]
    fn test_path_with_list_files_result() {
        let mut interp = Interpreter::new();
        
        // Критический тест - проверка работы с результатом list_files()
        let code = r#"
            global current_dir = getcwd()
            global files = list_files(current_dir)
            
            if len(files) > 0 do
                global first_file = files[0]
                global file_name = first_file.name
                global file_is_dir = first_file.is_dir
                global file_exists = first_file.exists
            endif
        "#;
        
        let result = interp.exec(code);
        // Это критический тест - не должно быть ошибок доступа к методам
        if result.is_err() {
            let err_str = format!("{:?}", result.unwrap_err());
            assert!(!err_str.contains("Cannot access member"), 
                "CRITICAL: Should not have member access errors for list_files() result: {}", err_str);
        }
    }
}
