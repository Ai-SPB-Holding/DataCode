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

// Ленивая инициализация регулярных выражений для производительности
use std::sync::OnceLock;

static DATE_REGEX_FLEXIBLE_4Y: OnceLock<regex::Regex> = OnceLock::new();
static DATE_REGEX_FLEXIBLE_2Y: OnceLock<regex::Regex> = OnceLock::new();
static DATE_REGEX_ISO_FLEXIBLE: OnceLock<regex::Regex> = OnceLock::new();

fn get_date_regex_flexible_4y() -> &'static regex::Regex {
    DATE_REGEX_FLEXIBLE_4Y.get_or_init(|| {
        regex::Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{4})$").unwrap()
    })
}

fn get_date_regex_flexible_2y() -> &'static regex::Regex {
    DATE_REGEX_FLEXIBLE_2Y.get_or_init(|| {
        regex::Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{2})$").unwrap()
    })
}

fn get_date_regex_iso_flexible() -> &'static regex::Regex {
    DATE_REGEX_ISO_FLEXIBLE.get_or_init(|| {
        regex::Regex::new(r"^(\d{4})-(\d{1,2})-(\d{1,2})$").unwrap()
    })
}

// Оптимизированная функция для определения даты
fn is_date_string(s: &str) -> bool {
    use chrono::{DateTime, NaiveDate};

    // Быстрая предварительная проверка - должна содержать цифры и разделители
    if s.len() < 6 || s.len() > 19 {
        return false;
    }

    // Быстрая проверка на наличие разделителей дат
    if !s.contains('/') && !s.contains('-') && !s.contains('.') && !s.contains('T') {
        return false;
    }

    // Проверяем RFC3339 (самый быстрый, так как точный формат)
    if s.contains('T') && DateTime::parse_from_rfc3339(s).is_ok() {
        return true;
    }

    // Проверяем ISO формат YYYY-MM-DD (точный формат)
    if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        if NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
            return true;
        }
    }

    // Проверяем европейские форматы с точками (точные форматы)
    if s.contains('.') {
        if s.len() == 10 && NaiveDate::parse_from_str(s, "%d.%m.%Y").is_ok() {
            return true;
        }
        if s.len() == 8 && NaiveDate::parse_from_str(s, "%d.%m.%y").is_ok() {
            return true;
        }
    }

    // Проверяем форматы с слешами (точные форматы сначала)
    if s.contains('/') {
        // Точные форматы с ведущими нулями
        if s.len() == 10 {
            if NaiveDate::parse_from_str(s, "%d/%m/%Y").is_ok() {
                return true;
            }
            if NaiveDate::parse_from_str(s, "%m/%d/%Y").is_ok() {
                return true;
            }
        }
        if s.len() == 8 {
            if NaiveDate::parse_from_str(s, "%d/%m/%y").is_ok() {
                return true;
            }
            if NaiveDate::parse_from_str(s, "%m/%d/%y").is_ok() {
                return true;
            }
        }

        // Гибкие форматы без ведущих нулей (более дорогие операции)
        if let Some(captures) = get_date_regex_flexible_4y().captures(s) {
            if let (Ok(first), Ok(second), Ok(year)) = (
                captures[1].parse::<u32>(),
                captures[2].parse::<u32>(),
                captures[3].parse::<i32>()
            ) {
                // Быстрая проверка диапазонов
                if first <= 31 && second <= 31 && year >= 1900 && year <= 2100 {
                    // Американский формат
                    if first <= 12 && second <= 31 {
                        if NaiveDate::from_ymd_opt(year, first, second).is_some() {
                            return true;
                        }
                    }
                    // Европейский формат (только если первое число > 12)
                    if second <= 12 && first <= 31 && first > 12 {
                        if NaiveDate::from_ymd_opt(year, second, first).is_some() {
                            return true;
                        }
                    }
                }
            }
        }

        // Двузначный год
        if let Some(captures) = get_date_regex_flexible_2y().captures(s) {
            if let (Ok(first), Ok(second), Ok(year_short)) = (
                captures[1].parse::<u32>(),
                captures[2].parse::<u32>(),
                captures[3].parse::<i32>()
            ) {
                if first <= 31 && second <= 31 && year_short <= 99 {
                    let year = if year_short <= 30 { 2000 + year_short } else { 1900 + year_short };

                    if first <= 12 && second <= 31 {
                        if NaiveDate::from_ymd_opt(year, first, second).is_some() {
                            return true;
                        }
                    }
                    if second <= 12 && first <= 31 && first > 12 {
                        if NaiveDate::from_ymd_opt(year, second, first).is_some() {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // ISO формат без ведущих нулей
    if s.contains('-') && !s.contains('T') {
        if let Some(captures) = get_date_regex_iso_flexible().captures(s) {
            if let (Ok(year), Ok(month), Ok(day)) = (
                captures[1].parse::<i32>(),
                captures[2].parse::<u32>(),
                captures[3].parse::<u32>()
            ) {
                if year >= 1900 && year <= 2100 && month <= 12 && day <= 31 {
                    if NaiveDate::from_ymd_opt(year, month, day).is_some() {
                        return true;
                    }
                }
            }
        }
    }

    false
}

// Статические данные для валют (инициализируются один раз)
static CURRENCY_SYMBOLS: &[char] = &['$', '€', '₽', '£', '¥', '₹', '₩', '₪', '₦', '₡', '₨', '₫', '₱', '₲', '₴', '₵', '₸', '₼'];
static CURRENCY_CODES: &[&str] = &["USD", "EUR", "RUB", "GBP", "JPY", "CNY", "INR", "KRW", "CAD", "AUD", "CHF", "SEK", "NOK", "DKK", "PLN", "CZK", "HUF", "RON", "BGN", "HRK", "RSD", "BAM", "MKD", "ALL", "TRY", "UAH", "BYN", "MDL", "GEL", "AMD", "AZN", "KZT", "KGS", "TJS", "TMT", "UZS"];

// Оптимизированная функция для определения денежных значений
pub fn is_currency_string(s: &str) -> bool {
    let trimmed = s.trim();

    // Быстрая предварительная проверка
    if trimmed.is_empty() || trimmed.len() > 50 {
        return false;
    }

    // Быстрая проверка на наличие валютных символов
    let has_currency_symbol = trimmed.chars().any(|c| CURRENCY_SYMBOLS.contains(&c));

    let has_currency_code = if !has_currency_symbol {
        // Проверяем коды валют только если нет символов
        let upper_trimmed = trimmed.to_uppercase();
        CURRENCY_CODES.iter().any(|&code| {
            upper_trimmed.ends_with(code) || upper_trimmed.starts_with(code)
        })
    } else {
        false
    };

    if !has_currency_symbol && !has_currency_code {
        return false;
    }

    // Быстрая проверка числовой части без создания новых строк
    let mut has_digits = false;
    let mut has_valid_chars_only = true;

    for c in trimmed.chars() {
        match c {
            '0'..='9' => has_digits = true,
            '-' | '+' | '.' | ',' | ' ' => {}, // Допустимые символы
            c if CURRENCY_SYMBOLS.contains(&c) => {}, // Валютные символы
            'A'..='Z' | 'a'..='z' => {
                // Проверяем, что это часть валютного кода
                let is_part_of_currency_code = CURRENCY_CODES.iter().any(|&code| {
                    trimmed.to_uppercase().contains(code)
                });
                if !is_part_of_currency_code {
                    has_valid_chars_only = false;
                    break;
                }
            },
            _ => {
                has_valid_chars_only = false;
                break;
            }
        }
    }

    has_digits && has_valid_chars_only
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