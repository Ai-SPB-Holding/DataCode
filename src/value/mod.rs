// Модульная структура системы типов DataCode
// Этот модуль координирует работу всех компонентов системы значений

pub mod types;
pub mod table;
pub mod conversions;
pub mod operations;
pub mod lazy;
pub mod relations;

// Реэкспорт основных типов для удобства использования
pub use types::{Value, DataType};
pub use table::Table;
pub use lazy::LazyTable;

pub use operations::ValueOperations;







#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_ops_trait() {
        let val1 = Value::Number(42.0);
        let val2 = Value::Number(10.0);

        let result = val1.add(&val2).unwrap();
        assert_eq!(result, Value::Number(52.0));

        assert!(val1.is_numeric());
        assert!(!val1.is_null());
        assert_eq!(val1.get_type(), DataType::Integer);
    }



    #[test]
    fn test_display_string() {
        assert_eq!(Value::Number(42.0).to_display_string(), "42");
        assert_eq!(Value::Number(3.14).to_display_string(), "3.14");
        assert_eq!(Value::String("hello".to_string()).to_display_string(), "hello");
        assert_eq!(Value::Bool(true).to_display_string(), "true");
        assert_eq!(Value::Null.to_display_string(), "null");
    }


}
