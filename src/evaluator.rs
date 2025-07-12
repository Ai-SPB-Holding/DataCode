use crate::value::Value;
use crate::value::Value::{Number, String as ValueString, Bool, Array, Path, Null, Object};
use crate::error::{DataCodeError, Result};
use crate::parser::{Expr, BinaryOp, UnaryOp};
use crate::builtins::call_function;
use std::collections::HashMap;

pub struct Evaluator<'a> {
    variables: &'a HashMap<String, Value>,
    line: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(variables: &'a HashMap<String, Value>, line: usize) -> Self {
        Self { variables, line }
    }
    
    pub fn evaluate(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            
            Expr::Variable(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| DataCodeError::variable_not_found(name, self.line))
            }
            
            Expr::Binary { left, operator, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.evaluate_binary_op(&left_val, operator, &right_val)
            }
            
            Expr::Unary { operator, operand } => {
                let operand_val = self.evaluate(operand)?;
                self.evaluate_unary_op(operator, &operand_val)
            }
            
            Expr::FunctionCall { name, args } => {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate(arg)?);
                }
                call_function(name, arg_values, self.line)
            }
            
            Expr::Index { object, index } => {
                let obj_val = self.evaluate(object)?;
                let idx_val = self.evaluate(index)?;
                self.evaluate_index(&obj_val, &idx_val)
            }
            
            Expr::Member { object, member } => {
                let obj_val = self.evaluate(object)?;
                self.evaluate_member(&obj_val, member)
            }
        }
    }
    
    fn evaluate_binary_op(&self, left: &Value, op: &BinaryOp, right: &Value) -> Result<Value> {
        use Value::*;
        
        match op {
            BinaryOp::Add => left.add(right).map_err(|e| DataCodeError::runtime_error(&e, self.line)),
            
            BinaryOp::PathJoin => left.add(right).map_err(|e| DataCodeError::runtime_error(&e, self.line)),
            
            BinaryOp::Subtract => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a - b)),
                _ => Err(DataCodeError::type_error("Number", "other", self.line)),
            },
            
            BinaryOp::Multiply => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a * b)),
                (ValueString(s), Number(n)) => {
                    let count = *n as usize;
                    Ok(ValueString(s.repeat(count)))
                }
                _ => Err(DataCodeError::type_error("Number", "other", self.line)),
            },
            
            BinaryOp::Divide => {
                // Интеллектуальная обработка оператора /
                // Если левый операнд - Path, то это PathJoin
                // Если оба операнда - числа, то это математическое деление
                match (left, right) {
                    (Path(p), ValueString(s)) => {
                        let mut path = p.clone();
                        path.push(s);
                        Ok(Path(path))
                    }
                    (Path(p1), Path(p2)) => {
                        let mut path = p1.clone();
                        path.push(p2);
                        Ok(Path(path))
                    }
                    (Number(a), Number(b)) => {
                        if *b == 0.0 {
                            Err(DataCodeError::runtime_error("Division by zero", self.line))
                        } else {
                            Ok(Number(a / b))
                        }
                    }
                    _ => {
                        // Если типы не подходят ни для PathJoin, ни для деления
                        Err(DataCodeError::runtime_error(
                            "Invalid operands for / operator. Use Path/String for path joining or Number/Number for division",
                            self.line
                        ))
                    }
                }
            },
            
            BinaryOp::Equal => Ok(Bool(self.values_equal(left, right))),
            
            BinaryOp::NotEqual => Ok(Bool(!self.values_equal(left, right))),
            
            BinaryOp::Less => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a < b)),
                (String(a), String(b)) => Ok(Bool(a < b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.line)),
            },
            
            BinaryOp::Greater => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a > b)),
                (String(a), String(b)) => Ok(Bool(a > b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.line)),
            },
            
            BinaryOp::LessEqual => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a <= b)),
                (String(a), String(b)) => Ok(Bool(a <= b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.line)),
            },
            
            BinaryOp::GreaterEqual => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a >= b)),
                (String(a), String(b)) => Ok(Bool(a >= b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.line)),
            },
            
            BinaryOp::And => {
                let left_bool = self.to_bool(left);
                if !left_bool {
                    Ok(Bool(false))
                } else {
                    Ok(Bool(self.to_bool(right)))
                }
            },
            
            BinaryOp::Or => {
                let left_bool = self.to_bool(left);
                if left_bool {
                    Ok(Bool(true))
                } else {
                    Ok(Bool(self.to_bool(right)))
                }
            },
        }
    }
    
    fn evaluate_unary_op(&self, op: &UnaryOp, operand: &Value) -> Result<Value> {
        match op {
            UnaryOp::Not => Ok(Bool(!self.to_bool(operand))),
            UnaryOp::Minus => match operand {
                Number(n) => Ok(Number(-n)),
                _ => Err(DataCodeError::type_error("Number", "other", self.line)),
            },
        }
    }
    
    fn evaluate_index(&self, object: &Value, index: &Value) -> Result<Value> {
        match (object, index) {
            (Array(arr), Number(n)) => {
                let idx = *n as usize;
                arr.get(idx)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error("Index out of bounds", self.line))
            }
            (Object(obj), ValueString(key)) => {
                obj.get(key)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error(&format!("Key '{}' not found", key), self.line))
            }
            _ => Err(DataCodeError::type_error("indexable type", "other", self.line)),
        }
    }
    
    fn evaluate_member(&self, object: &Value, member: &str) -> Result<Value> {
        match object {
            Object(obj) => {
                obj.get(member)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error(&format!("Member '{}' not found", member), self.line))
            }
            Array(arr) => {
                match member {
                    "length" => Ok(Number(arr.len() as f64)),
                    _ => Err(DataCodeError::runtime_error(&format!("Array has no member '{}'", member), self.line)),
                }
            }
            ValueString(s) => {
                match member {
                    "length" => Ok(Number(s.len() as f64)),
                    _ => Err(DataCodeError::runtime_error(&format!("String has no member '{}'", member), self.line)),
                }
            }
            _ => Err(DataCodeError::type_error("object with members", "other", self.line)),
        }
    }
    
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
            (ValueString(a), ValueString(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Null, Null) => true,
            (Array(a), Array(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y)),
            _ => false,
        }
    }
    
    fn to_bool(&self, value: &Value) -> bool {
        match value {
            Bool(b) => *b,
            Number(n) => *n != 0.0,
            ValueString(s) => !s.is_empty(),
            Array(arr) => !arr.is_empty(),
            Object(obj) => !obj.is_empty(),
            Null => false,
            Path(p) => p.exists(),
        }
    }
}

// Вспомогательная функция для быстрого парсинга и вычисления выражений
pub fn parse_and_evaluate(expr_str: &str, variables: &HashMap<String, Value>, line: usize) -> Result<Value> {
    let mut parser = crate::parser::Parser::new(expr_str);
    let expr = parser.parse_expression()?;
    let evaluator = Evaluator::new(variables, line);
    evaluator.evaluate(&expr)
}
