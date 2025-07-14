use data_code::value::{Value, DataType};
use data_code::interpreter::Interpreter;

#[cfg(test)]
mod date_tests {
    use super::*;

    #[test]
    fn test_date_formats_with_leading_zeros() {
        let test_cases = vec![
            "2023-12-25",     // ISO format
            "25.12.2023",     // European format
            "25/12/2023",     // European slash format
            "12/25/2023",     // American format
        ];

        for date_str in test_cases {
            let value = Value::String(date_str.to_string());
            let data_type = DataType::from_value(&value);
            assert_eq!(data_type, DataType::Date, "Failed for date: {}", date_str);
        }
    }

    #[test]
    fn test_date_formats_without_leading_zeros() {
        let test_cases = vec![
            "12/9/2019",      // American format without leading zeros
            "9/12/2019",      // European format without leading zeros
            "1/1/2020",       // Single digits
            "31/12/99",       // Two-digit year
            "1/1/00",         // Two-digit year, century boundary
        ];

        for date_str in test_cases {
            let value = Value::String(date_str.to_string());
            let data_type = DataType::from_value(&value);
            assert_eq!(data_type, DataType::Date, "Failed for date: {}", date_str);
        }
    }

    #[test]
    fn test_iso_dates_without_leading_zeros() {
        let test_cases = vec![
            "2023-1-1",       // ISO without leading zeros
            "2023-12-1",      // ISO partial leading zeros
        ];

        for date_str in test_cases {
            let value = Value::String(date_str.to_string());
            let data_type = DataType::from_value(&value);
            assert_eq!(data_type, DataType::Date, "Failed for ISO date: {}", date_str);
        }
    }

    #[test]
    fn test_invalid_dates() {
        let test_cases = vec![
            "not a date",
            "32/12/2023",     // Invalid day
            "15/13/2023",     // Invalid month (both interpretations fail)
            "abc/def/ghi",    // Non-numeric
            "2023-13-01",     // Invalid month in ISO
            "40/40/2023",     // Both day and month invalid
        ];

        for date_str in test_cases {
            let value = Value::String(date_str.to_string());
            let data_type = DataType::from_value(&value);
            assert_ne!(data_type, DataType::Date, "Should not be date: {}", date_str);
        }
    }
}

#[cfg(test)]
mod currency_tests {
    use super::*;

    #[test]
    fn test_currency_symbols() {
        let test_cases = vec![
            "$100",           // Dollar
            "€50",            // Euro
            "₽1000",          // Ruble
            "£25",            // Pound
            "¥500",           // Yen
            "100$",           // Dollar at end
            "$1,000.50",      // With comma separator
            "-$50",           // Negative amount
        ];

        for currency_str in test_cases {
            let value = Value::String(currency_str.to_string());
            let data_type = DataType::from_value(&value);
            assert_eq!(data_type, DataType::Currency, "Failed for currency: {}", currency_str);
        }
    }

    #[test]
    fn test_currency_codes() {
        let test_cases = vec![
            "100 USD",        // With space
            "50EUR",          // Without space
            "1000 RUB",       // Ruble code
            "25 GBP",         // Pound code
            "USD 100",        // Code at start
            "500 JPY",        // Yen code
        ];

        for currency_str in test_cases {
            let value = Value::String(currency_str.to_string());
            let data_type = DataType::from_value(&value);
            assert_eq!(data_type, DataType::Currency, "Failed for currency: {}", currency_str);
        }
    }

    #[test]
    fn test_invalid_currency() {
        let test_cases = vec![
            "just text",      // No currency indicator
            "100",            // Just number
            "abc USD",        // Non-numeric with currency code
            "$",              // Just symbol
            "USD",            // Just code
        ];

        for currency_str in test_cases {
            let value = Value::String(currency_str.to_string());
            let data_type = DataType::from_value(&value);
            assert_ne!(data_type, DataType::Currency, "Should not be currency: {}", currency_str);
        }
    }

    #[test]
    fn test_currency_value_creation() {
        let currency_value = Value::Currency("$100.50".to_string());
        let data_type = DataType::from_value(&currency_value);
        assert_eq!(data_type, DataType::Currency);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_csv_parsing_with_dates_and_currency() {
        let mut interp = Interpreter::new();
        
        // Test that the interpreter can handle mixed data types
        let result = interp.exec("global test_date = '12/9/2019'");
        assert!(result.is_ok());
        
        let result = interp.exec("global test_currency = '$1,500.00'");
        assert!(result.is_ok());
        
        let result = interp.exec("global test_string = 'regular text'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_priority() {
        // Currency should be detected before regular string
        let currency_value = Value::String("$100".to_string());
        let data_type = DataType::from_value(&currency_value);
        assert_eq!(data_type, DataType::Currency);
        
        // Date should be detected before regular string
        let date_value = Value::String("12/9/2019".to_string());
        let data_type = DataType::from_value(&date_value);
        assert_eq!(data_type, DataType::Date);
        
        // Regular string should remain string
        let string_value = Value::String("just text".to_string());
        let data_type = DataType::from_value(&string_value);
        assert_eq!(data_type, DataType::String);
    }
}
