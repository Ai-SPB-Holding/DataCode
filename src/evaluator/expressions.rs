// Логика вычисления выражений в DataCode
// Обрабатывает различные типы выражений: литералы, переменные, массивы и т.д.

use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::parser::Expr;
use super::Evaluator;

/// Обработчик вычисления выражений
pub struct ExpressionEvaluator<'a> {
    evaluator: &'a Evaluator<'a>,
}

impl<'a> ExpressionEvaluator<'a> {
    /// Создать новый обработчик выражений
    pub fn new(evaluator: &'a Evaluator<'a>) -> Self {
        Self { evaluator }
    }
    
    /// Вычислить выражение
    pub fn evaluate(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(value) => self.evaluate_literal(value),
            Expr::Variable(name) => self.evaluate_variable(name),
            Expr::Binary { left, operator, right } => self.evaluate_binary(left, operator, right),
            Expr::Unary { operator, operand } => self.evaluate_unary(operator, operand),
            Expr::FunctionCall { name, args, named_args } => {
                // Вычисляем именованные аргументы
                let mut named_arg_values = std::collections::HashMap::new();
                for (arg_name, arg_expr) in named_args {
                    let value = self.evaluate(&arg_expr)?;
                    named_arg_values.insert(arg_name.clone(), value);
                }
                self.evaluate_function_call_with_named_args(name, args, named_arg_values)
            },
            Expr::Index { object, index } => self.evaluate_index(object, index),
            Expr::Member { object, member } => self.evaluate_member(object, member),
            Expr::ArrayLiteral { elements } => self.evaluate_array_literal(elements),
            Expr::ObjectLiteral { pairs } => self.evaluate_object_literal(pairs),
            Expr::Spread { expression } => self.evaluate_spread(expression),
            Expr::NamedArg { name: _, value } => {
                // NamedArg should only appear in function call arguments
                // Evaluate the value expression
                self.evaluate(value)
            },
            Expr::TryStmt { .. } => {
                // Try statements are handled at statement level, not expression level
                Err(DataCodeError::runtime_error("Try statement cannot be used as expression", 0))
            },
            Expr::ThrowStatement { message } => self.evaluate_throw_statement(message),
            // Statements should not appear in expression context
            Expr::Assignment { .. } | Expr::Declaration { .. } | Expr::ReturnStmt { .. } |
            Expr::PrintStmt { .. } | Expr::ExprStmt { .. } | Expr::Block { .. } |
            Expr::IfStmt { .. } | Expr::ForStmt { .. } | Expr::FunctionDef { .. } => {
                Err(DataCodeError::runtime_error("Statement cannot be used as expression", 0))
            },
        }
    }
    
    /// Вычислить литерал
    fn evaluate_literal(&self, value: &Value) -> Result<Value> {
        Ok(value.clone())
    }
    
    /// Вычислить переменную
    fn evaluate_variable(&self, name: &str) -> Result<Value> {
        self.evaluator.get_variable(name)
    }
    
    /// Вычислить бинарное выражение
    fn evaluate_binary(&self, left: &Expr, operator: &crate::parser::BinaryOp, right: &Expr) -> Result<Value> {
        let left_val = self.evaluate(left)?;
        let right_val = self.evaluate(right)?;
        self.evaluator.evaluate_binary_op(&left_val, operator, &right_val)
    }
    
    /// Вычислить унарное выражение
    fn evaluate_unary(&self, operator: &crate::parser::UnaryOp, operand: &Expr) -> Result<Value> {
        let operand_val = self.evaluate(operand)?;
        self.evaluator.evaluate_unary_op(operator, &operand_val)
    }
    
    /// Вычислить вызов функции
    #[allow(dead_code)]
    fn evaluate_function_call(&self, name: &str, args: &[Expr]) -> Result<Value> {
        self.evaluator.evaluate_function_call(name, args)
    }
    
    /// Вычислить вызов функции с именованными аргументами
    fn evaluate_function_call_with_named_args(
        &self,
        name: &str,
        args: &[Expr],
        named_args: std::collections::HashMap<String, Value>
    ) -> Result<Value> {
        // Вычисляем позиционные аргументы
        let mut arg_values = Vec::new();
        for arg in args {
            match arg {
                Expr::Spread { expression } => {
                    // Обрабатываем spread оператор
                    let spread_value = self.evaluate(expression)?;
                    // TODO: expand spread
                    arg_values.push(spread_value);
                }
                _ => {
                    arg_values.push(self.evaluate(arg)?);
                }
            }
        }
        
        // Вызываем функцию с именованными аргументами
        crate::builtins::call_builtin_function_with_named_args(
            name,
            arg_values,
            named_args,
            self.evaluator.line()
        )
    }
    
    /// Вычислить индексацию
    fn evaluate_index(&self, object: &Expr, index: &Expr) -> Result<Value> {
        let obj_val = self.evaluate(object)?;
        let idx_val = self.evaluate(index)?;
        self.evaluator.evaluate_index(&obj_val, &idx_val)
    }
    
    /// Вычислить доступ к члену объекта
    fn evaluate_member(&self, object: &Expr, member: &str) -> Result<Value> {
        let obj_val = self.evaluate(object)?;
        self.evaluator.evaluate_member(&obj_val, member)
    }
    
    /// Вычислить литерал массива
    fn evaluate_array_literal(&self, elements: &[Expr]) -> Result<Value> {
        let mut array_values = Vec::new();
        for element in elements {
            array_values.push(self.evaluate(element)?);
        }
        Ok(Value::Array(array_values))
    }

    /// Вычислить литерал объекта
    fn evaluate_object_literal(&self, pairs: &[(String, Expr)]) -> Result<Value> {
        let mut object_map = std::collections::HashMap::new();
        for (key, value_expr) in pairs {
            let value = self.evaluate(value_expr)?;
            object_map.insert(key.clone(), value);
        }
        Ok(Value::Object(object_map))
    }

    /// Вычислить spread выражение
    fn evaluate_spread(&self, _expression: &Expr) -> Result<Value> {
        // Spread выражения не должны вычисляться напрямую
        // Они обрабатываются специально в контексте вызова функций
        Err(DataCodeError::runtime_error(
            "Spread operator can only be used in function calls",
            self.evaluator.line()
        ))
    }

    /// Обработать try блок (не поддерживается в выражениях)
    #[allow(dead_code)]
    fn evaluate_try_block(&self) -> Result<Value> {
        // Try блоки не должны вычисляться как выражения в evaluator
        // Они обрабатываются в интерпретаторе
        Err(DataCodeError::syntax_error("Try blocks are not supported in expressions", 1, 0))
    }
    
    /// Обработать throw statement
    fn evaluate_throw_statement(&self, message: &Expr) -> Result<Value> {
        let msg_value = self.evaluate(message)?;
        let msg_str = match msg_value {
            Value::String(s) => s,
            other => format!("{:?}", other),
        };
        Err(DataCodeError::runtime_error(&format!("Exception: {}", msg_str), 1))
    }
}

/// Трейт для вычисления выражений
pub trait ExpressionEvaluable {
    /// Вычислить выражение с заданным вычислителем
    #[allow(dead_code)]
    fn evaluate_with(&self, evaluator: &Evaluator) -> Result<Value>;
}

impl ExpressionEvaluable for Expr {
    fn evaluate_with(&self, evaluator: &Evaluator) -> Result<Value> {
        let expr_evaluator = ExpressionEvaluator::new(evaluator);
        expr_evaluator.evaluate(self)
    }
}

/// Вспомогательные функции для работы с выражениями
pub mod utils {
    use super::*;
    
    /// Проверить, является ли выражение константой
    #[allow(dead_code)]
    pub fn is_constant_expression(expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_) => true,
            Expr::ArrayLiteral { elements } => {
                elements.iter().all(is_constant_expression)
            }
            _ => false,
        }
    }
    
    /// Получить все переменные, используемые в выражении
    #[allow(dead_code)]
    pub fn get_variables_in_expression(expr: &Expr) -> Vec<String> {
        let mut variables = Vec::new();
        collect_variables(expr, &mut variables);
        variables.sort();
        variables.dedup();
        variables
    }
    
    fn collect_variables(expr: &Expr, variables: &mut Vec<String>) {
        match expr {
            Expr::Variable(name) => variables.push(name.clone()),
            Expr::Binary { left, right, .. } => {
                collect_variables(left, variables);
                collect_variables(right, variables);
            }
            Expr::Unary { operand, .. } => {
                collect_variables(operand, variables);
            }
            Expr::FunctionCall { args, .. } => {
                for arg in args {
                    collect_variables(arg, variables);
                }
            }
            Expr::Index { object, index } => {
                collect_variables(object, variables);
                collect_variables(index, variables);
            }
            Expr::Member { object, .. } => {
                collect_variables(object, variables);
            }
            Expr::ArrayLiteral { elements } => {
                for element in elements {
                    collect_variables(element, variables);
                }
            }
            Expr::ThrowStatement { message } => {
                collect_variables(message, variables);
            }
            _ => {}
        }
    }
    
    /// Оценить сложность выражения (количество операций)
    #[allow(dead_code)]
    pub fn expression_complexity(expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => 1,
            Expr::Binary { left, right, .. } => {
                1 + expression_complexity(left) + expression_complexity(right)
            }
            Expr::Unary { operand, .. } => {
                1 + expression_complexity(operand)
            }
            Expr::FunctionCall { args, .. } => {
                1 + args.iter().map(expression_complexity).sum::<usize>()
            }
            Expr::Index { object, index } => {
                1 + expression_complexity(object) + expression_complexity(index)
            }
            Expr::Member { object, .. } => {
                1 + expression_complexity(object)
            }
            Expr::ArrayLiteral { elements } => {
                1 + elements.iter().map(expression_complexity).sum::<usize>()
            }
            Expr::ThrowStatement { message } => {
                1 + expression_complexity(message)
            }
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    fn create_test_evaluator() -> (HashMap<String, Value>, Evaluator<'static>) {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), Value::Number(10.0));
        variables.insert("y".to_string(), Value::Number(5.0));
        variables.insert("name".to_string(), Value::String("test".to_string()));
        
        // Используем Box::leak для создания статической ссылки в тестах
        let static_vars = Box::leak(Box::new(variables));
        let evaluator = Evaluator::new(static_vars, 1);
        (HashMap::new(), evaluator)
    }

    #[test]
    fn test_evaluate_literal() {
        let (_, evaluator) = create_test_evaluator();
        let expr_evaluator = ExpressionEvaluator::new(&evaluator);
        
        let result = expr_evaluator.evaluate_literal(&Value::Number(42.0)).unwrap();
        assert_eq!(result, Value::Number(42.0));
        
        let result = expr_evaluator.evaluate_literal(&Value::String("hello".to_string())).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_evaluate_variable() {
        let (_, evaluator) = create_test_evaluator();
        let expr_evaluator = ExpressionEvaluator::new(&evaluator);
        
        let result = expr_evaluator.evaluate_variable("x").unwrap();
        assert_eq!(result, Value::Number(10.0));
        
        let error = expr_evaluator.evaluate_variable("unknown");
        assert!(error.is_err());
    }

    #[test]
    fn test_evaluate_array_literal() {
        let (_, evaluator) = create_test_evaluator();
        let expr_evaluator = ExpressionEvaluator::new(&evaluator);
        
        let elements = vec![
            Expr::Literal(Value::Number(1.0)),
            Expr::Literal(Value::Number(2.0)),
            Expr::Literal(Value::Number(3.0)),
        ];
        
        let result = expr_evaluator.evaluate_array_literal(&elements).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Number(1.0));
                assert_eq!(arr[1], Value::Number(2.0));
                assert_eq!(arr[2], Value::Number(3.0));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_utils_is_constant_expression() {
        assert!(utils::is_constant_expression(&Expr::Literal(Value::Number(42.0))));
        assert!(!utils::is_constant_expression(&Expr::Variable("x".to_string())));
        
        let array_expr = Expr::ArrayLiteral {
            elements: vec![
                Expr::Literal(Value::Number(1.0)),
                Expr::Literal(Value::Number(2.0)),
            ]
        };
        assert!(utils::is_constant_expression(&array_expr));
    }

    #[test]
    fn test_utils_expression_complexity() {
        assert_eq!(utils::expression_complexity(&Expr::Literal(Value::Number(42.0))), 1);
        assert_eq!(utils::expression_complexity(&Expr::Variable("x".to_string())), 1);
        
        let binary_expr = Expr::Binary {
            left: Box::new(Expr::Literal(Value::Number(1.0))),
            operator: crate::parser::BinaryOp::Add,
            right: Box::new(Expr::Literal(Value::Number(2.0))),
        };
        assert_eq!(utils::expression_complexity(&binary_expr), 3); // 1 + 1 + 1
    }
}
