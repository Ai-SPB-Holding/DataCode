// Логика обработки операторов в DataCode
// Включает бинарные и унарные операторы с интеллектуальной типизацией

use crate::value::{Value, ValueOperations};
use crate::value::Value::{Number, String, Bool, Path, PathPattern};
use crate::error::{DataCodeError, Result};
use crate::parser::{BinaryOp, UnaryOp};
use super::Evaluator;

/// Обработчик бинарных операторов
pub struct BinaryOperatorHandler<'a> {
    evaluator: &'a Evaluator<'a>,
}

impl<'a> BinaryOperatorHandler<'a> {
    /// Создать новый обработчик бинарных операторов
    pub fn new(evaluator: &'a Evaluator<'a>) -> Self {
        Self { evaluator }
    }
    
    /// Вычислить бинарный оператор
    pub fn evaluate(&self, left: &Value, op: &BinaryOp, right: &Value) -> Result<Value> {
        use Value::*;
        
        match op {
            BinaryOp::Add => self.evaluate_add(left, right),
            BinaryOp::PathJoin => self.evaluate_path_join(left, right),
            BinaryOp::Subtract => self.evaluate_subtract(left, right),
            BinaryOp::Multiply => self.evaluate_multiply(left, right),
            BinaryOp::Divide => self.evaluate_divide(left, right),
            BinaryOp::Modulo => self.evaluate_modulo(left, right),
            BinaryOp::Equal => Ok(Bool(self.evaluator.values_equal(left, right))),
            BinaryOp::NotEqual => Ok(Bool(!self.evaluator.values_equal(left, right))),
            BinaryOp::Less => self.evaluate_less(left, right),
            BinaryOp::Greater => self.evaluate_greater(left, right),
            BinaryOp::LessEqual => self.evaluate_less_equal(left, right),
            BinaryOp::GreaterEqual => self.evaluate_greater_equal(left, right),
            BinaryOp::And => self.evaluate_and(left, right),
            BinaryOp::Or => self.evaluate_or(left, right),
        }
    }
    
    /// Сложение с интеллектуальной типизацией
    fn evaluate_add(&self, left: &Value, right: &Value) -> Result<Value> {
        left.add(right).map_err(|e| DataCodeError::runtime_error(&e, self.evaluator.line()))
    }
    
    /// Соединение путей
    fn evaluate_path_join(&self, left: &Value, right: &Value) -> Result<Value> {
        left.add(right).map_err(|e| DataCodeError::runtime_error(&e, self.evaluator.line()))
    }
    
    /// Вычитание
    fn evaluate_subtract(&self, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Number(a), Number(b)) => Ok(Number(a - b)),
            _ => Err(DataCodeError::type_error("Number", "other", self.evaluator.line())),
        }
    }
    
    /// Умножение с поддержкой строк
    fn evaluate_multiply(&self, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Number(a), Number(b)) => Ok(Number(a * b)),
            (String(s), Number(n)) => {
                if *n >= 0.0 && n.fract() == 0.0 {
                    let count = *n as usize;
                    Ok(String(s.repeat(count)))
                } else {
                    Err(DataCodeError::runtime_error("String multiplication requires non-negative integer", self.evaluator.line()))
                }
            }
            (Number(n), String(s)) => {
                if *n >= 0.0 && n.fract() == 0.0 {
                    let count = *n as usize;
                    Ok(String(s.repeat(count)))
                } else {
                    Err(DataCodeError::runtime_error("String multiplication requires non-negative integer", self.evaluator.line()))
                }
            }
            _ => Err(DataCodeError::type_error("Number or String", "other", self.evaluator.line())),
        }
    }
    
    /// Деление с интеллектуальной обработкой путей
    fn evaluate_divide(&self, left: &Value, right: &Value) -> Result<Value> {
        // Интеллектуальная обработка оператора /
        // Если левый операнд - Path, то это PathJoin
        // Если оба операнда - числа, то это математическое деление
        match (left, right) {
            (Path(p), String(s)) => {
                let mut path = p.clone();
                let relative = s.trim_start_matches('/');
                path.push(relative);
                // Проверяем, содержит ли строка glob паттерны
                if s.contains('*') || s.contains('?') || s.contains('[') {
                    Ok(PathPattern(path))
                } else {
                    Ok(Path(path))
                }
            }
            (Path(p1), Path(p2)) => {
                let mut path = p1.clone();
                path.push(p2);
                Ok(Path(path))
            }
            (Number(a), Number(b)) => {
                if *b == 0.0 {
                    Err(DataCodeError::runtime_error("Division by zero", self.evaluator.line()))
                } else {
                    Ok(Number(a / b))
                }
            }
            _ => {
                // Если типы не подходят ни для PathJoin, ни для деления
                Err(DataCodeError::runtime_error(
                    "Invalid operands for / operator. Use Path/String for path joining or Number/Number for division",
                    self.evaluator.line()
                ))
            }
        }
    }

    /// Остаток от деления (модуло)
    fn evaluate_modulo(&self, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Number(a), Number(b)) => {
                if *b == 0.0 {
                    Err(DataCodeError::runtime_error("Modulo by zero", self.evaluator.line()))
                } else {
                    Ok(Number(a % b))
                }
            }
            _ => Err(DataCodeError::type_error("Number", "other", self.evaluator.line())),
        }
    }

    /// Сравнение "меньше"
    fn evaluate_less(&self, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Number(a), Number(b)) => Ok(Bool(a < b)),
            (String(a), String(b)) => Ok(Bool(a < b)),
            _ => Err(DataCodeError::type_error("comparable types", "other", self.evaluator.line())),
        }
    }
    
    /// Сравнение "больше"
    fn evaluate_greater(&self, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Number(a), Number(b)) => Ok(Bool(a > b)),
            (String(a), String(b)) => Ok(Bool(a > b)),
            _ => Err(DataCodeError::type_error("comparable types", "other", self.evaluator.line())),
        }
    }
    
    /// Сравнение "меньше или равно"
    fn evaluate_less_equal(&self, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Number(a), Number(b)) => Ok(Bool(a <= b)),
            (String(a), String(b)) => Ok(Bool(a <= b)),
            _ => Err(DataCodeError::type_error("comparable types", "other", self.evaluator.line())),
        }
    }
    
    /// Сравнение "больше или равно"
    fn evaluate_greater_equal(&self, left: &Value, right: &Value) -> Result<Value> {
        match (left, right) {
            (Number(a), Number(b)) => Ok(Bool(a >= b)),
            (String(a), String(b)) => Ok(Bool(a >= b)),
            _ => Err(DataCodeError::type_error("comparable types", "other", self.evaluator.line())),
        }
    }
    
    /// Логическое И
    fn evaluate_and(&self, left: &Value, right: &Value) -> Result<Value> {
        let left_bool = self.evaluator.to_bool(left);
        let right_bool = self.evaluator.to_bool(right);
        Ok(Bool(left_bool && right_bool))
    }
    
    /// Логическое ИЛИ
    fn evaluate_or(&self, left: &Value, right: &Value) -> Result<Value> {
        let left_bool = self.evaluator.to_bool(left);
        let right_bool = self.evaluator.to_bool(right);
        Ok(Bool(left_bool || right_bool))
    }
}

/// Обработчик унарных операторов
pub struct UnaryOperatorHandler<'a> {
    evaluator: &'a Evaluator<'a>,
}

impl<'a> UnaryOperatorHandler<'a> {
    /// Создать новый обработчик унарных операторов
    pub fn new(evaluator: &'a Evaluator<'a>) -> Self {
        Self { evaluator }
    }
    
    /// Вычислить унарный оператор
    pub fn evaluate(&self, op: &UnaryOp, operand: &Value) -> Result<Value> {
        match op {
            UnaryOp::Not => self.evaluate_not(operand),
            UnaryOp::Minus => self.evaluate_minus(operand),
        }
    }
    
    /// Логическое НЕ
    fn evaluate_not(&self, operand: &Value) -> Result<Value> {
        Ok(Bool(!self.evaluator.to_bool(operand)))
    }
    
    /// Унарный минус
    fn evaluate_minus(&self, operand: &Value) -> Result<Value> {
        match operand {
            Number(n) => Ok(Number(-n)),
            _ => Err(DataCodeError::type_error("Number", "other", self.evaluator.line())),
        }
    }
    

}

/// Трейт для операторов
pub trait OperatorEvaluable {
    /// Получить приоритет оператора
    fn precedence(&self) -> u8;
    
    /// Проверить, является ли оператор левоассоциативным
    fn is_left_associative(&self) -> bool;
}

impl OperatorEvaluable for BinaryOp {
    fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Equal | BinaryOp::NotEqual => 3,
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual => 4,
            BinaryOp::Add | BinaryOp::Subtract => 5,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 6,
            BinaryOp::PathJoin => 7,
        }
    }
    
    fn is_left_associative(&self) -> bool {
        match self {
            BinaryOp::PathJoin => false, // Правоассоциативный для цепочки путей
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_evaluator() -> Evaluator<'static> {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), Value::Number(10.0));
        variables.insert("y".to_string(), Value::Number(5.0));
        
        let static_vars = Box::leak(Box::new(variables));
        Evaluator::new(static_vars, 1)
    }

    #[test]
    fn test_binary_add() {
        let evaluator = create_test_evaluator();
        let handler = BinaryOperatorHandler::new(&evaluator);
        
        let result = handler.evaluate(&Value::Number(2.0), &BinaryOp::Add, &Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::Number(5.0));
        
        let result = handler.evaluate(&Value::String("hello".to_string()), &BinaryOp::Add, &Value::String(" world".to_string())).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_binary_multiply() {
        let evaluator = create_test_evaluator();
        let handler = BinaryOperatorHandler::new(&evaluator);
        
        let result = handler.evaluate(&Value::Number(3.0), &BinaryOp::Multiply, &Value::Number(4.0)).unwrap();
        assert_eq!(result, Value::Number(12.0));
        
        let result = handler.evaluate(&Value::String("hi".to_string()), &BinaryOp::Multiply, &Value::Number(3.0)).unwrap();
        assert_eq!(result, Value::String("hihihi".to_string()));
    }

    #[test]
    fn test_binary_divide() {
        let evaluator = create_test_evaluator();
        let handler = BinaryOperatorHandler::new(&evaluator);
        
        let result = handler.evaluate(&Value::Number(10.0), &BinaryOp::Divide, &Value::Number(2.0)).unwrap();
        assert_eq!(result, Value::Number(5.0));
        
        let error = handler.evaluate(&Value::Number(10.0), &BinaryOp::Divide, &Value::Number(0.0));
        assert!(error.is_err());
    }

    #[test]
    fn test_binary_comparison() {
        let evaluator = create_test_evaluator();
        let handler = BinaryOperatorHandler::new(&evaluator);
        
        let result = handler.evaluate(&Value::Number(5.0), &BinaryOp::Less, &Value::Number(10.0)).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = handler.evaluate(&Value::Number(10.0), &BinaryOp::Greater, &Value::Number(5.0)).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        let result = handler.evaluate(&Value::Number(5.0), &BinaryOp::Equal, &Value::Number(5.0)).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_unary_operators() {
        let evaluator = create_test_evaluator();
        let handler = UnaryOperatorHandler::new(&evaluator);
        
        let result = handler.evaluate(&UnaryOp::Minus, &Value::Number(5.0)).unwrap();
        assert_eq!(result, Value::Number(-5.0));
        

        
        let result = handler.evaluate(&UnaryOp::Not, &Value::Bool(true)).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_operator_precedence() {
        assert!(BinaryOp::Multiply.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
        assert!(BinaryOp::Equal.precedence() > BinaryOp::And.precedence());
    }

    #[test]
    fn test_operator_associativity() {
        assert!(BinaryOp::Add.is_left_associative());
        assert!(BinaryOp::Multiply.is_left_associative());
        // PathJoin не используется в новой архитектуре, поэтому убираем тест
    }
}
