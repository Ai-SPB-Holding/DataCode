// Вычисление выражений с возвратом сигналов вместо прямого выполнения
// Это позволяет избежать рекурсии Rust при обработке вызовов функций

use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::parser::Expr;
use crate::builtins::call_builtin_function_with_named_args;
use super::{Interpreter, ExecSignal};

impl Interpreter {
    /// Вычислить выражение и вернуть сигнал (Value, Call или Return)
    /// НЕ выполняет функции напрямую, а возвращает сигнал для главного цикла
    pub fn evaluate_expression_signal(&mut self, expr: &Expr) -> Result<ExecSignal> {
        match expr {
            Expr::Literal(value) => Ok(ExecSignal::Value(value.clone())),

            Expr::Variable(name) => {
                let value = self.get_variable(name)
                    .cloned()
                    .ok_or_else(|| DataCodeError::variable_not_found(name, self.current_line))?;
                Ok(ExecSignal::Value(value))
            }

            Expr::FunctionCall { name, args, named_args } => {
                // Вычисляем позиционные аргументы
                // ВАЖНО: Если аргумент содержит вызов функции, выполняем его через главный цикл
                let mut arg_values = Vec::new();
                for arg in args {
                    match arg {
                        Expr::Spread { expression } => {
                            let spread_signal = self.evaluate_expression_signal(expression)?;
                            let spread_value = match spread_signal {
                                ExecSignal::Value(v) => v,
                                ExecSignal::Call { function_id, args, return_slot } => {
                                    // Вложенный вызов функции в spread - возвращаем сигнал Call
                                    // ВАЖНО: НЕ вызываем call_user_function напрямую, чтобы избежать рекурсии Rust
                                    // Главный цикл call_user_function обработает его через единый цикл
                                    if std::env::var("DATACODE_DEBUG").is_ok() {
                                        eprintln!("🔍 DEBUG evaluate_expression_signal: Returning Call signal for function in spread: {}({:?})", function_id, args);
                                    }
                                    // Создаем временный слот для результата
                                    let temp_slot = format!("__spread_{}", self.call_stack.len());
                                    return Ok(ExecSignal::Call {
                                        function_id,
                                        args,
                                        return_slot: return_slot.or(Some(temp_slot)),
                                    });
                                }
                                ExecSignal::Return(_) => {
                                    return Err(DataCodeError::runtime_error(
                                        "Return statement cannot be used in spread operator",
                                        self.current_line
                                    ));
                                }
                            };
                            self.expand_spread_argument(spread_value, &mut arg_values)?;
                        }
                        _ => {
                            let arg_signal = self.evaluate_expression_signal(arg)?;
                            let arg_value = match arg_signal {
                                ExecSignal::Value(v) => v,
                                ExecSignal::Call { function_id, args, return_slot } => {
                                    // Вложенный вызов функции в аргументе - возвращаем сигнал Call
                                    // ВАЖНО: НЕ вызываем call_user_function напрямую, чтобы избежать рекурсии Rust
                                    // Главный цикл call_user_function обработает его через единый цикл
                                    if std::env::var("DATACODE_DEBUG").is_ok() {
                                        eprintln!("🔍 DEBUG evaluate_expression_signal: Returning Call signal for function in arg: {}({:?})", function_id, args);
                                    }
                                    // Создаем временный слот для результата
                                    let temp_slot = format!("__arg_{}", self.call_stack.len());
                                    return Ok(ExecSignal::Call {
                                        function_id,
                                        args,
                                        return_slot: return_slot.or(Some(temp_slot)),
                                    });
                                }
                                ExecSignal::Return(_) => {
                                    return Err(DataCodeError::runtime_error(
                                        "Return statement cannot be used in function arguments",
                                        self.current_line
                                    ));
                                }
                            };
                            arg_values.push(arg_value);
                        }
                    }
                }

                // Вычисляем именованные аргументы
                let mut named_arg_values = std::collections::HashMap::new();
                for (arg_name, arg_expr) in named_args {
                    let arg_signal = self.evaluate_expression_signal(&arg_expr)?;
                    let arg_value = match arg_signal {
                        ExecSignal::Value(v) => v,
                        ExecSignal::Call { function_id, args, return_slot } => {
                            // Вложенный вызов функции в именованном аргументе - возвращаем сигнал Call
                            // ВАЖНО: НЕ вызываем call_user_function напрямую, чтобы избежать рекурсии Rust
                            // Главный цикл call_user_function обработает его через единый цикл
                            if std::env::var("DATACODE_DEBUG").is_ok() {
                                eprintln!("🔍 DEBUG evaluate_expression_signal: Returning Call signal for function in named arg: {}({:?})", function_id, args);
                            }
                            // Создаем временный слот для результата
                            let temp_slot = format!("__named_arg_{}", self.call_stack.len());
                            return Ok(ExecSignal::Call {
                                function_id,
                                args,
                                return_slot: return_slot.or(Some(temp_slot)),
                            });
                        }
                        ExecSignal::Return(_) => {
                            return Err(DataCodeError::runtime_error(
                                "Return statement cannot be used in named arguments",
                                self.current_line
                            ));
                        }
                    };
                    named_arg_values.insert(arg_name.clone(), arg_value);
                }

                // Проверяем, является ли это пользовательской функцией
                if self.function_manager.contains_function(name) {
                    if !named_arg_values.is_empty() {
                        return Err(DataCodeError::runtime_error(
                            "User functions do not support named arguments yet",
                            self.current_line
                        ));
                    }

                    // Проверяем количество аргументов
                    let called_function = self.function_manager.get_function(name)
                        .ok_or_else(|| DataCodeError::function_not_found(name, self.current_line))?;
                    
                    if called_function.parameters.len() != arg_values.len() {
                        return Err(DataCodeError::wrong_argument_count(
                            name,
                            called_function.parameters.len(),
                            arg_values.len(),
                            self.current_line,
                        ));
                    }

                    // Создаем временный слот для результата
                    let temp_slot = format!("__temp_result_{}", self.call_stack.len());
                    
                    // Возвращаем сигнал вызова функции вместо выполнения
                    Ok(ExecSignal::Call {
                        function_id: name.clone(),
                        args: arg_values,
                        return_slot: Some(temp_slot),
                    })
                } else {
                    // Встроенная функция - выполняем сразу и возвращаем значение
                    let result = call_builtin_function_with_named_args(
                        name, 
                        arg_values, 
                        named_arg_values, 
                        self.current_line
                    )?;
                    Ok(ExecSignal::Value(result))
                }
            }

            Expr::Binary { left, operator, right } => {
                // Вычисляем левую часть
                let mut left_signal = self.evaluate_expression_signal(left)?;
                
                // Если левая часть - это вызов функции, проверяем кэш
                if let ExecSignal::Call { function_id, args, return_slot: _ } = &left_signal {
                    if let Some(cached_result) = self.function_cache.get(function_id, args) {
                        if std::env::var("DATACODE_DEBUG").is_ok() {
                            eprintln!("🔍 DEBUG evaluate_expression_signal: Cache HIT for {}({:?}) in binary op left, converting to Value", function_id, args);
                        }
                        left_signal = ExecSignal::Value(cached_result);
                    }
                }
                
                let left_val = match left_signal {
                    ExecSignal::Value(v) => v,
                    ExecSignal::Call { function_id, args, return_slot } => {
                        // Вызов функции в левой части - возвращаем сигнал Call
                        // ВАЖНО: НЕ вызываем call_user_function напрямую, чтобы избежать рекурсии Rust
                        if std::env::var("DATACODE_DEBUG").is_ok() {
                            eprintln!("🔍 DEBUG evaluate_expression_signal: Returning Call signal for function in binary op left: {}({:?})", function_id, args);
                        }
                        // Создаем временный слот для результата бинарной операции
                        let temp_slot = format!("__binary_left_{}", self.call_stack.len());
                        // Сохраняем информацию о бинарной операции в контексте
                        // Пока просто возвращаем Call - главный цикл обработает его
                        return Ok(ExecSignal::Call {
                            function_id,
                            args,
                            return_slot: return_slot.or(Some(temp_slot)),
                        });
                    }
                    ExecSignal::Return(_) => {
                        return Err(DataCodeError::runtime_error(
                            "Return statement cannot be used in binary operations",
                            self.current_line
                        ));
                    }
                };

                // Вычисляем правую часть
                let mut right_signal = self.evaluate_expression_signal(right)?;
                
                // Если правая часть - это вызов функции, проверяем кэш
                if let ExecSignal::Call { function_id, args, return_slot: _ } = &right_signal {
                    if let Some(cached_result) = self.function_cache.get(function_id, args) {
                        if std::env::var("DATACODE_DEBUG").is_ok() {
                            eprintln!("🔍 DEBUG evaluate_expression_signal: Cache HIT for {}({:?}) in binary op right, converting to Value", function_id, args);
                        }
                        right_signal = ExecSignal::Value(cached_result);
                    }
                }
                
                let right_val = match right_signal {
                    ExecSignal::Value(v) => v,
                    ExecSignal::Call { function_id, args, return_slot } => {
                        // Вызов функции в правой части - возвращаем сигнал Call
                        // ВАЖНО: НЕ вызываем call_user_function напрямую, чтобы избежать рекурсии Rust
                        if std::env::var("DATACODE_DEBUG").is_ok() {
                            eprintln!("🔍 DEBUG evaluate_expression_signal: Returning Call signal for function in binary op right: {}({:?})", function_id, args);
                        }
                        // Создаем временный слот для результата бинарной операции
                        let temp_slot = format!("__binary_right_{}", self.call_stack.len());
                        // Сохраняем информацию о бинарной операции в контексте
                        // Пока просто возвращаем Call - главный цикл обработает его
                        return Ok(ExecSignal::Call {
                            function_id,
                            args,
                            return_slot: return_slot.or(Some(temp_slot)),
                        });
                    }
                    ExecSignal::Return(_) => {
                        return Err(DataCodeError::runtime_error(
                            "Return statement cannot be used in binary operations",
                            self.current_line
                        ));
                    }
                };

                // Выполняем бинарную операцию
                use crate::parser::tokens::BinaryOp;
                let result = match operator {
                    BinaryOp::Add => self.add_values(&left_val, &right_val)?,
                    BinaryOp::Subtract => self.subtract_values(&left_val, &right_val)?,
                    BinaryOp::Multiply => self.multiply_values(&left_val, &right_val)?,
                    BinaryOp::Divide => self.divide_values(&left_val, &right_val)?,
                    BinaryOp::Modulo => self.modulo_values(&left_val, &right_val)?,
                    BinaryOp::Equal => Value::Bool(self.values_equal(&left_val, &right_val)),
                    BinaryOp::NotEqual => Value::Bool(!self.values_equal(&left_val, &right_val)),
                    BinaryOp::Less => {
                        let less = self.less_than_values(&left_val, &right_val)?;
                        Value::Bool(less.as_bool().unwrap_or(false))
                    }
                    BinaryOp::Greater => {
                        let greater = self.greater_than_values(&left_val, &right_val)?;
                        Value::Bool(greater.as_bool().unwrap_or(false))
                    }
                    BinaryOp::LessEqual => {
                        let less = self.less_than_values(&left_val, &right_val)?;
                        let equal = self.values_equal(&left_val, &right_val);
                        Value::Bool(less.as_bool().unwrap_or(false) || equal)
                    }
                    BinaryOp::GreaterEqual => {
                        let greater = self.greater_than_values(&left_val, &right_val)?;
                        let equal = self.values_equal(&left_val, &right_val);
                        Value::Bool(greater.as_bool().unwrap_or(false) || equal)
                    }
                    BinaryOp::And => {
                        let left_bool = self.to_bool(&left_val);
                        Value::Bool(left_bool && self.to_bool(&right_val))
                    }
                    BinaryOp::Or => {
                        let left_bool = self.to_bool(&left_val);
                        Value::Bool(left_bool || self.to_bool(&right_val))
                    }
                    _ => return Err(DataCodeError::runtime_error(
                        &format!("Unsupported binary operator: {:?}", operator),
                        self.current_line,
                    )),
                };
                Ok(ExecSignal::Value(result))
            }

            Expr::Unary { operator, operand } => {
                let operand_signal = self.evaluate_expression_signal(operand)?;
                let operand_val = match operand_signal {
                    ExecSignal::Value(v) => v,
                    _ => return Err(DataCodeError::runtime_error(
                        "Unary operations require a value",
                        self.current_line
                    )),
                };

                use crate::parser::tokens::UnaryOp;
                let result = match operator {
                    UnaryOp::Not => Value::Bool(!self.to_bool(&operand_val)),
                    UnaryOp::Minus => {
                        if let Value::Number(n) = operand_val {
                            Value::Number(-n)
                        } else {
                            return Err(DataCodeError::runtime_error(
                                "Unary minus requires a number",
                                self.current_line
                            ));
                        }
                    }
                };
                Ok(ExecSignal::Value(result))
            }

            Expr::Index { object, index } => {
                let obj_signal = self.evaluate_expression_signal(object)?;
                let obj_val = match obj_signal {
                    ExecSignal::Value(v) => v,
                    _ => return Err(DataCodeError::runtime_error(
                        "Indexing requires a value",
                        self.current_line
                    )),
                };

                let idx_signal = self.evaluate_expression_signal(index)?;
                let idx_val = match idx_signal {
                    ExecSignal::Value(v) => v,
                    _ => return Err(DataCodeError::runtime_error(
                        "Index must be a value",
                        self.current_line
                    )),
                };

                // Используем существующую логику индексации
                use crate::value::Value;
                let result = match (&obj_val, &idx_val) {
                    (Value::Array(arr), Value::Number(n)) => {
                        let idx = *n as usize;
                        arr.get(idx)
                            .cloned()
                            .ok_or_else(|| DataCodeError::runtime_error(
                                &format!("Array index {} out of bounds", n),
                                self.current_line
                            ))?
                    }
                    (Value::String(s), Value::Number(n)) => {
                        let idx = *n as usize;
                        let chars: Vec<char> = s.chars().collect();
                        if idx >= chars.len() {
                            return Err(DataCodeError::runtime_error(
                                &format!("String index {} out of bounds", n),
                                self.current_line
                            ));
                        }
                        Value::String(chars[idx].to_string())
                    }
                    (Value::Object(obj), Value::String(key)) => {
                        obj.get(key)
                            .cloned()
                            .ok_or_else(|| DataCodeError::runtime_error(
                                &format!("Key '{}' not found in object", key),
                                self.current_line
                            ))?
                    }
                    _ => return Err(DataCodeError::runtime_error(
                        &format!("Cannot index {:?} with {:?}", obj_val, idx_val),
                        self.current_line
                    )),
                };
                Ok(ExecSignal::Value(result))
            }

            Expr::Member { object, member } => {
                let obj_signal = self.evaluate_expression_signal(object)?;
                let obj_val = match obj_signal {
                    ExecSignal::Value(v) => v,
                    _ => return Err(DataCodeError::runtime_error(
                        "Member access requires a value",
                        self.current_line
                    )),
                };

                use crate::value::Value;
                let result = match &obj_val {
                    Value::Object(obj) => {
                        obj.get(member)
                            .cloned()
                            .ok_or_else(|| DataCodeError::runtime_error(
                                &format!("Member '{}' not found in object", member),
                                self.current_line
                            ))?
                    }
                    Value::Array(arr) => {
                        match member.as_str() {
                            "len" => Value::Number(arr.len() as f64),
                            "first" => {
                                if arr.is_empty() {
                                    Value::Null
                                } else {
                                    arr[0].clone()
                                }
                            }
                            "last" => {
                                if arr.is_empty() {
                                    Value::Null
                                } else {
                                    arr[arr.len() - 1].clone()
                                }
                            }
                            _ => return Err(DataCodeError::runtime_error(
                                &format!("Array has no member '{}'", member),
                                self.current_line
                            )),
                        }
                    }
                    Value::String(s) => {
                        match member.as_str() {
                            "len" => Value::Number(s.chars().count() as f64),
                            "upper" => Value::String(s.to_uppercase()),
                            "lower" => Value::String(s.to_lowercase()),
                            "trim" => Value::String(s.trim().to_string()),
                            _ => return Err(DataCodeError::runtime_error(
                                &format!("String has no member '{}'", member),
                                self.current_line
                            )),
                        }
                    }
                    _ => return Err(DataCodeError::runtime_error(
                        &format!("Cannot access member '{}' on {:?}", member, obj_val),
                        self.current_line
                    )),
                };
                Ok(ExecSignal::Value(result))
            }

            Expr::ArrayLiteral { elements } => {
                let mut array_elements = Vec::new();
                for elem in elements {
                    let elem_signal = self.evaluate_expression_signal(elem)?;
                    let elem_val = match elem_signal {
                        ExecSignal::Value(v) => v,
                        _ => return Err(DataCodeError::runtime_error(
                            "Array elements must be values",
                            self.current_line
                        )),
                    };
                    array_elements.push(elem_val);
                }
                Ok(ExecSignal::Value(Value::Array(array_elements)))
            }

            Expr::ObjectLiteral { pairs } => {
                let mut obj = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value_signal = self.evaluate_expression_signal(value_expr)?;
                    let value = match value_signal {
                        ExecSignal::Value(v) => v,
                        _ => return Err(DataCodeError::runtime_error(
                            "Object values must be values",
                            self.current_line
                        )),
                    };
                    obj.insert(key.clone(), value);
                }
                Ok(ExecSignal::Value(Value::Object(obj)))
            }

            _ => {
                // Для остальных типов выражений используем стандартный evaluator
                // но это может вызвать рекурсию, поэтому лучше обработать все случаи выше
                Err(DataCodeError::runtime_error(
                    &format!("Unsupported expression type in signal mode: {:?}", expr),
                    self.current_line
                ))
            }
        }
    }
}

