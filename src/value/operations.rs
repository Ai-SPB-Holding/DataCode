// Операции над значениями в DataCode
// Включает арифметические операции, сравнения и преобразования

use super::types::{Value, DataType};

/// Трейт для операций над значениями
pub trait ValueOperations {
    /// Сложение значений
    fn add(&self, other: &Value) -> Result<Value, String>;

    /// Вычитание значений
    fn subtract(&self, other: &Value) -> Result<Value, String>;

    /// Умножение значений
    fn multiply(&self, other: &Value) -> Result<Value, String>;

    /// Деление значений
    fn divide(&self, other: &Value) -> Result<Value, String>;

    /// Сравнение на равенство
    fn equals(&self, other: &Value) -> bool;

    /// Сравнение "меньше чем"
    fn less_than(&self, other: &Value) -> Result<bool, String>;

    /// Сравнение "больше чем"
    fn greater_than(&self, other: &Value) -> Result<bool, String>;

    /// Получить тип значения
    fn get_type(&self) -> DataType;

    /// Проверить, является ли значение числовым
    fn is_numeric(&self) -> bool;

    /// Проверить, является ли значение пустым
    fn is_null(&self) -> bool;

    /// Преобразовать в строку для отображения
    fn to_display_string(&self) -> String;
}

impl ValueOperations for Value {
    fn add(&self, other: &Value) -> Result<Value, String> {
        add_values(self, other)
    }
    
    fn subtract(&self, other: &Value) -> Result<Value, String> {
        subtract_values(self, other)
    }
    
    fn multiply(&self, other: &Value) -> Result<Value, String> {
        multiply_values(self, other)
    }
    
    fn divide(&self, other: &Value) -> Result<Value, String> {
        divide_values(self, other)
    }
    
    fn equals(&self, other: &Value) -> bool {
        values_equal(self, other)
    }
    
    fn less_than(&self, other: &Value) -> Result<bool, String> {
        compare_values(self, other).map(|ord| ord == std::cmp::Ordering::Less)
    }
    
    fn greater_than(&self, other: &Value) -> Result<bool, String> {
        compare_values(self, other).map(|ord| ord == std::cmp::Ordering::Greater)
    }

    fn get_type(&self) -> DataType {
        DataType::from_value(self)
    }

    fn is_numeric(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    fn to_display_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_display_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Table(table) => {
                let table_borrowed = table.borrow();
                format!("Table({}x{})", table_borrowed.rows.len(), table_borrowed.columns.len())
            }
            Value::Currency(s) => s.clone(),
            Value::Null => "null".to_string(),
            Value::Path(p) => p.to_string_lossy().to_string(),
            Value::PathPattern(p) => format!("{}*", p.to_string_lossy()),
        }
    }
}

/// Сложение двух значений
pub fn add_values(left: &Value, right: &Value) -> Result<Value, String> {
    use Value::*;
    match (left, right) {
        (Number(a), Number(b)) => Ok(Number(a + b)),
        (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
        (String(a), Number(b)) => Ok(String(format!("{}{}", a, b))),
        (Number(a), String(b)) => Ok(String(format!("{}{}", a, b))),
        (Path(p), String(s)) => {
            let mut new_path = p.clone();
            let relative = s.trim_start_matches('/');
            new_path.push(relative);
            // Проверяем, содержит ли строка glob паттерны
            if s.contains('*') || s.contains('?') || s.contains('[') {
                Ok(PathPattern(new_path))
            } else {
                Ok(Path(new_path))
            }
        }
        (String(s), Path(p)) => {
            let mut new_str = s.clone();
            new_str.push_str(p.to_str().ok_or("Invalid path to string conversion")?);
            Ok(String(new_str))
        }
        _ => Err(format!("Unsupported add operation between {:?} and {:?}", left, right)),
    }
}

/// Вычитание двух значений
pub fn subtract_values(left: &Value, right: &Value) -> Result<Value, String> {
    use Value::*;
    match (left, right) {
        (Number(a), Number(b)) => Ok(Number(a - b)),
        _ => Err(format!("Unsupported subtract operation between {:?} and {:?}", left, right)),
    }
}

/// Умножение двух значений
pub fn multiply_values(left: &Value, right: &Value) -> Result<Value, String> {
    use Value::*;
    match (left, right) {
        (Number(a), Number(b)) => Ok(Number(a * b)),
        (String(s), Number(n)) => {
            if *n >= 0.0 && n.fract() == 0.0 {
                let count = *n as usize;
                Ok(String(s.repeat(count)))
            } else {
                Err("String multiplication requires non-negative integer".to_string())
            }
        }
        (Number(n), String(s)) => {
            if *n >= 0.0 && n.fract() == 0.0 {
                let count = *n as usize;
                Ok(String(s.repeat(count)))
            } else {
                Err("String multiplication requires non-negative integer".to_string())
            }
        }
        _ => Err(format!("Unsupported multiply operation between {:?} and {:?}", left, right)),
    }
}

/// Деление двух значений
pub fn divide_values(left: &Value, right: &Value) -> Result<Value, String> {
    use Value::*;
    match (left, right) {
        (Number(a), Number(b)) => {
            if *b == 0.0 {
                Err("Division by zero".to_string())
            } else {
                Ok(Number(a / b))
            }
        }
        (Path(p), String(s)) => {
            // Специальная логика для путей: / используется для соединения путей
            let mut new_path = p.clone();
            let relative = s.trim_start_matches('/');
            new_path.push(relative);
            if s.contains('*') || s.contains('?') || s.contains('[') {
                Ok(PathPattern(new_path))
            } else {
                Ok(Path(new_path))
            }
        }
        _ => Err(format!("Unsupported divide operation between {:?} and {:?}", left, right)),
    }
}

/// Проверка равенства двух значений
pub fn values_equal(left: &Value, right: &Value) -> bool {
    use Value::*;
    match (left, right) {
        (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
        (String(a), String(b)) => a == b,
        (Bool(a), Bool(b)) => a == b,
        (Null, Null) => true,
        (Currency(a), Currency(b)) => a == b,
        (Path(a), Path(b)) => a == b,
        (PathPattern(a), PathPattern(b)) => a == b,
        // Автоматическое преобразование типов для сравнения
        (Number(n), String(s)) | (String(s), Number(n)) => {
            s.parse::<f64>().map(|parsed| (parsed - n).abs() < f64::EPSILON).unwrap_or(false)
        }
        _ => false,
    }
}

/// Сравнение двух значений
pub fn compare_values(left: &Value, right: &Value) -> Result<std::cmp::Ordering, String> {
    use Value::*;
    use std::cmp::Ordering;
    
    match (left, right) {
        (Number(a), Number(b)) => Ok(a.partial_cmp(b).unwrap_or(Ordering::Equal)),
        (String(a), String(b)) => Ok(a.cmp(b)),
        (Bool(a), Bool(b)) => Ok(a.cmp(b)),
        // Автоматическое преобразование для сравнения
        (Number(n), String(s)) => {
            match s.parse::<f64>() {
                Ok(parsed) => Ok(n.partial_cmp(&parsed).unwrap_or(Ordering::Equal)),
                Err(_) => Err(format!("Cannot compare number with non-numeric string: {}", s)),
            }
        }
        (String(s), Number(n)) => {
            match s.parse::<f64>() {
                Ok(parsed) => Ok(parsed.partial_cmp(n).unwrap_or(Ordering::Equal)),
                Err(_) => Err(format!("Cannot compare non-numeric string with number: {}", s)),
            }
        }
        _ => Err(format!("Cannot compare {:?} and {:?}", left, right)),
    }
}

/// Логическое И для значений
pub fn logical_and(left: &Value, right: &Value) -> Value {
    let left_bool = to_boolean(left);
    let right_bool = to_boolean(right);
    Value::Bool(left_bool && right_bool)
}

/// Логическое ИЛИ для значений
pub fn logical_or(left: &Value, right: &Value) -> Value {
    let left_bool = to_boolean(left);
    let right_bool = to_boolean(right);
    Value::Bool(left_bool || right_bool)
}

/// Логическое НЕ для значения
pub fn logical_not(value: &Value) -> Value {
    Value::Bool(!to_boolean(value))
}

/// Преобразование значения в булево
pub fn to_boolean(value: &Value) -> bool {
    use Value::*;
    match value {
        Bool(b) => *b,
        Number(n) => *n != 0.0,
        String(s) => !s.is_empty(),
        Array(arr) => !arr.is_empty(),
        Object(obj) => !obj.is_empty(),
        Table(table) => !table.borrow().rows.is_empty(),
        Currency(s) => !s.is_empty(),
        Path(p) => p.as_os_str().len() > 0,
        PathPattern(p) => p.as_os_str().len() > 0,
        Null => false,
    }
}

/// Унарный минус для значения
pub fn negate_value(value: &Value) -> Result<Value, String> {
    match value {
        Value::Number(n) => Ok(Value::Number(-n)),
        Value::Bool(b) => Ok(Value::Bool(!b)),
        _ => Err(format!("Cannot negate {:?}", value)),
    }
}

/// Получить абсолютное значение
pub fn abs_value(value: &Value) -> Result<Value, String> {
    match value {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(format!("Cannot get absolute value of {:?}", value)),
    }
}

/// Проверить, является ли значение "истинным" в логическом контексте
pub fn is_truthy(value: &Value) -> bool {
    to_boolean(value)
}

/// Проверить, является ли значение "ложным" в логическом контексте
pub fn is_falsy(value: &Value) -> bool {
    !to_boolean(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_add_numbers() {
        let result = add_values(&Value::Number(2.0), &Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_add_strings() {
        let result = add_values(&Value::String("hello".to_string()), &Value::String(" world".to_string())).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_add_string_number() {
        let result = add_values(&Value::String("value: ".to_string()), &Value::Number(42.0)).unwrap();
        assert_eq!(result, Value::String("value: 42".to_string()));
    }

    #[test]
    fn test_add_path_string() {
        let path = PathBuf::from("/home/user");
        let result = add_values(&Value::Path(path), &Value::String("documents".to_string())).unwrap();

        match result {
            Value::Path(p) => assert_eq!(p, PathBuf::from("/home/user/documents")),
            _ => panic!("Expected Path result"),
        }
    }

    #[test]
    fn test_add_path_glob() {
        let path = PathBuf::from("/home/user");
        let result = add_values(&Value::Path(path), &Value::String("*.txt".to_string())).unwrap();

        match result {
            Value::PathPattern(p) => assert_eq!(p, PathBuf::from("/home/user/*.txt")),
            _ => panic!("Expected PathPattern result"),
        }
    }

    #[test]
    fn test_subtract_numbers() {
        let result = subtract_values(&Value::Number(5.0), &Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_multiply_numbers() {
        let result = multiply_values(&Value::Number(4.0), &Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::Number(12.0));
    }

    #[test]
    fn test_multiply_string_number() {
        let result = multiply_values(&Value::String("hi".to_string()), &Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::String("hihihi".to_string()));
    }

    #[test]
    fn test_divide_numbers() {
        let result = divide_values(&Value::Number(10.0), &Value::Number(2.0)).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_divide_by_zero() {
        let result = divide_values(&Value::Number(10.0), &Value::Number(0.0));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Division by zero"));
    }

    #[test]
    fn test_divide_path_string() {
        let path = PathBuf::from("/home");
        let result = divide_values(&Value::Path(path), &Value::String("user".to_string())).unwrap();

        match result {
            Value::Path(p) => assert_eq!(p, PathBuf::from("/home/user")),
            _ => panic!("Expected Path result"),
        }
    }

    #[test]
    fn test_values_equal() {
        assert!(values_equal(&Value::Number(42.0), &Value::Number(42.0)));
        assert!(values_equal(&Value::String("hello".to_string()), &Value::String("hello".to_string())));
        assert!(values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(values_equal(&Value::Null, &Value::Null));

        assert!(!values_equal(&Value::Number(42.0), &Value::Number(43.0)));
        assert!(!values_equal(&Value::String("hello".to_string()), &Value::String("world".to_string())));
    }

    #[test]
    fn test_values_equal_cross_type() {
        assert!(values_equal(&Value::Number(42.0), &Value::String("42".to_string())));
        assert!(values_equal(&Value::String("3.14".to_string()), &Value::Number(3.14)));
        assert!(!values_equal(&Value::Number(42.0), &Value::String("hello".to_string())));
    }

    #[test]
    fn test_compare_values() {
        use std::cmp::Ordering;

        assert_eq!(compare_values(&Value::Number(5.0), &Value::Number(3.0)).unwrap(), Ordering::Greater);
        assert_eq!(compare_values(&Value::Number(3.0), &Value::Number(5.0)).unwrap(), Ordering::Less);
        assert_eq!(compare_values(&Value::Number(5.0), &Value::Number(5.0)).unwrap(), Ordering::Equal);

        assert_eq!(compare_values(&Value::String("b".to_string()), &Value::String("a".to_string())).unwrap(), Ordering::Greater);
        assert_eq!(compare_values(&Value::String("a".to_string()), &Value::String("b".to_string())).unwrap(), Ordering::Less);
    }

    #[test]
    fn test_logical_operations() {
        let true_val = Value::Bool(true);
        let false_val = Value::Bool(false);

        assert_eq!(logical_and(&true_val, &true_val), Value::Bool(true));
        assert_eq!(logical_and(&true_val, &false_val), Value::Bool(false));
        assert_eq!(logical_or(&false_val, &true_val), Value::Bool(true));
        assert_eq!(logical_or(&false_val, &false_val), Value::Bool(false));
        assert_eq!(logical_not(&true_val), Value::Bool(false));
        assert_eq!(logical_not(&false_val), Value::Bool(true));
    }

    #[test]
    fn test_to_boolean() {
        assert_eq!(to_boolean(&Value::Bool(true)), true);
        assert_eq!(to_boolean(&Value::Bool(false)), false);
        assert_eq!(to_boolean(&Value::Number(1.0)), true);
        assert_eq!(to_boolean(&Value::Number(0.0)), false);
        assert_eq!(to_boolean(&Value::String("hello".to_string())), true);
        assert_eq!(to_boolean(&Value::String("".to_string())), false);
        assert_eq!(to_boolean(&Value::Null), false);
        assert_eq!(to_boolean(&Value::Array(vec![Value::Number(1.0)])), true);
        assert_eq!(to_boolean(&Value::Array(vec![])), false);
    }

    #[test]
    fn test_negate_value() {
        assert_eq!(negate_value(&Value::Number(42.0)).unwrap(), Value::Number(-42.0));
        assert_eq!(negate_value(&Value::Number(-10.0)).unwrap(), Value::Number(10.0));
        assert_eq!(negate_value(&Value::Bool(true)).unwrap(), Value::Bool(false));
        assert_eq!(negate_value(&Value::Bool(false)).unwrap(), Value::Bool(true));

        assert!(negate_value(&Value::String("hello".to_string())).is_err());
    }

    #[test]
    fn test_abs_value() {
        assert_eq!(abs_value(&Value::Number(42.0)).unwrap(), Value::Number(42.0));
        assert_eq!(abs_value(&Value::Number(-42.0)).unwrap(), Value::Number(42.0));
        assert_eq!(abs_value(&Value::Number(0.0)).unwrap(), Value::Number(0.0));

        assert!(abs_value(&Value::String("hello".to_string())).is_err());
    }

    #[test]
    fn test_is_truthy_falsy() {
        assert!(is_truthy(&Value::Bool(true)));
        assert!(is_falsy(&Value::Bool(false)));
        assert!(is_truthy(&Value::Number(1.0)));
        assert!(is_falsy(&Value::Number(0.0)));
        assert!(is_truthy(&Value::String("hello".to_string())));
        assert!(is_falsy(&Value::String("".to_string())));
        assert!(is_falsy(&Value::Null));
    }

    #[test]
    fn test_value_operations_trait() {
        let val1 = Value::Number(10.0);
        let val2 = Value::Number(5.0);

        assert_eq!(val1.add(&val2).unwrap(), Value::Number(15.0));
        assert_eq!(val1.subtract(&val2).unwrap(), Value::Number(5.0));
        assert_eq!(val1.multiply(&val2).unwrap(), Value::Number(50.0));
        assert_eq!(val1.divide(&val2).unwrap(), Value::Number(2.0));

        assert!(!val1.equals(&val2));
        assert!(val1.greater_than(&val2).unwrap());
        assert!(!val1.less_than(&val2).unwrap());
    }
}
