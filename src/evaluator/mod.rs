// Модульная структура вычислителя выражений DataCode
// Этот модуль координирует работу всех компонентов вычисления выражений

pub mod expressions;
pub mod operators;
pub mod functions;
pub mod indexing;

// Реэкспорт основных типов для удобства использования
pub use expressions::ExpressionEvaluator;
pub use operators::{BinaryOperatorHandler, UnaryOperatorHandler};
pub use functions::FunctionCallHandler;
pub use indexing::{IndexingHandler, MemberAccessHandler};

use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::parser::Expr;
use std::collections::HashMap;

/// Основной вычислитель выражений DataCode
/// Координирует работу всех компонентов вычисления
pub struct Evaluator<'a> {
    variables: &'a HashMap<String, Value>,
    line: usize,
}

impl<'a> Evaluator<'a> {
    /// Создать новый вычислитель для заданных переменных и номера строки
    pub fn new(variables: &'a HashMap<String, Value>, line: usize) -> Self {
        Self { variables, line }
    }
    
    /// Получить ссылку на переменные
    #[allow(dead_code)]
    pub fn variables(&self) -> &HashMap<String, Value> {
        self.variables
    }
    
    /// Получить номер текущей строки
    pub fn line(&self) -> usize {
        self.line
    }
    
    /// Вычислить выражение
    pub fn evaluate(&self, expr: &Expr) -> Result<Value> {
        let expr_evaluator = ExpressionEvaluator::new(self);
        expr_evaluator.evaluate(expr)
    }
    
    /// Вычислить бинарный оператор
    pub fn evaluate_binary_op(&self, left: &Value, op: &crate::parser::BinaryOp, right: &Value) -> Result<Value> {
        let binary_handler = BinaryOperatorHandler::new(self);
        binary_handler.evaluate(left, op, right)
    }
    
    /// Вычислить унарный оператор
    pub fn evaluate_unary_op(&self, op: &crate::parser::UnaryOp, operand: &Value) -> Result<Value> {
        let unary_handler = UnaryOperatorHandler::new(self);
        unary_handler.evaluate(op, operand)
    }
    
    /// Вычислить вызов функции
    pub fn evaluate_function_call(&self, name: &str, args: &[Expr]) -> Result<Value> {
        let function_handler = FunctionCallHandler::new(self);
        function_handler.evaluate(name, args)
    }
    
    /// Вычислить индексацию
    pub fn evaluate_index(&self, object: &Value, index: &Value) -> Result<Value> {
        let indexing_handler = IndexingHandler::new(self);
        indexing_handler.evaluate(object, index)
    }
    
    /// Вычислить доступ к члену объекта
    pub fn evaluate_member(&self, object: &Value, member: &str) -> Result<Value> {
        let member_handler = MemberAccessHandler::new(self);
        member_handler.evaluate(object, member)
    }
    
    /// Получить значение переменной
    pub fn get_variable(&self, name: &str) -> Result<Value> {
        self.variables.get(name)
            .cloned()
            .ok_or_else(|| DataCodeError::variable_not_found(name, self.line))
    }
    
    /// Проверить равенство двух значений
    pub fn values_equal(&self, left: &Value, right: &Value) -> bool {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
            (String(a), String(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Null, Null) => true,
            _ => false,
        }
    }
    
    /// Преобразовать значение в булево
    pub fn to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Table(table) => !table.borrow().rows.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }
}

/// Вспомогательная функция для быстрого парсинга и вычисления выражений
#[allow(dead_code)]
pub fn parse_and_evaluate(expr_str: &str, variables: &HashMap<String, Value>, line: usize) -> Result<Value> {
    let mut parser = crate::parser::Parser::new(expr_str);
    let expr = parser.parse_expression()?;
    let evaluator = Evaluator::new(variables, line);
    evaluator.evaluate(&expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_evaluator_creation() {
        let variables = HashMap::new();
        let evaluator = Evaluator::new(&variables, 1);
        assert_eq!(evaluator.line(), 1);
        assert!(evaluator.variables().is_empty());
    }

    #[test]
    fn test_get_variable() {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), Value::Number(42.0));
        
        let evaluator = Evaluator::new(&variables, 1);
        let result = evaluator.get_variable("x").unwrap();
        assert_eq!(result, Value::Number(42.0));
        
        let error = evaluator.get_variable("y");
        assert!(error.is_err());
    }

    #[test]
    fn test_values_equal() {
        let variables = HashMap::new();
        let evaluator = Evaluator::new(&variables, 1);
        
        assert!(evaluator.values_equal(&Value::Number(42.0), &Value::Number(42.0)));
        assert!(evaluator.values_equal(&Value::String("hello".to_string()), &Value::String("hello".to_string())));
        assert!(evaluator.values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(evaluator.values_equal(&Value::Null, &Value::Null));
        
        assert!(!evaluator.values_equal(&Value::Number(42.0), &Value::Number(43.0)));
        assert!(!evaluator.values_equal(&Value::String("hello".to_string()), &Value::String("world".to_string())));
    }

    #[test]
    fn test_to_bool() {
        let variables = HashMap::new();
        let evaluator = Evaluator::new(&variables, 1);
        
        assert_eq!(evaluator.to_bool(&Value::Bool(true)), true);
        assert_eq!(evaluator.to_bool(&Value::Bool(false)), false);
        assert_eq!(evaluator.to_bool(&Value::Number(1.0)), true);
        assert_eq!(evaluator.to_bool(&Value::Number(0.0)), false);
        assert_eq!(evaluator.to_bool(&Value::String("hello".to_string())), true);
        assert_eq!(evaluator.to_bool(&Value::String("".to_string())), false);
        assert_eq!(evaluator.to_bool(&Value::Null), false);
    }

    #[test]
    fn test_parse_and_evaluate() {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), Value::Number(10.0));
        variables.insert("y".to_string(), Value::Number(5.0));
        
        let result = parse_and_evaluate("x + y", &variables, 1).unwrap();
        assert_eq!(result, Value::Number(15.0));
        
        let result = parse_and_evaluate("x * 2", &variables, 1).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }
}
