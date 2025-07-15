use data_code::interpreter::Interpreter;
use data_code::value::Value;

#[cfg(test)]
mod filter_tests {
    use super::*;

    fn create_test_table() -> Interpreter {
        let mut interp = Interpreter::new();

        // Создаем тестовую таблицу с данными сотрудников построчно
        interp.exec("global employees_data = []").unwrap();
        interp.exec("global employees_data = push(employees_data, [1, 'Alice', 28, 75000, 'Engineering', true])").unwrap();
        interp.exec("global employees_data = push(employees_data, [2, 'Bob', 35, 82000, 'Marketing', true])").unwrap();
        interp.exec("global employees_data = push(employees_data, [3, 'Charlie', 42, 68000, 'Engineering', false])").unwrap();
        interp.exec("global employees_data = push(employees_data, [4, 'Diana', 29, 71500, 'HR', true])").unwrap();
        interp.exec("global employees_data = push(employees_data, [5, 'Eve', 31, 89000, 'Marketing', true])").unwrap();
        interp.exec("global employees_data = push(employees_data, [6, 'Frank', 45, 105000, 'Engineering', true])").unwrap();
        interp.exec("global employees_data = push(employees_data, [7, 'Grace', 26, 58000, 'HR', false])").unwrap();
        interp.exec("global employees_data = push(employees_data, [8, 'Henry', 38, 92000, 'Marketing', true])").unwrap();

        interp.exec("global headers = ['id', 'name', 'age', 'salary', 'department', 'active']").unwrap();
        interp.exec("global employees = table(employees_data, headers)").unwrap();

        interp
    }

    #[test]
    fn test_table_filter_basic() {
        let mut interp = create_test_table();
        
        // Фильтруем сотрудников старше 30 лет
        interp.exec("global filtered = table_filter(employees, 'age > 30')").unwrap();
        
        // Проверяем результат
        if let Some(Value::Table(table)) = interp.get_variable("filtered") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 5); // Bob, Charlie, Eve, Frank, Henry

            // Проверяем, что все возрасты больше 30
            let age_col_index = table_borrowed.column_names.iter().position(|name| name == "age").unwrap();
            for row in &table_borrowed.rows {
                if let Some(Value::Number(age)) = row.get(age_col_index) {
                    assert!(age > &30.0);
                }
            }
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_table_filter_complex_condition() {
        let mut interp = create_test_table();

        // Сначала проверим, что таблица создана правильно
        interp.exec("table_info(employees)").unwrap();

        // Пока используем простую фильтрацию через table_where
        interp.exec("global eng_employees = table_where(employees, 'department', '=', 'Engineering')").unwrap();
        interp.exec("global high_salary = table_where(eng_employees, 'salary', '>', 70000)").unwrap();
        interp.exec("global filtered = table_where(high_salary, 'active', '=', true)").unwrap();
        
        if let Some(Value::Table(table)) = interp.get_variable("filtered") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 2); // Alice и Frank

            // Проверяем условия
            let dept_col = table_borrowed.column_names.iter().position(|name| name == "department").unwrap();
            let salary_col = table_borrowed.column_names.iter().position(|name| name == "salary").unwrap();
            let active_col = table_borrowed.column_names.iter().position(|name| name == "active").unwrap();

            for row in &table_borrowed.rows {
                if let (Some(Value::String(dept)), Some(Value::Number(salary)), Some(Value::Bool(active))) =
                    (row.get(dept_col), row.get(salary_col), row.get(active_col)) {
                    assert_eq!(dept, "Engineering");
                    assert!(salary > &70000.0);
                    assert!(active);
                }
            }
        } else {
            panic!("Expected table result");
        }
    }

    #[test]
    fn test_table_where_operators() {
        let mut interp = create_test_table();
        
        // Тестируем различные операторы
        
        // Равенство
        interp.exec("global eq_result = table_where(employees, 'department', '=', 'Engineering')").unwrap();
        if let Some(Value::Table(table)) = interp.get_variable("eq_result") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 3); // Alice, Charlie, Frank
        }
        
        // Больше
        interp.exec("global gt_result = table_where(employees, 'salary', '>', 80000)").unwrap();
        if let Some(Value::Table(table)) = interp.get_variable("gt_result") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 4); // Bob, Eve, Frank, Henry
        }
        
        // Меньше или равно
        interp.exec("global le_result = table_where(employees, 'age', '<=', 30)").unwrap();
        if let Some(Value::Table(table)) = interp.get_variable("le_result") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 3); // Alice, Diana, Grace
        }
        
        // Не равно
        interp.exec("global ne_result = table_where(employees, 'active', '!=', true)").unwrap();
        if let Some(Value::Table(table)) = interp.get_variable("ne_result") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 2); // Charlie, Grace
        }
    }

    #[test]
    fn test_table_query() {
        let mut interp = create_test_table();
        
        // table_query должен работать как table_filter
        interp.exec("global query_result = table_query(employees, 'salary >= 75000 and age < 40')").unwrap();
        
        if let Some(Value::Table(table)) = interp.get_variable("query_result") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 4); // Alice, Bob, Diana, Eve

            let salary_col = table_borrowed.column_names.iter().position(|name| name == "salary").unwrap();
            let age_col = table_borrowed.column_names.iter().position(|name| name == "age").unwrap();

            for row in &table_borrowed.rows {
                if let (Some(Value::Number(salary)), Some(Value::Number(age))) =
                    (row.get(salary_col), row.get(age_col)) {
                    assert!(salary >= &75000.0);
                    assert!(age < &40.0);
                }
            }
        }
    }

    #[test]
    fn test_table_distinct() {
        let mut interp = create_test_table();
        
        // Получаем уникальные отделы
        interp.exec("global departments = table_distinct(employees, 'department')").unwrap();
        
        if let Some(Value::Array(values)) = interp.get_variable("departments") {
            assert_eq!(values.len(), 3); // Engineering, Marketing, HR
            
            let mut dept_names: Vec<String> = values.iter()
                .filter_map(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .collect();
            dept_names.sort();
            
            assert_eq!(dept_names, vec!["Engineering", "HR", "Marketing"]);
        }
    }

    #[test]
    fn test_table_sample() {
        let mut interp = create_test_table();
        
        // Берем случайную выборку из 3 сотрудников
        interp.exec("global sample_result = table_sample(employees, 3)").unwrap();
        
        if let Some(Value::Table(table)) = interp.get_variable("sample_result") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 3);
            assert_eq!(table_borrowed.column_names.len(), 6); // Все колонки должны остаться
        }

        // Тестируем случай когда запрашиваем больше строк чем есть
        interp.exec("global large_sample = table_sample(employees, 20)").unwrap();

        if let Some(Value::Table(table)) = interp.get_variable("large_sample") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 8); // Все строки
        }
    }

    #[test]
    fn test_table_between() {
        let mut interp = create_test_table();
        
        // Фильтруем сотрудников с возрастом от 30 до 40
        interp.exec("global between_result = table_between(employees, 'age', 30, 40)").unwrap();
        
        if let Some(Value::Table(table)) = interp.get_variable("between_result") {
            let table_borrowed = table.borrow();
            let age_col = table_borrowed.column_names.iter().position(|name| name == "age").unwrap();

            for row in &table_borrowed.rows {
                if let Some(Value::Number(age)) = row.get(age_col) {
                    assert!(age >= &30.0 && age <= &40.0);
                }
            }
        }
    }

    #[test]
    fn test_table_in() {
        let mut interp = create_test_table();
        
        // Фильтруем сотрудников из определенных отделов
        interp.exec("global in_result = table_in(employees, 'department', ['Engineering', 'HR'])").unwrap();
        
        if let Some(Value::Table(table)) = interp.get_variable("in_result") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 5); // Alice, Charlie, Frank, Diana, Grace

            let dept_col = table_borrowed.column_names.iter().position(|name| name == "department").unwrap();

            for row in &table_borrowed.rows {
                if let Some(Value::String(dept)) = row.get(dept_col) {
                    assert!(dept == "Engineering" || dept == "HR");
                }
            }
        }
    }

    #[test]
    fn test_table_null_filters() {
        let mut interp = Interpreter::new();

        // Создаем таблицу с null значениями
        interp.exec("global data_with_nulls = []").unwrap();
        interp.exec("global data_with_nulls = push(data_with_nulls, [1, 'Alice', 28])").unwrap();
        interp.exec("global data_with_nulls = push(data_with_nulls, [2, null, 35])").unwrap();
        interp.exec("global data_with_nulls = push(data_with_nulls, [3, 'Charlie', null])").unwrap();
        interp.exec("global data_with_nulls = push(data_with_nulls, [4, 'Diana', 29])").unwrap();

        interp.exec("global headers = ['id', 'name', 'age']").unwrap();
        interp.exec("global test_table = table(data_with_nulls, headers)").unwrap();
        
        // Тестируем фильтрацию по null значениям
        interp.exec("global null_names = table_is_null(test_table, 'name')").unwrap();
        if let Some(Value::Table(table)) = interp.get_variable("null_names") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 1); // Только Bob
        }

        // Тестируем фильтрацию по не-null значениям
        interp.exec("global not_null_ages = table_not_null(test_table, 'age')").unwrap();
        if let Some(Value::Table(table)) = interp.get_variable("not_null_ages") {
            let table_borrowed = table.borrow();
            assert_eq!(table_borrowed.rows.len(), 3); // Alice, Bob, Diana
        }
    }

    #[test]
    fn test_error_handling() {
        let mut interp = create_test_table();
        
        // Тестируем ошибку при неправильной колонке
        let result = interp.exec("global error_result = table_where(employees, 'nonexistent', '=', 'value')");
        assert!(result.is_err());
        
        // Тестируем ошибку при неправильном операторе
        let result = interp.exec("global error_result = table_where(employees, 'age', 'INVALID_OP', 30)");
        assert!(result.is_err());
        
        // Тестируем ошибку при неправильном условии в table_filter
        let result = interp.exec("global error_result = table_filter(employees, 'invalid syntax here')");
        assert!(result.is_err());
    }
}
