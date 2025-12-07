// Основные типы данных DataCode
// Определяет Value enum и DataType enum с их базовой функциональностью

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use super::table::Table;
use super::conversions::{is_currency_string, is_date_string};

/// Перечисление типов данных в DataCode
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    Integer,
    Float,
    String,
    Bool,
    Date,
    Currency,
    Null,
    Mixed,
}

impl DataType {
    /// Определить тип данных по значению
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    DataType::Integer
                } else {
                    DataType::Float
                }
            }
            Value::String(s) => {
                // Сначала проверяем, является ли строка денежным значением
                if is_currency_string(s) {
                    DataType::Currency
                }
                // Затем проверяем, является ли строка датой
                else if is_date_string(s) {
                    DataType::Date
                } else {
                    DataType::String
                }
            }
            Value::Bool(_) => DataType::Bool,
            Value::Currency(_) => DataType::Currency,
            Value::Null => DataType::Null,
            _ => DataType::Mixed,
        }
    }
    
    /// Проверить, является ли тип числовым
    #[allow(dead_code)]
    pub fn is_numeric(&self) -> bool {
        matches!(self, DataType::Integer | DataType::Float)
    }
    
    /// Получить строковое представление типа
    pub fn to_string(&self) -> &'static str {
        match self {
            DataType::Integer => "Integer",
            DataType::Float => "Float",
            DataType::String => "String",
            DataType::Bool => "Bool",
            DataType::Date => "Date",
            DataType::Currency => "Currency",
            DataType::Null => "Null",
            DataType::Mixed => "Mixed",
        }
    }
    
    /// Проверить совместимость с другим типом
    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        use DataType::*;
        match (self, other) {
            // Числовые типы совместимы между собой
            (Integer, Float) | (Float, Integer) | (Integer, Integer) | (Float, Float) => true,
            // Одинаковые типы совместимы
            (a, b) if a == b => true,
            // Null совместим с любым типом
            (Null, _) | (_, Null) => true,
            // Остальные несовместимы
            _ => false,
        }
    }
}

/// Основное перечисление значений в DataCode
#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Table(Rc<RefCell<Table>>), // Оптимизированное хранение таблиц
    Currency(String), // Хранит оригинальную строку с валютой
    Null,
    Path(PathBuf),
    PathPattern(PathBuf), // Для glob паттернов типа /path/*.csv
}

impl Value {
    /// Создать новое числовое значение
    pub fn number(n: f64) -> Self {
        Value::Number(n)
    }
    
    /// Создать новое строковое значение
    pub fn string<S: Into<String>>(s: S) -> Self {
        Value::String(s.into())
    }
    
    /// Создать новое булево значение
    pub fn bool(b: bool) -> Self {
        Value::Bool(b)
    }
    
    /// Создать новый массив
    pub fn array(elements: Vec<Value>) -> Self {
        Value::Array(elements)
    }
    
    /// Создать новый объект
    pub fn object(map: HashMap<String, Value>) -> Self {
        Value::Object(map)
    }
    
    /// Создать новое значение валюты
    pub fn currency<S: Into<String>>(s: S) -> Self {
        Value::Currency(s.into())
    }
    
    /// Создать null значение
    pub fn null() -> Self {
        Value::Null
    }
    
    /// Создать путь
    pub fn path(path: PathBuf) -> Self {
        Value::Path(path)
    }

    /// Создать паттерн пути
    pub fn path_pattern(path: PathBuf) -> Self {
        Value::PathPattern(path)
    }

    /// Создать новую таблицу с оптимизированным хранением
    pub fn table(table: Table) -> Self {
        Value::Table(Rc::new(RefCell::new(table)))
    }

    /// Создать новую таблицу из существующего Rc<RefCell<Table>>
    pub fn table_from_rc(table: Rc<RefCell<Table>>) -> Self {
        Value::Table(table)
    }
    
    /// Проверить, является ли значение числовым
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Number(_))
    }
    
    /// Проверить, является ли значение пустым
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    
    /// Проверить, является ли значение массивом
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }
    
    /// Проверить, является ли значение объектом
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
    
    /// Проверить, является ли значение таблицей
    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }
    
    /// Получить тип значения
    pub fn get_type(&self) -> DataType {
        DataType::from_value(self)
    }
    
    /// Попытаться получить число
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }
    
    /// Попытаться получить строку
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(s) => Some(s),
            Value::Currency(s) => Some(s),
            _ => None,
        }
    }
    
    /// Попытаться получить булево значение
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Попытаться получить массив
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }
    
    /// Попытаться получить объект
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }
    
    /// Попытаться получить таблицу
    pub fn as_table(&self) -> Option<Rc<RefCell<Table>>> {
        match self {
            Value::Table(table) => Some(table.clone()),
            _ => None,
        }
    }

    /// Попытаться получить ссылку на таблицу для чтения
    pub fn as_table_ref(&self) -> Option<&Rc<RefCell<Table>>> {
        match self {
            Value::Table(table) => Some(table),
            _ => None,
        }
    }
    
    /// Попытаться получить путь
    pub fn as_path(&self) -> Option<&PathBuf> {
        match self {
            Value::Path(path) | Value::PathPattern(path) => Some(path),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Table(a), Value::Table(b)) => {
                // Сравниваем содержимое таблиц
                let a_borrowed = a.borrow();
                let b_borrowed = b.borrow();
                *a_borrowed == *b_borrowed
            },
            (Value::Currency(a), Value::Currency(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Path(a), Value::Path(b)) => a == b,
            (Value::PathPattern(a), Value::PathPattern(b)) => a == b,
            _ => false,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Vec<Value>> for Value {
    fn from(arr: Vec<Value>) -> Self {
        Value::Array(arr)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(obj: HashMap<String, Value>) -> Self {
        Value::Object(obj)
    }
}

impl From<PathBuf> for Value {
    fn from(path: PathBuf) -> Self {
        Value::Path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_from_value() {
        assert_eq!(DataType::from_value(&Value::Number(42.0)), DataType::Integer);
        assert_eq!(DataType::from_value(&Value::Number(3.14)), DataType::Float);
        assert_eq!(DataType::from_value(&Value::String("hello".to_string())), DataType::String);
        assert_eq!(DataType::from_value(&Value::Bool(true)), DataType::Bool);
        assert_eq!(DataType::from_value(&Value::Null), DataType::Null);
    }

    #[test]
    fn test_data_type_is_numeric() {
        assert!(DataType::Integer.is_numeric());
        assert!(DataType::Float.is_numeric());
        assert!(!DataType::String.is_numeric());
        assert!(!DataType::Bool.is_numeric());
    }

    #[test]
    fn test_data_type_compatibility() {
        assert!(DataType::Integer.is_compatible_with(&DataType::Float));
        assert!(DataType::Float.is_compatible_with(&DataType::Integer));
        assert!(DataType::String.is_compatible_with(&DataType::String));
        assert!(!DataType::String.is_compatible_with(&DataType::Integer));
        assert!(DataType::Null.is_compatible_with(&DataType::String));
    }

    #[test]
    fn test_value_constructors() {
        assert_eq!(Value::number(42.0), Value::Number(42.0));
        assert_eq!(Value::string("hello"), Value::String("hello".to_string()));
        assert_eq!(Value::bool(true), Value::Bool(true));
        assert_eq!(Value::null(), Value::Null);
    }

    #[test]
    fn test_value_type_checks() {
        let num = Value::Number(42.0);
        assert!(num.is_numeric());
        assert!(!num.is_null());
        assert!(!num.is_array());

        let null_val = Value::Null;
        assert!(null_val.is_null());
        assert!(!null_val.is_numeric());
    }

    #[test]
    fn test_value_accessors() {
        let num = Value::Number(42.0);
        assert_eq!(num.as_number(), Some(42.0));
        assert_eq!(num.as_string(), None);

        let string = Value::String("hello".to_string());
        assert_eq!(string.as_string(), Some(&"hello".to_string()));
        assert_eq!(string.as_number(), None);
    }

    #[test]
    fn test_value_from_conversions() {
        assert_eq!(Value::from(42.0), Value::Number(42.0));
        assert_eq!(Value::from(42i32), Value::Number(42.0));
        assert_eq!(Value::from("hello"), Value::String("hello".to_string()));
        assert_eq!(Value::from(true), Value::Bool(true));
    }

    #[test]
    fn test_data_type_to_string() {
        assert_eq!(DataType::Integer.to_string(), "Integer");
        assert_eq!(DataType::Float.to_string(), "Float");
        assert_eq!(DataType::String.to_string(), "String");
        assert_eq!(DataType::Bool.to_string(), "Bool");
    }
}
