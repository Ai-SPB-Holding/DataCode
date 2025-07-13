use std::collections::HashMap;
use std::path::PathBuf;

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
    use chrono::{DateTime, NaiveDate};

    // Проверяем различные форматы дат
    if DateTime::parse_from_rfc3339(s).is_ok() {
        return true;
    }

    // ISO формат YYYY-MM-DD
    if NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
        return true;
    }

    // Европейские форматы с точками
    if NaiveDate::parse_from_str(s, "%d.%m.%Y").is_ok() {
        return true;
    }
    if NaiveDate::parse_from_str(s, "%d.%m.%y").is_ok() {
        return true;
    }

    // Форматы с слешами - день/месяц/год
    if NaiveDate::parse_from_str(s, "%d/%m/%Y").is_ok() {
        return true;
    }
    if NaiveDate::parse_from_str(s, "%d/%m/%y").is_ok() {
        return true;
    }

    // Американские форматы - месяц/день/год
    if NaiveDate::parse_from_str(s, "%m/%d/%Y").is_ok() {
        return true;
    }
    if NaiveDate::parse_from_str(s, "%m/%d/%y").is_ok() {
        return true;
    }

    // Форматы без ведущих нулей
    // Пытаемся парсить как M/D/YYYY или D/M/YYYY
    if let Some(captures) = regex::Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{4})$").unwrap().captures(s) {
        if let (Ok(first), Ok(second), Ok(year)) = (
            captures[1].parse::<u32>(),
            captures[2].parse::<u32>(),
            captures[3].parse::<i32>()
        ) {
            // Проверяем, может ли это быть валидной датой в любом из форматов
            // Но только если значения не превышают разумные пределы

            // Сначала пробуем месяц/день/год (американский формат)
            if first <= 12 && second <= 31 {
                if NaiveDate::from_ymd_opt(year, first, second).is_some() {
                    return true;
                }
            }
            // Затем пробуем день/месяц/год (европейский формат)
            // НО только если первое число больше 12 (иначе это может быть американский формат)
            if second <= 12 && first <= 31 && first > 12 {
                if NaiveDate::from_ymd_opt(year, second, first).is_some() {
                    return true;
                }
            }
        }
    }

    // Форматы без ведущих нулей с двузначным годом
    if let Some(captures) = regex::Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{2})$").unwrap().captures(s) {
        if let (Ok(first), Ok(second), Ok(year_short)) = (
            captures[1].parse::<u32>(),
            captures[2].parse::<u32>(),
            captures[3].parse::<i32>()
        ) {
            // Преобразуем двузначный год в четырехзначный (предполагаем 20xx для 00-30, 19xx для 31-99)
            let year = if year_short <= 30 { 2000 + year_short } else { 1900 + year_short };

            // Проверяем оба формата с той же логикой
            if first <= 12 && second <= 31 {
                if NaiveDate::from_ymd_opt(year, first, second).is_some() {
                    return true;
                }
            }
            // Европейский формат только если первое число больше 12
            if second <= 12 && first <= 31 && first > 12 {
                if NaiveDate::from_ymd_opt(year, second, first).is_some() {
                    return true;
                }
            }
        }
    }

    // Форматы с дефисами без ведущих нулей
    if let Some(captures) = regex::Regex::new(r"^(\d{4})-(\d{1,2})-(\d{1,2})$").unwrap().captures(s) {
        if let (Ok(year), Ok(month), Ok(day)) = (
            captures[1].parse::<i32>(),
            captures[2].parse::<u32>(),
            captures[3].parse::<u32>()
        ) {
            if NaiveDate::from_ymd_opt(year, month, day).is_some() {
                return true;
            }
        }
    }

    false
}

// Вспомогательная функция для определения денежных значений
pub fn is_currency_string(s: &str) -> bool {
    let trimmed = s.trim();

    // Список валютных символов
    let currency_symbols = ['$', '€', '₽', '£', '¥', '₹', '₩', '₪', '₦', '₡', '₨', '₫', '₱', '₲', '₴', '₵', '₸', '₼', '₽'];

    // Проверяем, содержит ли строка валютный символ
    let has_currency_symbol = currency_symbols.iter().any(|&symbol| trimmed.contains(symbol));

    if !has_currency_symbol {
        // Проверяем текстовые обозначения валют
        let currency_codes = ["USD", "EUR", "RUB", "GBP", "JPY", "CNY", "INR", "KRW", "CAD", "AUD", "CHF", "SEK", "NOK", "DKK", "PLN", "CZK", "HUF", "RON", "BGN", "HRK", "RSD", "BAM", "MKD", "ALL", "TRY", "UAH", "BYN", "MDL", "GEL", "AMD", "AZN", "KZT", "KGS", "TJS", "TMT", "UZS"];
        let upper_trimmed = trimmed.to_uppercase();

        // Проверяем, заканчивается ли строка кодом валюты
        let has_currency_code = currency_codes.iter().any(|&code| {
            upper_trimmed.ends_with(code) || upper_trimmed.starts_with(code)
        });

        if !has_currency_code {
            return false;
        }
    }

    // Удаляем валютные символы и коды для проверки числовой части
    let mut cleaned = trimmed.to_string();

    // Удаляем валютные символы
    for symbol in currency_symbols.iter() {
        cleaned = cleaned.replace(*symbol, "");
    }

    // Удаляем текстовые коды валют
    let currency_codes = ["USD", "EUR", "RUB", "GBP", "JPY", "CNY", "INR", "KRW", "CAD", "AUD", "CHF", "SEK", "NOK", "DKK", "PLN", "CZK", "HUF", "RON", "BGN", "HRK", "RSD", "BAM", "MKD", "ALL", "TRY", "UAH", "BYN", "MDL", "GEL", "AMD", "AZN", "KZT", "KGS", "TJS", "TMT", "UZS"];
    for code in currency_codes.iter() {
        cleaned = cleaned.replace(code, "");
    }

    // Удаляем пробелы и запятые (разделители тысяч)
    cleaned = cleaned.replace(" ", "").replace(",", "").replace(".", "");

    // Проверяем, остались ли только цифры (возможно с минусом в начале)
    if cleaned.is_empty() {
        return false;
    }

    // Проверяем, что оставшаяся часть - это число
    cleaned.chars().all(|c| c.is_ascii_digit()) ||
    (cleaned.starts_with('-') && cleaned[1..].chars().all(|c| c.is_ascii_digit()))
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Table(Table),
    Currency(String), // Хранит оригинальную строку с валютой
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