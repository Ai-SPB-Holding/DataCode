// Логика вызова функций в DataCode
// Обрабатывает встроенные функции и пользовательские функции

use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::parser::Expr;
use crate::builtins::call_builtin_function;
use super::Evaluator;

/// Обработчик вызовов функций
pub struct FunctionCallHandler<'a> {
    evaluator: &'a Evaluator<'a>,
}

impl<'a> FunctionCallHandler<'a> {
    /// Создать новый обработчик вызовов функций
    pub fn new(evaluator: &'a Evaluator<'a>) -> Self {
        Self { evaluator }
    }
    
    /// Вычислить вызов функции
    pub fn evaluate(&self, name: &str, args: &[Expr]) -> Result<Value> {
        // Вычисляем аргументы
        let mut arg_values = Vec::new();
        for arg in args {
            let arg_value = self.evaluator.evaluate(arg)?;
            arg_values.push(arg_value);
        }
        
        // Вызываем функцию
        self.call_function(name, arg_values)
    }
    
    /// Вызвать функцию с готовыми значениями аргументов
    pub fn call_function(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Сначала проверяем встроенные функции
        if self.is_builtin_function(name) {
            return call_builtin_function(name, args, self.evaluator.line());
        }
        
        // Затем проверяем пользовательские функции
        // TODO: Добавить поддержку пользовательских функций
        // if let Some(user_function) = self.get_user_function(name) {
        //     return self.call_user_function(user_function, args);
        // }
        
        // Функция не найдена
        Err(DataCodeError::function_not_found(name, self.evaluator.line()))
    }
    
    /// Проверить, является ли функция встроенной
    fn is_builtin_function(&self, name: &str) -> bool {
        crate::builtins::is_builtin_function(name)
    }
    
    /// Валидировать количество аргументов
    pub fn validate_arg_count(&self, function_name: &str, expected: usize, actual: usize) -> Result<()> {
        if actual != expected {
            Err(DataCodeError::runtime_error(
                &format!("{}: Expected {} arguments, got {}", function_name, expected, actual),
                self.evaluator.line()
            ))
        } else {
            Ok(())
        }
    }
    
    /// Валидировать минимальное количество аргументов
    pub fn validate_min_arg_count(&self, function_name: &str, min_expected: usize, actual: usize) -> Result<()> {
        if actual < min_expected {
            Err(DataCodeError::runtime_error(
                &format!("{}: Expected at least {} arguments, got {}", function_name, min_expected, actual),
                self.evaluator.line()
            ))
        } else {
            Ok(())
        }
    }
    
    /// Валидировать диапазон количества аргументов
    pub fn validate_arg_count_range(&self, function_name: &str, min: usize, max: usize, actual: usize) -> Result<()> {
        if actual < min || actual > max {
            Err(DataCodeError::runtime_error(
                &format!("{}: Expected {}-{} arguments, got {}", function_name, min, max, actual),
                self.evaluator.line()
            ))
        } else {
            Ok(())
        }
    }
    
    /// Валидировать тип аргумента
    pub fn validate_arg_type(&self, function_name: &str, arg_index: usize, expected_type: &str, actual: &Value) -> Result<()> {
        let actual_type = match actual {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Bool(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Table(_) => "table",
            Value::Currency(_) => "currency",
            Value::Path(_) => "path",
            Value::PathPattern(_) => "path_pattern",
            Value::Null => "null",
        };
        
        if actual_type != expected_type {
            Err(DataCodeError::runtime_error(
                &format!("{}: Argument {} expected {}, got {}", function_name, arg_index + 1, expected_type, actual_type),
                self.evaluator.line()
            ))
        } else {
            Ok(())
        }
    }
    
    /// Получить аргумент как число
    pub fn get_number_arg(&self, function_name: &str, args: &[Value], index: usize) -> Result<f64> {
        if index >= args.len() {
            return Err(DataCodeError::runtime_error(
                &format!("{}: Missing argument {}", function_name, index + 1),
                self.evaluator.line()
            ));
        }
        
        match &args[index] {
            Value::Number(n) => Ok(*n),
            _ => Err(DataCodeError::runtime_error(
                &format!("{}: Argument {} must be a number", function_name, index + 1),
                self.evaluator.line()
            ))
        }
    }
    
    /// Получить аргумент как строку
    pub fn get_string_arg(&self, function_name: &str, args: &[Value], index: usize) -> Result<String> {
        if index >= args.len() {
            return Err(DataCodeError::runtime_error(
                &format!("{}: Missing argument {}", function_name, index + 1),
                self.evaluator.line()
            ));
        }
        
        match &args[index] {
            Value::String(s) => Ok(s.clone()),
            _ => Err(DataCodeError::runtime_error(
                &format!("{}: Argument {} must be a string", function_name, index + 1),
                self.evaluator.line()
            ))
        }
    }
    
    /// Получить аргумент как массив
    pub fn get_array_arg(&self, function_name: &str, args: &[Value], index: usize) -> Result<Vec<Value>> {
        if index >= args.len() {
            return Err(DataCodeError::runtime_error(
                &format!("{}: Missing argument {}", function_name, index + 1),
                self.evaluator.line()
            ));
        }
        
        match &args[index] {
            Value::Array(arr) => Ok(arr.clone()),
            _ => Err(DataCodeError::runtime_error(
                &format!("{}: Argument {} must be an array", function_name, index + 1),
                self.evaluator.line()
            ))
        }
    }
    
    /// Получить аргумент как булево значение
    pub fn get_bool_arg(&self, function_name: &str, args: &[Value], index: usize) -> Result<bool> {
        if index >= args.len() {
            return Err(DataCodeError::runtime_error(
                &format!("{}: Missing argument {}", function_name, index + 1),
                self.evaluator.line()
            ));
        }
        
        match &args[index] {
            Value::Bool(b) => Ok(*b),
            _ => Err(DataCodeError::runtime_error(
                &format!("{}: Argument {} must be a boolean", function_name, index + 1),
                self.evaluator.line()
            ))
        }
    }
}

/// Трейт для функций
pub trait FunctionCallable {
    /// Имя функции
    fn name(&self) -> &str;
    
    /// Минимальное количество аргументов
    fn min_args(&self) -> usize;
    
    /// Максимальное количество аргументов (None = неограничено)
    fn max_args(&self) -> Option<usize>;
    
    /// Описание функции
    fn description(&self) -> &str;
    
    /// Вызвать функцию
    fn call(&self, args: Vec<Value>, line: usize) -> Result<Value>;
}

/// Информация о функции
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub min_args: usize,
    pub max_args: Option<usize>,
    pub description: String,
    pub category: String,
}

impl FunctionInfo {
    /// Создать новую информацию о функции
    pub fn new(name: String, min_args: usize, max_args: Option<usize>, description: String, category: String) -> Self {
        Self {
            name,
            min_args,
            max_args,
            description,
            category,
        }
    }
    
    /// Проверить, подходит ли количество аргументов
    pub fn accepts_arg_count(&self, count: usize) -> bool {
        count >= self.min_args && self.max_args.map_or(true, |max| count <= max)
    }
}

/// Реестр функций
pub struct FunctionRegistry {
    functions: std::collections::HashMap<String, FunctionInfo>,
}

impl FunctionRegistry {
    /// Создать новый реестр функций
    pub fn new() -> Self {
        Self {
            functions: std::collections::HashMap::new(),
        }
    }
    
    /// Зарегистрировать функцию
    pub fn register(&mut self, info: FunctionInfo) {
        self.functions.insert(info.name.clone(), info);
    }
    
    /// Получить информацию о функции
    pub fn get_function_info(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }
    
    /// Получить все функции
    pub fn get_all_functions(&self) -> &std::collections::HashMap<String, FunctionInfo> {
        &self.functions
    }
    
    /// Получить функции по категории
    pub fn get_functions_by_category(&self, category: &str) -> Vec<&FunctionInfo> {
        self.functions.values()
            .filter(|info| info.category == category)
            .collect()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
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
    fn test_function_call_handler_creation() {
        let evaluator = create_test_evaluator();
        let handler = FunctionCallHandler::new(&evaluator);
        
        // Проверяем, что обработчик создается корректно
        assert!(handler.is_builtin_function("print"));
        assert!(!handler.is_builtin_function("unknown_function"));
    }

    #[test]
    fn test_validate_arg_count() {
        let evaluator = create_test_evaluator();
        let handler = FunctionCallHandler::new(&evaluator);
        
        assert!(handler.validate_arg_count("test", 2, 2).is_ok());
        assert!(handler.validate_arg_count("test", 2, 3).is_err());
        assert!(handler.validate_min_arg_count("test", 2, 3).is_ok());
        assert!(handler.validate_min_arg_count("test", 2, 1).is_err());
    }

    #[test]
    fn test_get_typed_args() {
        let evaluator = create_test_evaluator();
        let handler = FunctionCallHandler::new(&evaluator);
        
        let args = vec![
            Value::Number(42.0),
            Value::String("hello".to_string()),
            Value::Bool(true),
            Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]),
        ];
        
        assert_eq!(handler.get_number_arg("test", &args, 0).unwrap(), 42.0);
        assert_eq!(handler.get_string_arg("test", &args, 1).unwrap(), "hello");
        assert_eq!(handler.get_bool_arg("test", &args, 2).unwrap(), true);
        assert_eq!(handler.get_array_arg("test", &args, 3).unwrap().len(), 2);
        
        // Тестируем ошибки типов
        assert!(handler.get_string_arg("test", &args, 0).is_err());
        assert!(handler.get_number_arg("test", &args, 1).is_err());
    }

    #[test]
    fn test_function_info() {
        let info = FunctionInfo::new(
            "test_func".to_string(),
            1,
            Some(3),
            "Test function".to_string(),
            "test".to_string(),
        );
        
        assert!(info.accepts_arg_count(1));
        assert!(info.accepts_arg_count(2));
        assert!(info.accepts_arg_count(3));
        assert!(!info.accepts_arg_count(0));
        assert!(!info.accepts_arg_count(4));
    }

    #[test]
    fn test_function_registry() {
        let mut registry = FunctionRegistry::new();
        
        let info = FunctionInfo::new(
            "test_func".to_string(),
            1,
            Some(2),
            "Test function".to_string(),
            "test".to_string(),
        );
        
        registry.register(info);
        
        assert!(registry.get_function_info("test_func").is_some());
        assert!(registry.get_function_info("unknown").is_none());
        
        let test_functions = registry.get_functions_by_category("test");
        assert_eq!(test_functions.len(), 1);
    }
}
