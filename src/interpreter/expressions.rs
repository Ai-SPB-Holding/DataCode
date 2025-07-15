use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::parser::{Expr, BinaryOp, UnaryOp};
use crate::builtins::call_builtin_function;
use super::variables::VariableManager;
use super::user_functions::UserFunctionManager;

/// Вычислитель выражений для интерпретатора
pub struct ExpressionEvaluator<'a> {
    pub variable_manager: &'a VariableManager,
    pub function_manager: &'a UserFunctionManager,
    pub current_line: usize,
}

impl<'a> ExpressionEvaluator<'a> {
    pub fn new(
        variable_manager: &'a VariableManager,
        function_manager: &'a UserFunctionManager,
        current_line: usize,
    ) -> Self {
        Self {
            variable_manager,
            function_manager,
            current_line,
        }
    }

    /// Вычислить выражение
    pub fn evaluate(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),

            Expr::Variable(name) => {
                self.variable_manager
                    .get_variable(name)
                    .cloned()
                    .ok_or_else(|| DataCodeError::variable_not_found(name, self.current_line))
            }

            Expr::FunctionCall { name, args } => {
                // Проверяем, является ли это пользовательской функцией
                if self.function_manager.contains_function(name) {
                    // Для пользовательских функций возвращаем специальную ошибку,
                    // которая будет обработана на уровне интерпретатора
                    Err(DataCodeError::runtime_error(
                        &format!("USER_FUNCTION_CALL_EXPR:{}:{}", name, args.len()),
                        self.current_line,
                    ))
                } else {
                    // Сначала вычисляем аргументы для встроенных функций
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.evaluate(arg)?);
                    }
                    // Встроенная функция
                    call_builtin_function(name, arg_values, self.current_line)
                }
            }

            Expr::Binary { left, operator, right } => {
                // Сначала пытаемся вычислить левую часть
                let left_val = match self.evaluate(left) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                    }
                    result => result?
                };

                // Затем пытаемся вычислить правую часть
                let right_val = match self.evaluate(right) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                    }
                    result => result?
                };

                self.evaluate_binary_op(&left_val, operator, &right_val)
            }

            Expr::Unary { operator, operand } => {
                let operand_val = match self.evaluate(operand) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                    }
                    result => result?
                };
                self.evaluate_unary_op(operator, &operand_val)
            }

            Expr::Index { object, index } => {
                let obj_val = match self.evaluate(object) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                    }
                    result => result?
                };
                let idx_val = match self.evaluate(index) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                    }
                    result => result?
                };
                self.evaluate_index_access(&obj_val, &idx_val)
            }

            Expr::ArrayLiteral { elements } => {
                let mut array_values = Vec::new();
                for element in elements {
                    let element_val = match self.evaluate(element) {
                        Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                            return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                        }
                        result => result?
                    };
                    array_values.push(element_val);
                }
                Ok(Value::Array(array_values))
            }

            Expr::ObjectLiteral { pairs } => {
                let mut object_map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = match self.evaluate(value_expr) {
                        Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                            return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                        }
                        result => result?
                    };
                    object_map.insert(key.clone(), value);
                }
                Ok(Value::Object(object_map))
            }

            Expr::Member { object, member } => {
                let obj_val = match self.evaluate(object) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        return Err(e); // Пробрасываем ошибку вызова пользовательской функции
                    }
                    result => result?
                };
                self.evaluate_member_access(&obj_val, member)
            }

            Expr::Spread { .. } => Err(DataCodeError::runtime_error(
                "Spread operator can only be used in function calls",
                self.current_line,
            )),

            _ => Err(DataCodeError::runtime_error(
                "Unsupported expression type",
                self.current_line,
            )),
        }
    }

    /// Вычислить бинарную операцию
    fn evaluate_binary_op(&self, left: &Value, op: &BinaryOp, right: &Value) -> Result<Value> {
        use BinaryOp::*;
        use Value::*;

        match op {
            Add => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a + b)),
                (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
                (String(a), Number(b)) => Ok(String(format!("{}{}", a, b))),
                (Number(a), String(b)) => Ok(String(format!("{}{}", a, b))),
                (Path(p), String(s)) => {
                    let result = p.join(s);
                    if s.contains('*') || s.contains('?') || s.contains('[') {
                        Ok(Value::PathPattern(result))
                    } else {
                        Ok(Path(result))
                    }
                }
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot add {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            PathJoin => match (left, right) {
                (Path(p), String(s)) => {
                    let result = p.join(s);
                    if s.contains('*') || s.contains('?') || s.contains('[') {
                        Ok(Value::PathPattern(result))
                    } else {
                        Ok(Path(result))
                    }
                }
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot join path {:?} with {:?}", left, right),
                    self.current_line,
                )),
            },

            Subtract => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a - b)),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot subtract {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            Multiply => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a * b)),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot multiply {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            Divide => match (left, right) {
                (Number(a), Number(b)) => {
                    if *b == 0.0 {
                        Err(DataCodeError::runtime_error("Division by zero", self.current_line))
                    } else {
                        Ok(Number(a / b))
                    }
                }
                (Path(p), String(s)) => {
                    let result = p.join(s);
                    if s.contains('*') || s.contains('?') || s.contains('[') {
                        Ok(Value::PathPattern(result))
                    } else {
                        Ok(Path(result))
                    }
                }
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot divide {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            Modulo => match (left, right) {
                (Number(a), Number(b)) => {
                    if *b == 0.0 {
                        Err(DataCodeError::runtime_error("Modulo by zero", self.current_line))
                    } else {
                        Ok(Number(a % b))
                    }
                }
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot modulo {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            Equal => Ok(Bool(self.values_equal(left, right))),
            NotEqual => Ok(Bool(!self.values_equal(left, right))),

            Less => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a < b)),
                (String(a), String(b)) => Ok(Bool(a < b)),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot compare {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            Greater => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a > b)),
                (String(a), String(b)) => Ok(Bool(a > b)),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot compare {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            LessEqual => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a <= b)),
                (String(a), String(b)) => Ok(Bool(a <= b)),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot compare {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            GreaterEqual => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a >= b)),
                (String(a), String(b)) => Ok(Bool(a >= b)),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot compare {:?} and {:?}", left, right),
                    self.current_line,
                )),
            },

            And => {
                let left_bool = self.to_bool(left);
                if !left_bool {
                    Ok(Bool(false))
                } else {
                    Ok(Bool(self.to_bool(right)))
                }
            },

            Or => {
                let left_bool = self.to_bool(left);
                if left_bool {
                    Ok(Bool(true))
                } else {
                    Ok(Bool(self.to_bool(right)))
                }
            },
        }
    }

    /// Вычислить унарную операцию
    fn evaluate_unary_op(&self, op: &UnaryOp, operand: &Value) -> Result<Value> {
        use UnaryOp::*;
        use Value::*;

        match op {
            Minus => match operand {
                Number(n) => Ok(Number(-n)),
                _ => Err(DataCodeError::runtime_error(
                    &format!("Cannot negate {:?}", operand),
                    self.current_line,
                )),
            },

            Not => Ok(Bool(!self.to_bool(operand))),
        }
    }

    /// Вычислить доступ по индексу
    fn evaluate_index_access(&self, object: &Value, index: &Value) -> Result<Value> {
        use Value::*;

        match (object, index) {
            (Array(arr), Number(n)) => {
                let idx = *n as usize;
                if idx < arr.len() {
                    Ok(arr[idx].clone())
                } else {
                    Err(DataCodeError::runtime_error(
                        &format!("Array index {} out of bounds", idx),
                        self.current_line,
                    ))
                }
            }
            (Table(table), String(column_name)) => {
                let table_borrowed = table.borrow();
                if let Some(col_index) = table_borrowed.column_names.iter().position(|name| name == column_name) {
                    let column_data: Vec<Value> = table_borrowed.rows.iter()
                        .map(|row| row.get(col_index).cloned().unwrap_or(Null))
                        .collect();
                    Ok(Array(column_data))
                } else {
                    Err(DataCodeError::runtime_error(
                        &format!("Column '{}' not found in table", column_name),
                        self.current_line,
                    ))
                }
            }
            (Object(obj), String(key)) => {
                obj.get(key)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error(
                        &format!("Key '{}' not found in object", key),
                        self.current_line,
                    ))
            }
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot index {:?} with {:?}", object, index),
                self.current_line,
            )),
        }
    }

    /// Проверить равенство значений
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
            (String(a), String(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Null, Null) => true,
            _ => false,
        }
    }

    /// Вычислить доступ к члену объекта
    fn evaluate_member_access(&self, object: &Value, member: &str) -> Result<Value> {
        use Value::*;

        match object {
            Object(obj) => {
                obj.get(member)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error(
                        &format!("Member '{}' not found in object", member),
                        self.current_line,
                    ))
            }
            Array(arr) => {
                // Поддержка свойств массива, например arr.length
                match member {
                    "length" | "len" => Ok(Number(arr.len() as f64)),
                    _ => Err(DataCodeError::runtime_error(
                        &format!("Array has no member '{}'", member),
                        self.current_line,
                    ))
                }
            }
            String(s) => {
                // Поддержка свойств строки, например str.length
                match member {
                    "length" | "len" => Ok(Number(s.len() as f64)),
                    _ => Err(DataCodeError::runtime_error(
                        &format!("String has no member '{}'", member),
                        self.current_line,
                    ))
                }
            }
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot access member '{}' on {:?}", member, object),
                self.current_line,
            )),
        }
    }

    /// Преобразовать значение в булево
    fn to_bool(&self, value: &Value) -> bool {
        use Value::*;
        match value {
            Bool(b) => *b,
            Number(n) => *n != 0.0,
            String(s) => !s.is_empty(),
            Currency(c) => !c.is_empty(),
            Array(arr) => !arr.is_empty(),
            Object(obj) => !obj.is_empty(),
            Table(table) => !table.borrow().rows.is_empty(),
            Null => false,
            Path(p) => p.exists(),
            PathPattern(_) => true,
        }
    }
}
