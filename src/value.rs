use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    Integer,
    Float,
    String,
    Bool,
    Date,
    Null,
    Mixed,
}

impl DataType {
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
                // Попытка определить, является ли строка датой
                if is_date_string(s) {
                    DataType::Date
                } else {
                    DataType::String
                }
            }
            Value::Bool(_) => DataType::Bool,
            Value::Null => DataType::Null,
            _ => DataType::Mixed,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    pub name: String,
    pub inferred_type: DataType,
    pub type_counts: HashMap<DataType, usize>,
    pub total_values: usize,
    pub warnings: Vec<String>,
}

impl TableColumn {
    pub fn new(name: String) -> Self {
        Self {
            name,
            inferred_type: DataType::Null,
            type_counts: HashMap::new(),
            total_values: 0,
            warnings: Vec::new(),
        }
    }

    pub fn add_value(&mut self, value: &Value) {
        let data_type = DataType::from_value(value);
        *self.type_counts.entry(data_type.clone()).or_insert(0) += 1;
        self.total_values += 1;

        // Определяем основной тип на основе большинства
        self.infer_primary_type();
    }

    fn infer_primary_type(&mut self) {
        if self.total_values == 0 {
            return;
        }

        // Специальная логика для числовых типов - объединяем Integer и Float
        let integer_count = self.type_counts.get(&DataType::Integer).unwrap_or(&0);
        let float_count = self.type_counts.get(&DataType::Float).unwrap_or(&0);
        let numeric_count = integer_count + float_count;

        // Находим тип с наибольшим количеством значений
        let mut max_count = 0;
        let mut primary_type = DataType::Null;

        // Если есть числовые данные, рассматриваем их как единый тип
        if numeric_count > 0 {
            max_count = numeric_count;
            // Если есть хотя бы одно float значение, считаем всю колонку Float
            // Это позволяет избежать предупреждений о смешанных числовых типах
            primary_type = if *float_count > 0 {
                DataType::Float
            } else {
                DataType::Integer
            };


        }

        // Проверяем остальные типы только если числовые типы не доминируют
        if numeric_count == 0 {
            for (data_type, count) in &self.type_counts {
                // Пропускаем числовые типы, так как мы их уже обработали
                if matches!(data_type, DataType::Integer | DataType::Float) {
                    continue;
                }

                if *count > max_count {
                    max_count = *count;
                    primary_type = data_type.clone();
                }
            }
        }

        self.inferred_type = primary_type;

        // Генерируем предупреждения о неоднородности
        self.warnings.clear();

        // Для числовых типов (Integer и Float) НЕ генерируем предупреждения
        // так как они считаются совместимыми числовыми типами
        if matches!(self.inferred_type, DataType::Integer | DataType::Float) {
            let non_numeric_count = self.total_values - numeric_count;



            // Предупреждение только если есть НЕ-числовые значения (String, Bool, etc.)
            if non_numeric_count > 0 {
                let percentage = (non_numeric_count as f64 / self.total_values as f64) * 100.0;
                self.warnings.push(format!(
                    "Колонка '{}' содержит неоднородные данные: {:.1}% значений не являются числовыми",
                    self.name, percentage
                ));
            }
            // НЕ генерируем предупреждения для смешанных Integer/Float - это нормально!
        } else {
            // Для не-числовых типов используем старую логику
            // НО если основной тип Integer или Float, то учитываем числовую совместимость
            if matches!(self.inferred_type, DataType::Integer | DataType::Float) {
                // Даже если основной тип Integer или Float, считаем их совместимыми
                let non_numeric_count = self.total_values - numeric_count;
                if non_numeric_count > 0 {
                    let percentage = (non_numeric_count as f64 / self.total_values as f64) * 100.0;
                    self.warnings.push(format!(
                        "Колонка '{}' содержит неоднородные данные: {:.1}% значений не являются числовыми",
                        self.name, percentage
                    ));
                }
            } else {
                // Для действительно не-числовых типов
                let non_primary_count = self.total_values - max_count;
                if non_primary_count > 0 {
                    let percentage = (non_primary_count as f64 / self.total_values as f64) * 100.0;
                    self.warnings.push(format!(
                        "Колонка '{}' содержит неоднородные данные: {:.1}% значений не соответствуют основному типу {:?}",
                        self.name, percentage, self.inferred_type
                    ));
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<Value>>,
    pub column_names: Vec<String>,
}

impl Table {
    pub fn new(column_names: Vec<String>) -> Self {
        let columns = column_names.iter()
            .map(|name| TableColumn::new(name.clone()))
            .collect();

        Self {
            columns,
            rows: Vec::new(),
            column_names,
        }
    }

    pub fn add_row(&mut self, row: Vec<Value>) -> Result<(), String> {
        if row.len() != self.column_names.len() {
            return Err(format!(
                "Количество значений в строке ({}) не соответствует количеству колонок ({})",
                row.len(), self.column_names.len()
            ));
        }

        // Обновляем типизацию колонок
        for (i, value) in row.iter().enumerate() {
            if let Some(column) = self.columns.get_mut(i) {
                column.add_value(value);
            }
        }

        self.rows.push(row);
        Ok(())
    }

    pub fn get_warnings(&self) -> Vec<String> {
        self.columns.iter()
            .flat_map(|col| col.warnings.iter())
            .cloned()
            .collect()
    }
}

// Вспомогательная функция для определения даты
fn is_date_string(s: &str) -> bool {
    // Простая проверка на формат даты (можно расширить)
    use chrono::{DateTime, NaiveDate};

    // Проверяем различные форматы дат
    if DateTime::parse_from_rfc3339(s).is_ok() {
        return true;
    }

    if NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
        return true;
    }

    if NaiveDate::parse_from_str(s, "%d.%m.%Y").is_ok() {
        return true;
    }

    if NaiveDate::parse_from_str(s, "%d/%m/%Y").is_ok() {
        return true;
    }

    false
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Table(Table),
    Null,
    Path(PathBuf),
    PathPattern(PathBuf), // Для glob паттернов типа /path/*.csv
}

impl Value {
    // Пример операции сложения для Path + String
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
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
            (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            (Number(a), Number(b)) => Ok(Number(a + b)),
            _ => Err(format!("Unsupported add operation between {:?} and {:?}", self, other)),
        }
    }
}