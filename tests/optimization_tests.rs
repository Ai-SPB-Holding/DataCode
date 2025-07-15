// Тесты для оптимизированной системы значений DataCode
// Проверяет корректность работы Rc<RefCell<T>> и ленивой обработки данных

use data_code::interpreter::Interpreter;
use data_code::value::{Value, Table, LazyTable};
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(test)]
mod optimization_tests {
    use super::*;

    #[test]
    fn test_optimized_table_creation() {
        let mut interp = Interpreter::new();

        // Создаем тестовые данные
        let result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30], [3, 'Charlie', 35]]");
        assert!(result1.is_ok(), "Failed to create data array: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Проверяем, что таблица создана правильно
        let table_var = interp.get_variable("my_table");
        assert!(table_var.is_some(), "Table variable not found");
        
        if let Some(Value::Table(table_rc)) = table_var {
            let table_borrowed = table_rc.borrow();
            assert_eq!(table_borrowed.column_names, vec!["id", "name", "age"]);
            assert_eq!(table_borrowed.rows.len(), 3);
        } else {
            panic!("Expected table value");
        }
    }

    #[test]
    fn test_optimized_table_filter() {
        let mut interp = Interpreter::new();

        // Создаем тестовые данные
        let result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30], [3, 'Charlie', 35]]");
        assert!(result1.is_ok(), "Failed to create data array: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Тестируем фильтрацию
        let result4 = interp.exec("global filtered = table_filter(my_table, 'age > 25')");
        assert!(result4.is_ok(), "Failed to filter table: {:?}", result4);

        // Проверяем результат фильтрации
        let filtered_var = interp.get_variable("filtered");
        assert!(filtered_var.is_some(), "Filtered table variable not found");
        
        if let Some(Value::Table(table_rc)) = filtered_var {
            let table_borrowed = table_rc.borrow();
            assert_eq!(table_borrowed.rows.len(), 2); // Bob и Charlie
        } else {
            panic!("Expected filtered table value");
        }
    }

    #[test]
    fn test_optimized_table_where() {
        let mut interp = Interpreter::new();

        // Создаем тестовые данные
        let result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30], [3, 'Charlie', 35]]");
        assert!(result1.is_ok(), "Failed to create data array: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Тестируем WHERE фильтрацию
        let result4 = interp.exec("global filtered = table_where(my_table, 'age', '>', 25)");
        assert!(result4.is_ok(), "Failed to filter table with WHERE: {:?}", result4);

        // Проверяем результат фильтрации
        let filtered_var = interp.get_variable("filtered");
        assert!(filtered_var.is_some(), "Filtered table variable not found");
        
        if let Some(Value::Table(table_rc)) = filtered_var {
            let table_borrowed = table_rc.borrow();
            assert_eq!(table_borrowed.rows.len(), 2); // Bob и Charlie
        } else {
            panic!("Expected filtered table value");
        }
    }

    #[test]
    fn test_optimized_table_head() {
        let mut interp = Interpreter::new();

        // Создаем тестовые данные
        let result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30], [3, 'Charlie', 35], [4, 'David', 40]]");
        assert!(result1.is_ok(), "Failed to create data array: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Тестируем head операцию
        let result4 = interp.exec("global head_table = table_head(my_table, 2)");
        assert!(result4.is_ok(), "Failed to get table head: {:?}", result4);

        // Проверяем результат
        let head_var = interp.get_variable("head_table");
        assert!(head_var.is_some(), "Head table variable not found");
        
        if let Some(Value::Table(table_rc)) = head_var {
            let table_borrowed = table_rc.borrow();
            assert_eq!(table_borrowed.rows.len(), 2); // Первые 2 строки
            assert_eq!(table_borrowed.column_names, vec!["id", "name", "age"]);
        } else {
            panic!("Expected head table value");
        }
    }

    #[test]
    fn test_optimized_table_select() {
        let mut interp = Interpreter::new();

        // Создаем тестовые данные
        let result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30], [3, 'Charlie', 35]]");
        assert!(result1.is_ok(), "Failed to create data array: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        assert!(result2.is_ok(), "Failed to create headers: {:?}", result2);

        let result3 = interp.exec("global my_table = table(data, headers)");
        assert!(result3.is_ok(), "Failed to create table: {:?}", result3);

        // Проверяем, что таблица создалась правильно
        let table_var = interp.get_variable("my_table");
        println!("my_table variable: {:?}", table_var);
        assert!(table_var.is_some(), "Table variable not found");

        // Тестируем select операцию напрямую
        let result4 = interp.exec("table_select(my_table, ['name', 'age'])");
        println!("Direct table_select result: {:?}", result4);

        // Тестируем select операцию с присваиванием
        let result5 = interp.exec("global selected = table_select(my_table, ['name', 'age'])");
        println!("table_select with assignment result: {:?}", result5);
        assert!(result5.is_ok(), "Failed to select columns: {:?}", result5);

        // Проверяем результат
        let selected_var = interp.get_variable("selected");
        println!("selected variable: {:?}", selected_var);
        assert!(selected_var.is_some(), "Selected table variable not found");

        if let Some(Value::Table(table_rc)) = selected_var {
            let table_borrowed = table_rc.borrow();
            assert_eq!(table_borrowed.column_names, vec!["name", "age"]);
            assert_eq!(table_borrowed.rows.len(), 3);
            // Проверяем, что у нас только 2 колонки в каждой строке
            for row in &table_borrowed.rows {
                assert_eq!(row.len(), 2);
            }
        } else {
            panic!("Expected selected table value, got: {:?}", selected_var);
        }
    }

    #[test]
    fn test_lazy_table_operations() {
        // Создаем тестовую таблицу
        let mut table = Table::new(vec!["id".to_string(), "name".to_string(), "age".to_string()]);
        table.add_row(vec![Value::Number(1.0), Value::String("Alice".to_string()), Value::Number(25.0)]).unwrap();
        table.add_row(vec![Value::Number(2.0), Value::String("Bob".to_string()), Value::Number(30.0)]).unwrap();
        table.add_row(vec![Value::Number(3.0), Value::String("Charlie".to_string()), Value::Number(35.0)]).unwrap();
        table.add_row(vec![Value::Number(4.0), Value::String("David".to_string()), Value::Number(40.0)]).unwrap();

        let table_rc = Rc::new(RefCell::new(table));

        // Тестируем цепочку ленивых операций
        let lazy_table = LazyTable::new(table_rc)
            .where_op("age".to_string(), ">".to_string(), Value::Number(25.0), 1)
            .select(vec!["name".to_string(), "age".to_string()], 1)
            .head(2, 1);

        // Материализуем результат
        let result = lazy_table.materialize();
        assert!(result.is_ok(), "Failed to materialize lazy table: {:?}", result);

        let materialized = result.unwrap();
        assert_eq!(materialized.column_names, vec!["name", "age"]);
        assert_eq!(materialized.rows.len(), 2); // Bob и Charlie, но только первые 2
        
        // Проверяем содержимое
        assert_eq!(materialized.rows[0][0], Value::String("Bob".to_string()));
        assert_eq!(materialized.rows[0][1], Value::Number(30.0));
        assert_eq!(materialized.rows[1][0], Value::String("Charlie".to_string()));
        assert_eq!(materialized.rows[1][1], Value::Number(35.0));
    }

    #[test]
    fn test_memory_efficiency() {
        let mut interp = Interpreter::new();

        // Создаем большую таблицу напрямую
        let mut big_data_cmd = String::from("global big_data = [");
        for i in 0..100 {  // Уменьшим количество для быстрого тестирования
            if i > 0 {
                big_data_cmd.push_str(", ");
            }
            big_data_cmd.push_str(&format!("[{}, 'User{}', {}]", i, i, 20 + (i % 60)));
        }
        big_data_cmd.push(']');

        let result1 = interp.exec(&big_data_cmd);
        assert!(result1.is_ok(), "Failed to create big_data: {:?}", result1);

        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        assert!(result2.is_ok());

        let result3 = interp.exec("global big_table = table(big_data, headers)");
        assert!(result3.is_ok(), "Failed to create big table: {:?}", result3);

        // Выполняем несколько операций фильтрации
        let result4 = interp.exec("global filtered1 = table_where(big_table, 'age', '>', 30)");
        assert!(result4.is_ok(), "Failed first filter: {:?}", result4);

        let result5 = interp.exec("global filtered2 = table_where(filtered1, 'age', '<', 70)");
        assert!(result5.is_ok(), "Failed second filter: {:?}", result5);

        let result6 = interp.exec("global selected = table_select(filtered2, ['name', 'age'])");
        assert!(result6.is_ok(), "Failed select: {:?}", result6);

        // Проверяем, что все операции выполнились корректно
        let selected_var = interp.get_variable("selected");
        assert!(selected_var.is_some(), "Selected table variable not found");

        // Добавим отладочную информацию
        let filtered1_var = interp.get_variable("filtered1");
        if let Some(Value::Table(table_rc)) = filtered1_var {
            let table_borrowed = table_rc.borrow();
            println!("Filtered1 has {} rows", table_borrowed.rows.len());
        }

        let filtered2_var = interp.get_variable("filtered2");
        if let Some(Value::Table(table_rc)) = filtered2_var {
            let table_borrowed = table_rc.borrow();
            println!("Filtered2 has {} rows", table_borrowed.rows.len());
        }

        if let Some(Value::Table(table_rc)) = selected_var {
            let table_borrowed = table_rc.borrow();
            assert_eq!(table_borrowed.column_names, vec!["name", "age"]);
            println!("Selected table has {} rows", table_borrowed.rows.len());

            // Если нет результатов, проверим исходную таблицу
            if table_borrowed.rows.len() == 0 {
                let big_table_var = interp.get_variable("big_table");
                if let Some(Value::Table(big_table_rc)) = big_table_var {
                    let big_table_borrowed = big_table_rc.borrow();
                    println!("Big table has {} rows", big_table_borrowed.rows.len());
                    if big_table_borrowed.rows.len() > 0 {
                        if let Value::Number(age) = &big_table_borrowed.rows[0][2] {
                            println!("First row age: {}", age);
                        }
                    }
                }
            }

            assert!(table_borrowed.rows.len() > 0, "Expected some filtered results");

            // Проверяем, что все возрасты в диапазоне 30-70
            for row in &table_borrowed.rows {
                if let Value::Number(age) = &row[1] {
                    assert!(*age > 30.0 && *age < 70.0, "Age {} not in expected range", age);
                }
            }
        } else {
            panic!("Expected selected table value");
        }
    }

    #[test]
    fn test_table_sharing() {
        let mut interp = Interpreter::new();

        // Создаем таблицу
        let result1 = interp.exec("global data = [[1, 'Alice', 25], [2, 'Bob', 30]]");
        assert!(result1.is_ok());

        let result2 = interp.exec("global headers = ['id', 'name', 'age']");
        assert!(result2.is_ok());

        let result3 = interp.exec("global original_table = table(data, headers)");
        assert!(result3.is_ok());

        // Создаем несколько представлений одной таблицы
        let result4 = interp.exec("global view1 = table_where(original_table, 'age', '>', 20)");
        assert!(result4.is_ok());

        let result5 = interp.exec("global view2 = table_select(original_table, ['name', 'age'])");
        assert!(result5.is_ok());

        // Проверяем, что все представления работают корректно
        let view1_var = interp.get_variable("view1");
        let view2_var = interp.get_variable("view2");
        
        assert!(view1_var.is_some() && view2_var.is_some(), "Views not created");
        
        if let (Some(Value::Table(table1_rc)), Some(Value::Table(table2_rc))) = (view1_var, view2_var) {
            let table1_borrowed = table1_rc.borrow();
            let table2_borrowed = table2_rc.borrow();
            
            assert_eq!(table1_borrowed.rows.len(), 2); // Все строки прошли фильтр age > 20
            assert_eq!(table2_borrowed.column_names, vec!["name", "age"]);
            assert_eq!(table2_borrowed.rows.len(), 2);
        } else {
            panic!("Expected table values in views");
        }
    }
}
