// Оптимизированный реестр встроенных функций для DataCode
// Критическая реализация с хэш-таблицей для O(1) доступа

use std::collections::HashMap;
use crate::value::Value;
use crate::error::{Result, DataCodeError};



/// Обертка для функции read_file
fn read_file_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::file::call_file_function("read_file", args, line)
}

/// Обертка для функции len
fn len_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::array::call_array_function("len", args, line)
}

/// Обертка для функции table_headers
fn table_headers_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::table::call_table_function("table_headers", args, line)
}

/// Обертка для функции table_filter
fn table_filter_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::table::call_table_function("table_filter", args, line)
}

/// Обертка для функции table_where
fn table_where_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::table::call_table_function("table_where", args, line)
}

/// Обертка для функции table_select
fn table_select_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::table::call_table_function("table_select", args, line)
}

/// Обертка для функции table_head
fn table_head_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::table::call_table_function("table_head", args, line)
}

/// Обертка для функции sum
fn sum_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::array::call_array_function("sum", args, line)
}

/// Обертка для функции avg
fn avg_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    crate::builtins::array::call_array_function("average", args, line)
}

/// Обертка для функции write_file
fn write_file_wrapper(args: Vec<Value>, line: usize) -> Result<Value> {
    if args.len() != 2 {
        return Err(DataCodeError::wrong_argument_count("write_file", 2, args.len(), line));
    }

    let path = match &args[0] {
        Value::Path(p) => p.clone(),
        Value::String(s) => std::path::PathBuf::from(s),
        _ => return Err(DataCodeError::type_error("Path or String", "other", line)),
    };

    let content = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err(DataCodeError::type_error("String", "other", line)),
    };

    std::fs::write(&path, content)
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to write file: {}", e), line))?;

    Ok(Value::Null)
}

/// Тип функции в реестре
pub type BuiltinFunction = fn(Vec<Value>, usize) -> Result<Value>;

/// Информация о встроенной функции
#[derive(Clone)]
pub struct FunctionInfo {
    pub name: &'static str,
    pub function: BuiltinFunction,
    pub min_args: usize,
    pub max_args: Option<usize>,
    pub _description: &'static str,
    pub category: &'static str,
}

impl FunctionInfo {
    /// Создать новую информацию о функции
    pub fn new(
        name: &'static str,
        function: BuiltinFunction,
        min_args: usize,
        max_args: Option<usize>,
        description: &'static str,
        category: &'static str,
    ) -> Self {
        Self {
            name,
            function,
            min_args,
            max_args,
            _description: description,
            category,
        }
    }
    
    /// Проверить количество аргументов
    pub fn validate_args(&self, arg_count: usize) -> bool {
        if arg_count < self.min_args {
            return false;
        }
        
        if let Some(max) = self.max_args {
            if arg_count > max {
                return false;
            }
        }
        
        true
    }
}

/// Оптимизированный реестр функций с хэш-таблицей
pub struct FunctionRegistry {
    functions: HashMap<&'static str, FunctionInfo>,
    categories: HashMap<&'static str, Vec<&'static str>>,
}

impl FunctionRegistry {
    /// Создать новый реестр
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
            categories: HashMap::new(),
        };
        
        registry.register_all_functions();
        registry
    }
    
    /// Зарегистрировать функцию
    #[allow(dead_code)]
    pub fn register(&mut self, info: FunctionInfo) {
        // Добавляем в категорию
        self.categories
            .entry(info.category)
            .or_insert_with(Vec::new)
            .push(info.name);
        
        // Добавляем в основной реестр
        self.functions.insert(info.name, info);
    }
    
    /// Получить функцию по имени (O(1) доступ)
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }
    
    /// Вызвать функцию
    pub fn call_function(&self, name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
        if let Some(info) = self.get_function(name) {
            if !info.validate_args(args.len()) {
                return Err(crate::error::DataCodeError::runtime_error(
                    &format!(
                        "Function '{}' expects {} arguments, got {}",
                        name,
                        if let Some(max) = info.max_args {
                            format!("{}-{}", info.min_args, max)
                        } else {
                            format!("at least {}", info.min_args)
                        },
                        args.len()
                    ),
                    line
                ));
            }
            
            (info.function)(args, line)
        } else {
            Err(crate::error::DataCodeError::runtime_error(
                &format!("Unknown function: {}", name),
                line
            ))
        }
    }
    
    /// Получить все функции в категории
    #[allow(dead_code)]
    pub fn get_functions_by_category(&self, category: &str) -> Vec<&FunctionInfo> {
        if let Some(function_names) = self.categories.get(category) {
            function_names
                .iter()
                .filter_map(|name| self.functions.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Получить все категории
    #[allow(dead_code)]
    pub fn get_categories(&self) -> Vec<&str> {
        self.categories.keys().copied().collect()
    }
    
    /// Получить все функции
    #[allow(dead_code)]
    pub fn get_all_functions(&self) -> &HashMap<&'static str, FunctionInfo> {
        &self.functions
    }
    
    /// Проверить существование функции
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// Зарегистрировать все встроенные функции
    fn register_all_functions(&mut self) {
        // Функции фильтрации таблиц
        self.register(FunctionInfo::new(
            "table_filter",
            table_filter_wrapper,
            2, Some(2),
            "Filter table rows based on condition",
            "table"
        ));

        self.register(FunctionInfo::new(
            "table_where",
            table_where_wrapper,
            4, Some(4),
            "Filter table rows where column matches condition",
            "table"
        ));

        self.register(FunctionInfo::new(
            "table_select",
            table_select_wrapper,
            2, None,
            "Select specific columns from table",
            "table"
        ));

        self.register(FunctionInfo::new(
            "sum",
            sum_wrapper,
            1, None,
            "Calculate sum of numbers",
            "math"
        ));

        self.register(FunctionInfo::new(
            "avg",
            avg_wrapper,
            1, None,
            "Calculate average of numbers",
            "math"
        ));

        self.register(FunctionInfo::new(
            "len",
            len_wrapper,
            1, Some(1),
            "Get length of string or array",
            "string"
        ));

        self.register(FunctionInfo::new(
            "read_file",
            read_file_wrapper,
            1, Some(3),
            "Read file contents. Supports: read_file(path), read_file(path, sheet_name), read_file(path, header_row), read_file(path, header_row, sheet_name)",
            "file"
        ));

        self.register(FunctionInfo::new(
            "write_file",
            write_file_wrapper,
            2, Some(2),
            "Write content to file",
            "file"
        ));

        self.register(FunctionInfo::new(
            "table_headers",
            table_headers_wrapper,
            1, Some(1),
            "Get table column headers",
            "table"
        ));

        self.register(FunctionInfo::new(
            "table_head",
            table_head_wrapper,
            1, Some(2),
            "Get first n rows of table",
            "table"
        ));
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_FUNCTION_REGISTRY: FunctionRegistry = FunctionRegistry::new();
}

/// Быстрый вызов функции через глобальный реестр
pub fn call_builtin_function_fast(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    GLOBAL_FUNCTION_REGISTRY.call_function(name, args, line)
}

/// Проверить существование функции
pub fn function_exists(name: &str) -> bool {
    GLOBAL_FUNCTION_REGISTRY.has_function(name)
}

/// Получить информацию о функции
#[allow(dead_code)]
pub fn get_function_info(name: &str) -> Option<&FunctionInfo> {
    GLOBAL_FUNCTION_REGISTRY.get_function(name)
}

/// Получить все функции по категории
#[allow(dead_code)]
pub fn get_functions_by_category(category: &str) -> Vec<&FunctionInfo> {
    GLOBAL_FUNCTION_REGISTRY.get_functions_by_category(category)
}

/// Получить список всех категорий
#[allow(dead_code)]
pub fn get_all_categories() -> Vec<String> {
    GLOBAL_FUNCTION_REGISTRY.get_categories().into_iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_registry_basic() {
        let registry = FunctionRegistry::new();
        
        // Проверяем, что функции зарегистрированы
        assert!(registry.has_function("table_filter"));
        assert!(registry.has_function("sum"));
        assert!(registry.has_function("len"));
        assert!(!registry.has_function("nonexistent_function"));
    }
    
    #[test]
    fn test_function_info_validation() {
        let info = FunctionInfo::new(
            "test_func",
            |_args, _line| Ok(Value::Null),
            2, Some(4),
            "Test function",
            "test"
        );
        
        assert!(!info.validate_args(1)); // Слишком мало
        assert!(info.validate_args(2));  // Минимум
        assert!(info.validate_args(3));  // В пределах
        assert!(info.validate_args(4));  // Максимум
        assert!(!info.validate_args(5)); // Слишком много
    }
    
    #[test]
    fn test_categories() {
        let registry = FunctionRegistry::new();
        let categories = registry.get_categories();
        
        assert!(categories.contains(&"table"));
        assert!(categories.contains(&"math"));
        assert!(categories.contains(&"string"));
        assert!(categories.contains(&"file"));
        
        let table_functions = registry.get_functions_by_category("table");
        assert!(!table_functions.is_empty());
        
        let math_functions = registry.get_functions_by_category("math");
        assert!(!math_functions.is_empty());
    }
    
    #[test]
    fn test_global_registry() {
        assert!(function_exists("table_filter"));
        assert!(function_exists("sum"));
        assert!(!function_exists("nonexistent"));
        
        let info = get_function_info("sum");
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "sum");
        assert_eq!(info.unwrap().category, "math");
    }
}
