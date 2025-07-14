// Логика индексации и доступа к элементам в DataCode
// Обрабатывает доступ к элементам массивов, объектов и таблиц

use crate::value::Value;
use crate::error::{DataCodeError, Result};
use super::Evaluator;

/// Обработчик индексации
pub struct IndexingHandler<'a> {
    evaluator: &'a Evaluator<'a>,
}

impl<'a> IndexingHandler<'a> {
    /// Создать новый обработчик индексации
    pub fn new(evaluator: &'a Evaluator<'a>) -> Self {
        Self { evaluator }
    }
    
    /// Вычислить индексацию
    pub fn evaluate(&self, object: &Value, index: &Value) -> Result<Value> {
        match (object, index) {
            (Value::Array(arr), Value::Number(n)) => self.index_array(arr, *n),
            (Value::String(s), Value::Number(n)) => self.index_string(s, *n),
            (Value::Object(obj), Value::String(key)) => self.index_object(obj, key),
            (Value::Table(table), Value::Number(n)) => self.index_table_row(table, *n),
            (Value::Table(table), Value::String(column_name)) => self.index_table_column(table, column_name),
            _ => Err(DataCodeError::type_error("indexable type", "other", self.evaluator.line())),
        }
    }
    
    /// Индексация массива
    fn index_array(&self, arr: &[Value], index: f64) -> Result<Value> {
        if index.fract() != 0.0 {
            return Err(DataCodeError::runtime_error("Array index must be an integer", self.evaluator.line()));
        }
        
        let idx = index as i64;
        let len = arr.len() as i64;
        
        // Поддержка отрицательных индексов
        let actual_idx = if idx < 0 {
            len + idx
        } else {
            idx
        };
        
        if actual_idx < 0 || actual_idx >= len {
            Err(DataCodeError::runtime_error(
                &format!("Array index {} out of bounds (length: {})", idx, len),
                self.evaluator.line()
            ))
        } else {
            Ok(arr[actual_idx as usize].clone())
        }
    }
    
    /// Индексация строки (получение символа)
    fn index_string(&self, s: &str, index: f64) -> Result<Value> {
        if index.fract() != 0.0 {
            return Err(DataCodeError::runtime_error("String index must be an integer", self.evaluator.line()));
        }
        
        let chars: Vec<char> = s.chars().collect();
        let idx = index as i64;
        let len = chars.len() as i64;
        
        // Поддержка отрицательных индексов
        let actual_idx = if idx < 0 {
            len + idx
        } else {
            idx
        };
        
        if actual_idx < 0 || actual_idx >= len {
            Err(DataCodeError::runtime_error(
                &format!("String index {} out of bounds (length: {})", idx, len),
                self.evaluator.line()
            ))
        } else {
            Ok(Value::String(chars[actual_idx as usize].to_string()))
        }
    }
    
    /// Индексация объекта
    fn index_object(&self, obj: &std::collections::HashMap<String, Value>, key: &str) -> Result<Value> {
        obj.get(key)
            .cloned()
            .ok_or_else(|| DataCodeError::runtime_error(&format!("Key '{}' not found in object", key), self.evaluator.line()))
    }
    
    /// Индексация таблицы по номеру строки
    fn index_table_row(&self, table: &crate::value::Table, index: f64) -> Result<Value> {
        if index.fract() != 0.0 {
            return Err(DataCodeError::runtime_error("Table row index must be an integer", self.evaluator.line()));
        }
        
        let idx = index as i64;
        let len = table.rows.len() as i64;
        
        // Поддержка отрицательных индексов
        let actual_idx = if idx < 0 {
            len + idx
        } else {
            idx
        };
        
        if actual_idx < 0 || actual_idx >= len {
            Err(DataCodeError::runtime_error(
                &format!("Table row index {} out of bounds (rows: {})", idx, len),
                self.evaluator.line()
            ))
        } else {
            // Возвращаем строку как массив
            Ok(Value::Array(table.rows[actual_idx as usize].clone()))
        }
    }
    
    /// Индексация таблицы по имени колонки
    fn index_table_column(&self, table: &crate::value::Table, column_name: &str) -> Result<Value> {
        // Находим индекс колонки
        let col_index = table.column_names.iter()
            .position(|name| name == column_name)
            .ok_or_else(|| DataCodeError::runtime_error(
                &format!("Column '{}' not found in table", column_name),
                self.evaluator.line()
            ))?;
        
        // Извлекаем все значения из этой колонки
        let column_values: Vec<Value> = table.rows.iter()
            .map(|row| row.get(col_index).cloned().unwrap_or(Value::Null))
            .collect();
        
        Ok(Value::Array(column_values))
    }
}

/// Обработчик доступа к членам объекта
pub struct MemberAccessHandler<'a> {
    evaluator: &'a Evaluator<'a>,
}

impl<'a> MemberAccessHandler<'a> {
    /// Создать новый обработчик доступа к членам
    pub fn new(evaluator: &'a Evaluator<'a>) -> Self {
        Self { evaluator }
    }
    
    /// Вычислить доступ к члену объекта
    pub fn evaluate(&self, object: &Value, member: &str) -> Result<Value> {
        match object {
            Value::Object(obj) => self.access_object_member(obj, member),
            Value::Table(table) => self.access_table_member(table, member),
            Value::Array(arr) => self.access_array_member(arr, member),
            Value::String(s) => self.access_string_member(s, member),
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot access member '{}' on {:?}", member, object),
                self.evaluator.line()
            )),
        }
    }
    
    /// Доступ к члену объекта
    fn access_object_member(&self, obj: &std::collections::HashMap<String, Value>, member: &str) -> Result<Value> {
        obj.get(member)
            .cloned()
            .ok_or_else(|| DataCodeError::runtime_error(&format!("Member '{}' not found", member), self.evaluator.line()))
    }
    
    /// Доступ к свойствам таблицы
    fn access_table_member(&self, table: &crate::value::Table, member: &str) -> Result<Value> {
        match member {
            "rows" => Ok(Value::Number(table.rows.len() as f64)),
            "columns" => Ok(Value::Number(table.columns.len() as f64)),
            "column_names" => {
                let names: Vec<Value> = table.column_names.iter()
                    .map(|name| Value::String(name.clone()))
                    .collect();
                Ok(Value::Array(names))
            }
            _ => {
                // Попробуем найти колонку с таким именем
                if table.column_names.contains(&member.to_string()) {
                    let indexing_handler = IndexingHandler::new(self.evaluator);
                    indexing_handler.index_table_column(table, member)
                } else {
                    Err(DataCodeError::runtime_error(
                        &format!("Table has no member or column '{}'", member),
                        self.evaluator.line()
                    ))
                }
            }
        }
    }
    
    /// Доступ к свойствам массива
    fn access_array_member(&self, arr: &[Value], member: &str) -> Result<Value> {
        match member {
            "length" | "len" => Ok(Value::Number(arr.len() as f64)),
            "first" => {
                if arr.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(arr[0].clone())
                }
            }
            "last" => {
                if arr.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(arr[arr.len() - 1].clone())
                }
            }
            _ => Err(DataCodeError::runtime_error(
                &format!("Array has no member '{}'", member),
                self.evaluator.line()
            )),
        }
    }
    
    /// Доступ к свойствам строки
    fn access_string_member(&self, s: &str, member: &str) -> Result<Value> {
        match member {
            "length" | "len" => Ok(Value::Number(s.chars().count() as f64)),
            "upper" => Ok(Value::String(s.to_uppercase())),
            "lower" => Ok(Value::String(s.to_lowercase())),
            "trim" => Ok(Value::String(s.trim().to_string())),
            _ => Err(DataCodeError::runtime_error(
                &format!("String has no member '{}'", member),
                self.evaluator.line()
            )),
        }
    }
}

/// Трейт для индексируемых типов
pub trait Indexable {
    /// Получить элемент по индексу
    fn get_by_index(&self, index: &Value) -> Result<Value>;
    
    /// Проверить, поддерживает ли тип индексацию
    fn supports_indexing(&self) -> bool;
}

impl Indexable for Value {
    fn get_by_index(&self, index: &Value) -> Result<Value> {
        match (self, index) {
            (Value::Array(arr), Value::Number(n)) => {
                let idx = *n as usize;
                arr.get(idx)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error("Index out of bounds", 0))
            }
            (Value::Object(obj), Value::String(key)) => {
                obj.get(key)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error("Key not found", 0))
            }
            _ => Err(DataCodeError::runtime_error("Type does not support indexing", 0)),
        }
    }
    
    fn supports_indexing(&self) -> bool {
        matches!(self, Value::Array(_) | Value::Object(_) | Value::Table(_) | Value::String(_))
    }
}

/// Трейт для типов с членами
pub trait HasMembers {
    /// Получить член по имени
    fn get_member(&self, member: &str) -> Result<Value>;
    
    /// Получить список всех членов
    fn get_member_names(&self) -> Vec<String>;
}

impl HasMembers for Value {
    fn get_member(&self, member: &str) -> Result<Value> {
        match self {
            Value::Object(obj) => {
                obj.get(member)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error("Member not found", 0))
            }
            Value::Array(arr) => {
                match member {
                    "length" | "len" => Ok(Value::Number(arr.len() as f64)),
                    _ => Err(DataCodeError::runtime_error("Unknown array member", 0)),
                }
            }
            _ => Err(DataCodeError::runtime_error("Type does not have members", 0)),
        }
    }
    
    fn get_member_names(&self) -> Vec<String> {
        match self {
            Value::Object(obj) => obj.keys().cloned().collect(),
            Value::Array(_) => vec!["length".to_string(), "len".to_string()],
            Value::String(_) => vec!["length".to_string(), "len".to_string(), "upper".to_string(), "lower".to_string()],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_evaluator() -> Evaluator<'static> {
        let variables = HashMap::new();
        let static_vars = Box::leak(Box::new(variables));
        Evaluator::new(static_vars, 1)
    }

    #[test]
    fn test_array_indexing() {
        let evaluator = create_test_evaluator();
        let handler = IndexingHandler::new(&evaluator);
        
        let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        
        let result = handler.index_array(&arr, 0.0).unwrap();
        assert_eq!(result, Value::Number(1.0));
        
        let result = handler.index_array(&arr, 2.0).unwrap();
        assert_eq!(result, Value::Number(3.0));
        
        // Отрицательные индексы
        let result = handler.index_array(&arr, -1.0).unwrap();
        assert_eq!(result, Value::Number(3.0));
        
        // Выход за границы
        let error = handler.index_array(&arr, 5.0);
        assert!(error.is_err());
    }

    #[test]
    fn test_string_indexing() {
        let evaluator = create_test_evaluator();
        let handler = IndexingHandler::new(&evaluator);
        
        let result = handler.index_string("hello", 0.0).unwrap();
        assert_eq!(result, Value::String("h".to_string()));
        
        let result = handler.index_string("hello", 4.0).unwrap();
        assert_eq!(result, Value::String("o".to_string()));
        
        // Отрицательные индексы
        let result = handler.index_string("hello", -1.0).unwrap();
        assert_eq!(result, Value::String("o".to_string()));
    }

    #[test]
    fn test_object_indexing() {
        let evaluator = create_test_evaluator();
        let handler = IndexingHandler::new(&evaluator);
        
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), Value::String("test".to_string()));
        obj.insert("value".to_string(), Value::Number(42.0));
        
        let result = handler.index_object(&obj, "name").unwrap();
        assert_eq!(result, Value::String("test".to_string()));
        
        let error = handler.index_object(&obj, "unknown");
        assert!(error.is_err());
    }

    #[test]
    fn test_member_access() {
        let evaluator = create_test_evaluator();
        let handler = MemberAccessHandler::new(&evaluator);
        
        let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        
        let result = handler.access_array_member(&arr, "length").unwrap();
        assert_eq!(result, Value::Number(3.0));
        
        let result = handler.access_array_member(&arr, "first").unwrap();
        assert_eq!(result, Value::Number(1.0));
        
        let result = handler.access_array_member(&arr, "last").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_string_member_access() {
        let evaluator = create_test_evaluator();
        let handler = MemberAccessHandler::new(&evaluator);
        
        let result = handler.access_string_member("Hello", "length").unwrap();
        assert_eq!(result, Value::Number(5.0));
        
        let result = handler.access_string_member("Hello", "upper").unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
        
        let result = handler.access_string_member("Hello", "lower").unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_indexable_trait() {
        let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert!(arr.supports_indexing());
        
        let result = arr.get_by_index(&Value::Number(0.0)).unwrap();
        assert_eq!(result, Value::Number(1.0));
        
        let num = Value::Number(42.0);
        assert!(!num.supports_indexing());
    }

    #[test]
    fn test_has_members_trait() {
        let arr = Value::Array(vec![Value::Number(1.0)]);
        let members = arr.get_member_names();
        assert!(members.contains(&"length".to_string()));
        
        let result = arr.get_member("length").unwrap();
        assert_eq!(result, Value::Number(1.0));
    }
}
