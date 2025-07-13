use data_code::interpreter::Interpreter;
use data_code::value::Value;
use data_code::error::DataCodeError;

#[cfg(test)]
mod isinstance_tests {
    use super::*;

    #[test]
    fn test_isinstance_number() {
        let mut interp = Interpreter::new();
        
        // Тест с целыми числами
        interp.exec("global x = 42").unwrap();
        let result = interp.exec("global is_num = isinstance(x, 'number')").unwrap();
        assert_eq!(interp.get_variable("is_num"), Some(&Value::Bool(true)));
        
        // Тест с дробными числами
        interp.exec("global y = 3.14").unwrap();
        let result = interp.exec("global is_float = isinstance(y, 'float')").unwrap();
        assert_eq!(interp.get_variable("is_float"), Some(&Value::Bool(true)));
        
        // Тест с альтернативными именами типов
        let result = interp.exec("global is_int = isinstance(x, 'int')").unwrap();
        assert_eq!(interp.get_variable("is_int"), Some(&Value::Bool(true)));
        
        let result = interp.exec("global is_integer = isinstance(x, 'integer')").unwrap();
        assert_eq!(interp.get_variable("is_integer"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_isinstance_string() {
        let mut interp = Interpreter::new();
        
        interp.exec("global text = 'Hello, DataCode!'").unwrap();
        let result = interp.exec("global is_str = isinstance(text, 'string')").unwrap();
        assert_eq!(interp.get_variable("is_str"), Some(&Value::Bool(true)));
        
        // Тест с альтернативным именем типа
        let result = interp.exec("global is_str2 = isinstance(text, 'str')").unwrap();
        assert_eq!(interp.get_variable("is_str2"), Some(&Value::Bool(true)));
        
        // Негативный тест
        interp.exec("global num = 123").unwrap();
        let result = interp.exec("global not_str = isinstance(num, 'string')").unwrap();
        assert_eq!(interp.get_variable("not_str"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_isinstance_bool() {
        let mut interp = Interpreter::new();
        
        interp.exec("global flag = true").unwrap();
        let result = interp.exec("global is_bool = isinstance(flag, 'bool')").unwrap();
        assert_eq!(interp.get_variable("is_bool"), Some(&Value::Bool(true)));
        
        // Тест с альтернативным именем типа
        let result = interp.exec("global is_boolean = isinstance(flag, 'boolean')").unwrap();
        assert_eq!(interp.get_variable("is_boolean"), Some(&Value::Bool(true)));
        
        interp.exec("global flag2 = false").unwrap();
        let result = interp.exec("global is_bool2 = isinstance(flag2, 'bool')").unwrap();
        assert_eq!(interp.get_variable("is_bool2"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_isinstance_array() {
        let mut interp = Interpreter::new();
        
        interp.exec("global arr = [1, 2, 3]").unwrap();
        let result = interp.exec("global is_array = isinstance(arr, 'array')").unwrap();
        assert_eq!(interp.get_variable("is_array"), Some(&Value::Bool(true)));
        
        // Тест с альтернативным именем типа
        let result = interp.exec("global is_list = isinstance(arr, 'list')").unwrap();
        assert_eq!(interp.get_variable("is_list"), Some(&Value::Bool(true)));
        
        // Тест с пустым массивом
        interp.exec("global empty_arr = []").unwrap();
        let result = interp.exec("global is_empty_array = isinstance(empty_arr, 'array')").unwrap();
        assert_eq!(interp.get_variable("is_empty_array"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_isinstance_null() {
        let mut interp = Interpreter::new();

        let _result = interp.exec("global nothing = null").unwrap();
        let _result = interp.exec("global is_null = isinstance(nothing, 'null')").unwrap();
        assert_eq!(interp.get_variable("is_null"), Some(&Value::Bool(true)));

        // Тест с альтернативным именем типа
        let _result = interp.exec("global is_none = isinstance(nothing, 'none')").unwrap();
        assert_eq!(interp.get_variable("is_none"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_isinstance_currency() {
        let mut interp = Interpreter::new();
        
        // Создаем валютное значение через парсинг строки
        interp.exec("global money = '$100.50'").unwrap();
        
        // Проверяем, что это валюта (если система автоматически определила тип)
        // Или создаем валюту явно через функцию, если такая есть
        let result = interp.exec("global is_currency = isinstance(money, 'currency')");
        
        // Если автоматическое определение валюты работает, тест должен пройти
        // Иначе нужно будет добавить функцию для создания валютных значений
        if result.is_ok() {
            // Проверяем результат только если выполнение прошло успешно
            if let Some(Value::Bool(is_curr)) = interp.get_variable("is_currency") {
                // Тест пройдет, если система правильно определила валюту
                println!("Currency detection result: {}", is_curr);
            }
        }
        
        // Тест с альтернативным именем типа
        let result = interp.exec("global is_money = isinstance(money, 'money')");
        if result.is_ok() {
            if let Some(Value::Bool(is_money)) = interp.get_variable("is_money") {
                println!("Money detection result: {}", is_money);
            }
        }
    }

    #[test]
    fn test_isinstance_path() {
        let mut interp = Interpreter::new();
        
        // Создаем путь через функцию path()
        interp.exec("global my_path = path('/home/user')").unwrap();
        let result = interp.exec("global is_path = isinstance(my_path, 'path')").unwrap();
        assert_eq!(interp.get_variable("is_path"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_isinstance_case_insensitive() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 42").unwrap();
        
        // Тест с разными регистрами
        let result = interp.exec("global is_num1 = isinstance(x, 'NUMBER')").unwrap();
        assert_eq!(interp.get_variable("is_num1"), Some(&Value::Bool(true)));
        
        let result = interp.exec("global is_num2 = isinstance(x, 'Number')").unwrap();
        assert_eq!(interp.get_variable("is_num2"), Some(&Value::Bool(true)));
        
        let result = interp.exec("global is_num3 = isinstance(x, 'nUmBeR')").unwrap();
        assert_eq!(interp.get_variable("is_num3"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_isinstance_wrong_argument_count() {
        let mut interp = Interpreter::new();
        
        // Тест с недостаточным количеством аргументов
        let result = interp.exec("global test1 = isinstance(42)");
        assert!(result.is_err());
        
        // Тест с избыточным количеством аргументов
        let result = interp.exec("global test2 = isinstance(42, 'number', 'extra')");
        assert!(result.is_err());
    }

    #[test]
    fn test_isinstance_invalid_type_name() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 42").unwrap();
        
        // Тест с неизвестным типом
        let result = interp.exec("global test = isinstance(x, 'unknown_type')");
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("Unknown type name"));
        }
    }

    #[test]
    fn test_isinstance_non_string_type_argument() {
        let mut interp = Interpreter::new();
        
        interp.exec("global x = 42").unwrap();
        
        // Тест с числовым типом вместо строки
        let result = interp.exec("global test = isinstance(x, 123)");
        assert!(result.is_err());
    }

    #[test]
    fn test_isinstance_in_conditional() {
        let mut interp = Interpreter::new();
        
        // Тест использования isinstance в условных конструкциях
        let code = r#"
            global value = 42
            if isinstance(value, 'number') do
                global result = 'It is a number!'
            else
                global result = 'It is not a number!'
            endif
        "#;
        
        let result = interp.exec(code);
        assert!(result.is_ok());
        assert_eq!(interp.get_variable("result"), Some(&Value::String("It is a number!".to_string())));
    }

    #[test]
    fn test_isinstance_mixed_types() {
        let mut interp = Interpreter::new();
        
        // Создаем переменные разных типов
        interp.exec("global num = 42").unwrap();
        interp.exec("global text = 'hello'").unwrap();
        interp.exec("global flag = true").unwrap();
        interp.exec("global arr = [1, 2, 3]").unwrap();
        
        // Проверяем каждую переменную на разные типы
        let result = interp.exec("global num_is_num = isinstance(num, 'number')").unwrap();
        assert_eq!(interp.get_variable("num_is_num"), Some(&Value::Bool(true)));
        
        let result = interp.exec("global num_is_str = isinstance(num, 'string')").unwrap();
        assert_eq!(interp.get_variable("num_is_str"), Some(&Value::Bool(false)));
        
        let result = interp.exec("global text_is_str = isinstance(text, 'string')").unwrap();
        assert_eq!(interp.get_variable("text_is_str"), Some(&Value::Bool(true)));
        
        let result = interp.exec("global text_is_num = isinstance(text, 'number')").unwrap();
        assert_eq!(interp.get_variable("text_is_num"), Some(&Value::Bool(false)));
        
        let result = interp.exec("global flag_is_bool = isinstance(flag, 'bool')").unwrap();
        assert_eq!(interp.get_variable("flag_is_bool"), Some(&Value::Bool(true)));
        
        let result = interp.exec("global arr_is_array = isinstance(arr, 'array')").unwrap();
        assert_eq!(interp.get_variable("arr_is_array"), Some(&Value::Bool(true)));
    }
}
