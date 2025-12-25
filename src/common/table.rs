// Структура данных для таблиц

use crate::common::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Table {
    pub columns: HashMap<String, Vec<Value>>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub name: Option<String>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            headers: Vec::new(),
            rows: Vec::new(),
            name: None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn from_data(data: Vec<Vec<Value>>, headers: Option<Vec<String>>) -> Self {
        let mut table = Self::new();
        
        if data.is_empty() {
            if let Some(headers) = headers {
                table.headers = headers;
                // Создаем пустые колонки для каждой заголовка
                for header in &table.headers {
                    table.columns.insert(header.clone(), Vec::new());
                }
            }
            return table;
        }

        // Определяем количество колонок из первой строки
        let num_columns = data[0].len();

        // Генерируем заголовки, если не предоставлены
        let headers = headers.unwrap_or_else(|| {
            (0..num_columns)
                .map(|i| format!("Column_{}", i))
                .collect()
        });

        table.headers = headers.clone();

        // Оптимизация: создаем колонки напрямую из data с предварительным выделением памяти
        // Это избегает лишних копирований и улучшает производительность
        let num_rows = data.len();
        for (i, header) in headers.iter().enumerate() {
            let mut column = Vec::with_capacity(num_rows);
            for row in &data {
                if i < row.len() {
                    column.push(row[i].clone());
                } else {
                    column.push(Value::Null);
                }
            }
            table.columns.insert(header.clone(), column);
        }

        // Сохраняем строки
        table.rows = data;

        table
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    pub fn get_column(&self, name: &str) -> Option<&Vec<Value>> {
        self.columns.get(name)
    }

    pub fn get_row(&self, index: usize) -> Option<&Vec<Value>> {
        self.rows.get(index)
    }
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        // Сравниваем заголовки
        if self.headers != other.headers {
            return false;
        }

        // Сравниваем количество строк
        if self.rows.len() != other.rows.len() {
            return false;
        }

        // Сравниваем данные по колонкам
        for header in &self.headers {
            let self_col = self.columns.get(header);
            let other_col = other.columns.get(header);
            
            match (self_col, other_col) {
                (Some(a), Some(b)) => {
                    if a != b {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
    }
}

