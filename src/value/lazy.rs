// Ленивая обработка данных для оптимизации производительности DataCode
// Реализует LazyTable для отложенного выполнения операций

use std::rc::Rc;
use std::cell::RefCell;
use super::table::Table;
use super::types::Value;
use crate::error::{DataCodeError, Result};
use regex;

/// Операции, которые могут быть отложены
#[derive(Clone, Debug)]
pub enum LazyOperation {
    Filter {
        condition: String,
        line: usize,
    },
    Where {
        column: String,
        operator: String,
        value: Value,
        line: usize,
    },
    Select {
        columns: Vec<String>,
        line: usize,
    },
    Head {
        n: usize,
        _line: usize,
    },
    Tail {
        n: usize,
        _line: usize,
    },
    #[allow(dead_code)]
    Sort {
        column: String,
        ascending: bool,
        _line: usize,
    },
}

/// Ленивая таблица для отложенного выполнения операций
#[derive(Clone, Debug)]
pub struct LazyTable {
    /// Исходная таблица
    source: Rc<RefCell<Table>>,
    /// Цепочка операций для выполнения
    operations: Vec<LazyOperation>,
}

impl LazyTable {
    /// Создать новую ленивую таблицу
    pub fn new(source: Rc<RefCell<Table>>) -> Self {
        Self {
            source,
            operations: Vec::new(),
        }
    }
    
    /// Добавить операцию фильтрации
    pub fn filter(mut self, condition: String, line: usize) -> Self {
        self.operations.push(LazyOperation::Filter { condition, line });
        self
    }
    
    /// Добавить операцию WHERE
    pub fn where_op(mut self, column: String, operator: String, value: Value, line: usize) -> Self {
        self.operations.push(LazyOperation::Where { column, operator, value, line });
        self
    }
    
    /// Добавить операцию выбора колонок
    pub fn select(mut self, columns: Vec<String>, line: usize) -> Self {
        self.operations.push(LazyOperation::Select { columns, line });
        self
    }
    
    /// Добавить операцию head
    pub fn head(mut self, n: usize, line: usize) -> Self {
        self.operations.push(LazyOperation::Head { n, _line: line });
        self
    }
    
    /// Добавить операцию tail
    pub fn tail(mut self, n: usize, line: usize) -> Self {
        self.operations.push(LazyOperation::Tail { n, _line: line });
        self
    }
    
    /// Добавить операцию сортировки
    #[allow(dead_code)]
    pub fn sort(mut self, column: String, ascending: bool, line: usize) -> Self {
        self.operations.push(LazyOperation::Sort { column, ascending, _line: line });
        self
    }
    
    /// Материализовать таблицу - выполнить все отложенные операции
    pub fn materialize(self) -> Result<Table> {
        let mut current_table = {
            let source_borrowed = self.source.borrow();
            source_borrowed.clone()
        };
        
        // Выполняем операции в порядке добавления
        for operation in &self.operations {
            current_table = self.apply_operation(current_table, operation.clone())?;
        }
        
        Ok(current_table)
    }
    
    /// Применить одну операцию к таблице
    fn apply_operation(&self, mut table: Table, operation: LazyOperation) -> Result<Table> {
        match operation {
            LazyOperation::Filter { condition, line } => {
                self.apply_filter_operation(table, &condition, line)
            },
            LazyOperation::Where { column, operator, value, line } => {
                self.apply_where_operation(table, &column, &operator, &value, line)
            },
            LazyOperation::Select { columns, line } => {
                self.apply_select_operation(table, &columns, line)
            },
            LazyOperation::Head { n, _line: _ } => {
                // Простая реализация head - берем первые n строк
                table.rows.truncate(n);
                Ok(table)
            },
            LazyOperation::Tail { n, _line: _ } => {
                // Простая реализация tail - берем последние n строк
                let total_rows = table.rows.len();
                if total_rows > n {
                    table.rows = table.rows.into_iter().skip(total_rows - n).collect();
                }
                Ok(table)
            },
            LazyOperation::Sort { column, ascending, _line } => {
                self.apply_sort_operation(table, &column, ascending, _line)
            },
        }
    }
    
    /// Применить операцию фильтрации
    fn apply_filter_operation(&self, table: Table, condition: &str, line: usize) -> Result<Table> {
        use std::collections::HashMap;
        
        let mut filtered_table = Table::new(table.column_names.clone());
        
        for row in &table.rows {
            // Создаем контекст переменных для текущей строки
            let mut row_context = HashMap::new();
            for (i, col_name) in table.column_names.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    row_context.insert(col_name.clone(), value.clone());
                }
            }
            
            // Парсим выражение
            let mut parser = crate::parser::Parser::new(condition);
            let expr = match parser.parse_expression() {
                Ok(expr) => expr,
                Err(e) => return Err(DataCodeError::runtime_error(&format!("Ошибка парсинга условия: {}", e), line)),
            };
            
            // Создаем evaluator с контекстом строки
            let evaluator = crate::evaluator::Evaluator::new(&row_context, line);
            
            // Вычисляем условие
            match evaluator.evaluate(&expr) {
                Ok(Value::Bool(true)) => {
                    if let Err(e) = filtered_table.add_row(row.clone()) {
                        return Err(DataCodeError::runtime_error(&e, line));
                    }
                }
                Ok(Value::Bool(false)) => {
                    // Строка не прошла фильтр
                }
                Ok(_) => {
                    return Err(DataCodeError::runtime_error("Условие фильтрации должно возвращать boolean", line));
                }
                Err(e) => {
                    return Err(DataCodeError::runtime_error(&format!("Ошибка в условии фильтрации: {}", e), line));
                }
            }
        }
        
        Ok(filtered_table)
    }
    
    /// Применить операцию WHERE
    fn apply_where_operation(&self, table: Table, column: &str, operator: &str, value: &Value, line: usize) -> Result<Table> {
        // Находим индекс колонки
        let col_index = table.column_names.iter()
            .position(|name| name == column)
            .ok_or_else(|| DataCodeError::runtime_error(
                &format!("Колонка '{}' не найдена в таблице", column),
                line
            ))?;
        
        let mut filtered_table = Table::new(table.column_names.clone());
        
        for row in &table.rows {
            if let Some(row_value) = row.get(col_index) {
                let matches = match operator {
                    "=" | "==" => self.values_equal(row_value, value),
                    "!=" | "<>" => !self.values_equal(row_value, value),
                    ">" => self.compare_values_for_filter(row_value, value) == std::cmp::Ordering::Greater,
                    "<" => self.compare_values_for_filter(row_value, value) == std::cmp::Ordering::Less,
                    ">=" => {
                        let cmp = self.compare_values_for_filter(row_value, value);
                        cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal
                    },
                    "<=" => {
                        let cmp = self.compare_values_for_filter(row_value, value);
                        cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal
                    },
                    "LIKE" => match (row_value, value) {
                        (Value::String(s1), Value::String(s2)) => {
                            // Простая реализация LIKE с поддержкой % и _
                            let pattern = s2.replace('%', ".*").replace('_', ".");
                            match regex::Regex::new(&pattern) {
                                Ok(re) => re.is_match(s1),
                                Err(_) => false,
                            }
                        }
                        _ => false,
                    },
                    "IN" => match value {
                        Value::Array(arr) => arr.iter().any(|v| self.values_equal(row_value, v)),
                        _ => false,
                    },
                    _ => return Err(DataCodeError::runtime_error(
                        &format!("Неподдерживаемый оператор: {}", operator),
                        line
                    )),
                };
                
                if matches {
                    if let Err(e) = filtered_table.add_row(row.clone()) {
                        return Err(DataCodeError::runtime_error(&e, line));
                    }
                }
            }
        }
        
        Ok(filtered_table)
    }
    
    /// Применить операцию выбора колонок
    fn apply_select_operation(&self, table: Table, columns: &[String], line: usize) -> Result<Table> {
        let mut selected_table = Table::new(columns.to_vec());
        
        let column_indices: Result<Vec<usize>> = columns.iter()
            .map(|col_name| {
                table.column_names.iter()
                    .position(|name| name == col_name)
                    .ok_or_else(|| DataCodeError::runtime_error(
                        &format!("Column '{}' not found", col_name),
                        line
                    ))
            })
            .collect();
        
        let column_indices = column_indices?;
        
        for row in &table.rows {
            let selected_row: Vec<Value> = column_indices.iter()
                .map(|&index| row.get(index).cloned().unwrap_or(Value::Null))
                .collect();
            
            if let Err(e) = selected_table.add_row(selected_row) {
                return Err(DataCodeError::runtime_error(&e, line));
            }
        }
        
        Ok(selected_table)
    }
    
    /// Применить операцию сортировки
    fn apply_sort_operation(&self, mut table: Table, column: &str, ascending: bool, line: usize) -> Result<Table> {
        let col_index = table.column_names.iter()
            .position(|name| name == column)
            .ok_or_else(|| DataCodeError::runtime_error(
                &format!("Column '{}' not found", column),
                line
            ))?;
        
        table.rows.sort_by(|a, b| {
            let val_a = a.get(col_index).unwrap_or(&Value::Null);
            let val_b = b.get(col_index).unwrap_or(&Value::Null);
            
            let cmp = self.compare_values_for_filter(val_a, val_b);
            if ascending { cmp } else { cmp.reverse() }
        });
        
        Ok(table)
    }
    
    /// Сравнить значения для фильтрации
    fn compare_values_for_filter(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => n1.partial_cmp(n2).unwrap_or(Ordering::Equal),
            (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
            (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
    
    /// Проверить равенство значений
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Currency(c1), Value::Currency(c2)) => c1 == c2,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}
