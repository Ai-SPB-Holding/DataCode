// Упрощенный модуль параллельных операций для DataCode
// Использует Rayon для базовых параллельных операций без сложных зависимостей

use crate::value::{Value, Table};
use crate::error::{Result, DataCodeError};
use std::rc::Rc;
use std::cell::RefCell;


/// Упрощенный движок параллельных операций
pub struct SimpleParallelEngine {
    operation_count: usize,
}

impl SimpleParallelEngine {
    /// Создать новый движок
    pub fn new() -> Self {
        Self {
            operation_count: 0,
        }
    }
    
    /// Параллельная операция map для массива
    pub fn parallel_map<F>(&mut self, values: Vec<Value>, func: F) -> Result<Vec<Value>>
    where
        F: Fn(&Value) -> Result<Value> + Send + Sync,
    {
        self.operation_count += 1;
        
        // Используем обычный итератор вместо параллельного для совместимости
        values.iter()
            .map(|value| func(value))
            .collect::<Result<Vec<Value>>>()
    }
    
    /// Параллельная фильтрация массива
    pub fn parallel_filter<F>(&mut self, values: Vec<Value>, predicate: F) -> Result<Vec<Value>>
    where
        F: Fn(&Value) -> bool + Send + Sync,
    {
        self.operation_count += 1;
        
        Ok(values.into_iter()
            .filter(|value| predicate(value))
            .collect())
    }
    
    /// Параллельная фильтрация таблицы
    pub fn parallel_table_filter<F>(&mut self, table: Rc<RefCell<Table>>, predicate: F) -> Result<Rc<RefCell<Table>>>
    where
        F: Fn(&Vec<Value>) -> bool + Send + Sync,
    {
        self.operation_count += 1;
        
        let table_borrowed = table.borrow();
        let column_names = table_borrowed.column_names.clone();
        let rows = table_borrowed.rows.clone();
        drop(table_borrowed);
        
        let filtered_rows: Vec<Vec<Value>> = rows.into_iter()
            .filter(|row| predicate(row))
            .collect();
        
        let mut new_table = Table::new(column_names);
        for row in filtered_rows {
            new_table.add_row(row)?;
        }
        
        Ok(Rc::new(RefCell::new(new_table)))
    }
    
    /// Параллельная сортировка таблицы
    pub fn parallel_sort_table(&mut self, table: Rc<RefCell<Table>>, column_name: &str, ascending: bool) -> Result<Rc<RefCell<Table>>> {
        self.operation_count += 1;
        
        let table_borrowed = table.borrow();
        let column_names = table_borrowed.column_names.clone();
        let mut rows = table_borrowed.rows.clone();
        
        // Найдем индекс колонки для сортировки
        let column_index = column_names.iter()
            .position(|name| name == column_name)
            .ok_or_else(|| DataCodeError::runtime_error(&format!("Column '{}' not found", column_name), 0))?;
        
        drop(table_borrowed);
        
        // Обычная сортировка
        rows.sort_by(|a, b| {
            if column_index >= a.len() || column_index >= b.len() {
                return std::cmp::Ordering::Equal;
            }
            
            let ordering = self.compare_values(&a[column_index], &b[column_index]);
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
        
        let mut new_table = Table::new(column_names);
        for row in rows {
            new_table.add_row(row)?;
        }
        
        Ok(Rc::new(RefCell::new(new_table)))
    }
    
    /// Параллельная агрегация
    pub fn parallel_aggregate(&mut self, table: Rc<RefCell<Table>>, operation: &str, column_name: &str) -> Result<Value> {
        self.operation_count += 1;
        
        let table_borrowed = table.borrow();
        let column_names = &table_borrowed.column_names;
        let rows = &table_borrowed.rows;
        
        let column_index = column_names.iter()
            .position(|name| name == column_name)
            .ok_or_else(|| DataCodeError::runtime_error(&format!("Column '{}' not found", column_name), 0))?;
        
        let values: Vec<&Value> = rows.iter()
            .filter_map(|row| row.get(column_index))
            .collect();
        
        match operation {
            "sum" => {
                let sum: f64 = values.iter()
                    .filter_map(|v| match v {
                        Value::Number(n) => Some(*n),
                        _ => None,
                    })
                    .sum();
                Ok(Value::Number(sum))
            },
            "avg" | "mean" => {
                let numbers: Vec<f64> = values.iter()
                    .filter_map(|v| match v {
                        Value::Number(n) => Some(*n),
                        _ => None,
                    })
                    .collect();
                
                if numbers.is_empty() {
                    Ok(Value::Null)
                } else {
                    let avg = numbers.iter().sum::<f64>() / numbers.len() as f64;
                    Ok(Value::Number(avg))
                }
            },
            "count" => {
                Ok(Value::Number(values.len() as f64))
            },
            "max" => {
                let max = values.iter()
                    .filter_map(|v| match v {
                        Value::Number(n) => Some(*n),
                        _ => None,
                    })
                    .fold(f64::NEG_INFINITY, |a, b| a.max(b));
                
                if max == f64::NEG_INFINITY {
                    Ok(Value::Null)
                } else {
                    Ok(Value::Number(max))
                }
            },
            "min" => {
                let min = values.iter()
                    .filter_map(|v| match v {
                        Value::Number(n) => Some(*n),
                        _ => None,
                    })
                    .fold(f64::INFINITY, |a, b| a.min(b));
                
                if min == f64::INFINITY {
                    Ok(Value::Null)
                } else {
                    Ok(Value::Number(min))
                }
            },
            _ => Err(DataCodeError::runtime_error(&format!("Unsupported aggregation operation: {}", operation), 0))
        }
    }
    
    /// Сравнить два значения для сортировки
    fn compare_values(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
            (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
            (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
            (Value::Null, _) => std::cmp::Ordering::Less,
            (_, Value::Null) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
    
    /// Получить количество выполненных операций
    pub fn get_operation_count(&self) -> usize {
        self.operation_count
    }
    
    /// Сбросить счетчик операций
    pub fn reset_operation_count(&mut self) {
        self.operation_count = 0;
    }
}

impl Default for SimpleParallelEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Table, Value};
    
    #[test]
    fn test_simple_parallel_engine_creation() {
        let engine = SimpleParallelEngine::new();
        assert_eq!(engine.get_operation_count(), 0);
    }
    
    #[test]
    fn test_parallel_map() {
        let mut engine = SimpleParallelEngine::new();
        let values = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ];
        
        let result = engine.parallel_map(values, |v| {
            match v {
                Value::Number(n) => Ok(Value::Number(n * 2.0)),
                _ => Ok(v.clone()),
            }
        });
        
        assert!(result.is_ok());
        let mapped = result.unwrap();
        assert_eq!(mapped.len(), 3);
        assert_eq!(engine.get_operation_count(), 1);
        
        if let Value::Number(n) = &mapped[0] {
            assert_eq!(*n, 2.0);
        }
    }
    
    #[test]
    fn test_parallel_filter() {
        let mut engine = SimpleParallelEngine::new();
        let values = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ];
        
        let result = engine.parallel_filter(values, |v| {
            match v {
                Value::Number(n) => *n > 2.0,
                _ => false,
            }
        });
        
        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(engine.get_operation_count(), 1);
    }
    
    #[test]
    fn test_parallel_aggregate() {
        let mut engine = SimpleParallelEngine::new();
        let mut table = Table::new(vec!["id".to_string(), "value".to_string()]);
        
        table.add_row(vec![Value::Number(1.0), Value::Number(10.0)]).unwrap();
        table.add_row(vec![Value::Number(2.0), Value::Number(20.0)]).unwrap();
        table.add_row(vec![Value::Number(3.0), Value::Number(30.0)]).unwrap();
        
        let table_rc = Rc::new(RefCell::new(table));
        
        // Тест суммы
        let sum_result = engine.parallel_aggregate(table_rc.clone(), "sum", "value");
        assert!(sum_result.is_ok());
        if let Value::Number(sum) = sum_result.unwrap() {
            assert_eq!(sum, 60.0);
        }
        
        // Тест среднего
        let avg_result = engine.parallel_aggregate(table_rc.clone(), "avg", "value");
        assert!(avg_result.is_ok());
        if let Value::Number(avg) = avg_result.unwrap() {
            assert_eq!(avg, 20.0);
        }
        
        // Тест количества
        let count_result = engine.parallel_aggregate(table_rc, "count", "value");
        assert!(count_result.is_ok());
        if let Value::Number(count) = count_result.unwrap() {
            assert_eq!(count, 3.0);
        }
    }
}
