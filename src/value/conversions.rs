// Логика преобразования и распознавания типов данных в DataCode
// Включает функции для определения дат, валют и других специальных типов

use std::sync::OnceLock;

// Ленивая инициализация регулярных выражений для производительности
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

/// Проверить, является ли строка датой
/// Поддерживает различные форматы дат: ISO, европейский, американский
pub fn is_date_string(s: &str) -> bool {
    use chrono::{DateTime, NaiveDate};

    // Быстрая предварительная проверка - должна содержать цифры и разделители
    if s.len() < 6 || s.len() > 35 {  // Увеличиваем лимит для RFC3339 с таймзонами
        return false;
    }

    // Быстрая проверка на наличие разделителей дат
    if !s.contains('/') && !s.contains('-') && !s.contains('.') && !s.contains('T') {
        return false;
    }

    // Проверяем RFC3339 (самый быстрый, так как точный формат)
    if s.contains('T') {
        if let Ok(_) = DateTime::parse_from_rfc3339(s) {
            return true;
        }
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

/// Проверить, является ли строка денежным значением
/// Поддерживает валютные символы и коды валют
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

/// Попытаться преобразовать строку в число
pub fn try_parse_number(s: &str) -> Option<f64> {
    s.trim().parse().ok()
}

/// Попытаться преобразовать строку в булево значение
pub fn try_parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Some(true),
        "false" | "no" | "0" | "off" => Some(false),
        _ => None,
    }
}

/// Нормализовать строку валюты (удалить лишние пробелы, привести к стандартному формату)
pub fn normalize_currency_string(s: &str) -> String {
    s.trim().to_string()
}

/// Получить список поддерживаемых валютных символов
pub fn get_currency_symbols() -> &'static [char] {
    CURRENCY_SYMBOLS
}

/// Получить список поддерживаемых кодов валют
pub fn get_currency_codes() -> &'static [&'static str] {
    CURRENCY_CODES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_date_string_iso() {
        assert!(is_date_string("2023-12-25"));
        assert!(is_date_string("2023-1-1"));
        assert!(is_date_string("2023-01-01"));
        assert!(!is_date_string("2023-13-01")); // Неверный месяц
        assert!(!is_date_string("2023-01-32")); // Неверный день
    }

    #[test]
    fn test_is_date_string_slash_formats() {
        assert!(is_date_string("12/25/2023"));
        assert!(is_date_string("25/12/2023"));
        assert!(is_date_string("1/1/23"));
        assert!(is_date_string("12/9/2019")); // Пример из требований
        assert!(!is_date_string("13/25/2023")); // Неверные значения
    }

    #[test]
    fn test_is_date_string_dot_formats() {
        assert!(is_date_string("25.12.2023"));
        assert!(is_date_string("01.01.23"));
        assert!(!is_date_string("32.12.2023")); // Неверный день
    }

    #[test]
    fn test_is_date_string_rfc3339() {
        assert!(is_date_string("2023-12-25T10:30:00Z"));
        assert!(is_date_string("2023-12-25T10:30:00+03:00"));
        assert!(!is_date_string("2023-12-25T25:30:00Z")); // Неверный час
    }

    #[test]
    fn test_is_date_string_invalid() {
        assert!(!is_date_string("not a date"));
        assert!(!is_date_string("123456"));
        assert!(!is_date_string(""));
        assert!(!is_date_string("2023"));
        assert!(!is_date_string("hello/world/test"));
    }

    #[test]
    fn test_is_currency_string_symbols() {
        assert!(is_currency_string("$100"));
        assert!(is_currency_string("€50.99"));
        assert!(is_currency_string("₽1000"));
        assert!(is_currency_string("£25.50"));
        assert!(is_currency_string("100$"));
        assert!(is_currency_string("50.99 €"));
    }

    #[test]
    fn test_is_currency_string_codes() {
        assert!(is_currency_string("100 USD"));
        assert!(is_currency_string("EUR 50.99"));
        assert!(is_currency_string("1000 RUB"));
        assert!(is_currency_string("GBP 25.50"));
    }

    #[test]
    fn test_is_currency_string_invalid() {
        assert!(!is_currency_string("100")); // Только число
        assert!(!is_currency_string("hello")); // Только текст
        assert!(!is_currency_string("$")); // Только символ без числа
        assert!(!is_currency_string("USD")); // Только код без числа
        assert!(!is_currency_string("")); // Пустая строка
    }

    #[test]
    fn test_try_parse_number() {
        assert_eq!(try_parse_number("42"), Some(42.0));
        assert_eq!(try_parse_number("3.14"), Some(3.14));
        assert_eq!(try_parse_number("-10"), Some(-10.0));
        assert_eq!(try_parse_number("  42  "), Some(42.0));
        assert_eq!(try_parse_number("not a number"), None);
        assert_eq!(try_parse_number(""), None);
    }

    #[test]
    fn test_try_parse_bool() {
        assert_eq!(try_parse_bool("true"), Some(true));
        assert_eq!(try_parse_bool("false"), Some(false));
        assert_eq!(try_parse_bool("TRUE"), Some(true));
        assert_eq!(try_parse_bool("FALSE"), Some(false));
        assert_eq!(try_parse_bool("yes"), Some(true));
        assert_eq!(try_parse_bool("no"), Some(false));
        assert_eq!(try_parse_bool("1"), Some(true));
        assert_eq!(try_parse_bool("0"), Some(false));
        assert_eq!(try_parse_bool("on"), Some(true));
        assert_eq!(try_parse_bool("off"), Some(false));
        assert_eq!(try_parse_bool("maybe"), None);
        assert_eq!(try_parse_bool(""), None);
    }

    #[test]
    fn test_normalize_currency_string() {
        assert_eq!(normalize_currency_string("  $100  "), "$100");
        assert_eq!(normalize_currency_string("€50.99"), "€50.99");
        assert_eq!(normalize_currency_string(""), "");
    }

    #[test]
    fn test_get_currency_symbols() {
        let symbols = get_currency_symbols();
        assert!(symbols.contains(&'$'));
        assert!(symbols.contains(&'€'));
        assert!(symbols.contains(&'₽'));
        assert!(symbols.contains(&'£'));
    }

    #[test]
    fn test_get_currency_codes() {
        let codes = get_currency_codes();
        assert!(codes.contains(&"USD"));
        assert!(codes.contains(&"EUR"));
        assert!(codes.contains(&"RUB"));
        assert!(codes.contains(&"GBP"));
    }
}
