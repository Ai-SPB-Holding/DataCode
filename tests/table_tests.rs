use data_code::interpreter::Interpreter;
use data_code::value::{Value, DataType};

#[cfg(test)]
mod table_tests {
    use super::*;

    #[test]
    fn test_table_creation_from_arrays() {
        let mut interp = Interpreter::new();

        // Создаем тестовые данные по частям
        let result1 = interp.exec("global data = [[1, 'Alice', true], [2, 'Bob', false], [3, 'Charlie', true]]");
        assert!(result1.is_ok(), "Failed to create data array: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'active']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);
        
        // Проверяем, что таблица создана
        let table_value = interp.get_variable("my_table").unwrap();
        match table_value {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), 3);
                assert_eq!(table.columns.len(), 3);
                assert_eq!(table.column_names, vec!["id", "name", "active"]);
            }
            _ => panic!("Expected Table, got {:?}", table_value),
        }
    }

    #[test]
    fn test_table_creation_from_objects() {
        let mut interp = Interpreter::new();

        // Create array data that represents table rows
        interp.set_variable("data".to_string(), Value::Array(vec![
            Value::Array(vec![
                Value::String("Alice".to_string()),
                Value::Number(25.0),
                Value::String("New York".to_string())
            ]),
            Value::Array(vec![
                Value::String("Bob".to_string()),
                Value::Number(30.0),
                Value::String("London".to_string())
            ])
        ]), true);

        // Create headers
        interp.set_variable("headers".to_string(), Value::Array(vec![
            Value::String("name".to_string()),
            Value::String("age".to_string()),
            Value::String("city".to_string())
        ]), true);

        let result = interp.exec("global my_table = table(data, headers)");
        assert!(result.is_ok(), "Failed to create table from arrays: {:?}", result);

        let table_value = interp.get_variable("my_table").unwrap();
        match table_value {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), 2);
                assert_eq!(table.column_names.len(), 3);
                assert!(table.column_names.contains(&"name".to_string()));
                assert!(table.column_names.contains(&"age".to_string()));
                assert!(table.column_names.contains(&"city".to_string()));
            }
            _ => panic!("Expected Table, got {:?}", table_value),
        }
    }

    #[test]
    fn test_table_type_inference() {
        let mut interp = Interpreter::new();

        // Создаем таблицу с разными типами данных по частям
        let result1 = interp.exec("global data = [[1, 'Alice', 25.5, true], [2, 'Bob', 30, false], ['3', 'Charlie', '25', true]]");
        assert!(result1.is_ok(), "Failed to create data array: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age', 'active']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);
        
        let table_value = interp.get_variable("my_table").unwrap();
        match table_value {
            Value::Table(table) => {
                // Проверяем типизацию колонок
                let id_column = &table.columns[0];
                let name_column = &table.columns[1];
                let age_column = &table.columns[2];
                let active_column = &table.columns[3];
                
                // id колонка должна быть смешанной (числа и строки)
                assert_eq!(id_column.name, "id");
                assert!(id_column.type_counts.len() > 1, "ID column should have mixed types");
                
                // name колонка должна быть строковой
                assert_eq!(name_column.name, "name");
                assert_eq!(name_column.inferred_type, DataType::String);
                
                // age колонка должна быть смешанной (числа и строки)
                assert_eq!(age_column.name, "age");
                assert!(age_column.type_counts.len() > 1, "Age column should have mixed types");
                
                // active колонка должна быть булевой
                assert_eq!(active_column.name, "active");
                assert_eq!(active_column.inferred_type, DataType::Bool);
            }
            _ => panic!("Expected Table, got {:?}", table_value),
        }
    }

    #[test]
    fn test_show_table_function() {
        let mut interp = Interpreter::new();

        // Создаем простую таблицу по частям
        let result1 = interp.exec("global data = [[1, 'Alice'], [2, 'Bob']]");
        assert!(result1.is_ok(), "Failed to create data: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Тестируем show_table (проверяем, что не падает)
        let result = interp.exec("show_table(my_table)");
        assert!(result.is_ok(), "show_table failed: {:?}", result);
    }

    #[test]
    fn test_table_info_function() {
        let mut interp = Interpreter::new();

        // Создаем таблицу по частям
        let result1 = interp.exec("global data = [[1, 'Alice'], [2, 'Bob'], [3, 'Charlie']]");
        assert!(result1.is_ok(), "Failed to create data: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Тестируем table_info
        let result = interp.exec("table_info(my_table)");
        assert!(result.is_ok(), "table_info failed: {:?}", result);
    }

    #[test]
    fn test_table_head_function() {
        let mut interp = Interpreter::new();

        // Создаем таблицу с несколькими строками по частям
        let result1 = interp.exec("global data = [[1, 'Alice'], [2, 'Bob'], [3, 'Charlie'], [4, 'David'], [5, 'Eve']]");
        assert!(result1.is_ok(), "Failed to create data: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Тестируем table_head с параметром
        let result = interp.exec("table_head(my_table, 3)");
        assert!(result.is_ok(), "table_head failed: {:?}", result);

        // Тестируем table_head без параметра (по умолчанию 5)
        let result = interp.exec("table_head(my_table)");
        assert!(result.is_ok(), "table_head without parameter failed: {:?}", result);
    }

    #[test]
    fn test_table_tail_function() {
        let mut interp = Interpreter::new();

        // Создаем таблицу по частям
        let result1 = interp.exec("global data = [[1, 'Alice'], [2, 'Bob'], [3, 'Charlie'], [4, 'David'], [5, 'Eve']]");
        assert!(result1.is_ok(), "Failed to create data: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Тестируем table_tail
        let result = interp.exec("table_tail(my_table, 2)");
        assert!(result.is_ok(), "table_tail failed: {:?}", result);
    }

    #[test]
    fn test_table_select_function() {
        let mut interp = Interpreter::new();

        // Создаем таблицу с несколькими колонками по частям
        let result1 = interp.exec("global data = [[1, 'Alice', 25, 'Engineer'], [2, 'Bob', 30, 'Designer'], [3, 'Charlie', 35, 'Manager']]");
        assert!(result1.is_ok(), "Failed to create data: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age', 'job']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Выбираем только некоторые колонки
        let result = interp.exec("global selected = table_select(my_table, ['name', 'age'])");
        assert!(result.is_ok(), "table_select failed: {:?}", result);
        
        // Проверяем результат
        let selected_value = interp.get_variable("selected").unwrap();
        match selected_value {
            Value::Table(table) => {
                assert_eq!(table.columns.len(), 2);
                assert_eq!(table.column_names, vec!["name", "age"]);
                assert_eq!(table.rows.len(), 3);
            }
            _ => panic!("Expected Table, got {:?}", selected_value),
        }
    }

    #[test]
    fn test_table_sort_function() {
        let mut interp = Interpreter::new();

        // Создаем таблицу для сортировки по частям
        let result1 = interp.exec("global data = [[3, 'Charlie'], [1, 'Alice'], [2, 'Bob']]");
        assert!(result1.is_ok(), "Failed to create data: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Сортируем по id
        let result = interp.exec("global sorted = table_sort(my_table, 'id')");
        assert!(result.is_ok(), "table_sort failed: {:?}", result);
        
        // Проверяем результат
        let sorted_value = interp.get_variable("sorted").unwrap();
        match sorted_value {
            Value::Table(table) => {
                assert_eq!(table.rows.len(), 3);
                // Первая строка должна содержать id=1
                if let Some(Value::Number(first_id)) = table.rows[0].get(0) {
                    assert_eq!(*first_id, 1.0);
                } else {
                    panic!("Expected first row to have id=1");
                }
            }
            _ => panic!("Expected Table, got {:?}", sorted_value),
        }
    }
}
