use crate::value::Value;
use crate::error::{DataCodeError, Result};
use super::Interpreter;
use super::execution::execute_line;

/// Обработчик управляющих конструкций
#[allow(dead_code)]
pub struct ControlFlowHandler;

impl ControlFlowHandler {
    /// Выполнить условную конструкцию if/else
    pub fn execute_if(
        interpreter: &mut Interpreter,
        condition: &crate::parser::Expr,
        then_body: &[String],
        else_body: &Option<Vec<String>>,
    ) -> Result<()> {
        let condition_value = interpreter.evaluate_expression(condition)?;
        let condition_bool = Self::to_bool(&condition_value);

        if condition_bool {
            Self::execute_block(interpreter, then_body)?;
        } else if let Some(else_lines) = else_body {
            Self::execute_block(interpreter, else_lines)?;
        }

        Ok(())
    }

    /// Выполнить цикл for
    pub fn execute_for(
        interpreter: &mut Interpreter,
        variables: &[String],
        iterable: &crate::parser::Expr,
        body: &[String],
    ) -> Result<()> {
        let iterable_value = interpreter.evaluate_expression(iterable)?;
        
        // НЕ создаём scope здесь - он будет создаваться для каждой итерации

        let result = match iterable_value {
            Value::Array(arr) => Self::iterate_over_array(interpreter, variables, &arr, body),
            Value::Table(table) => {
                let table_borrowed = table.borrow();
                Self::iterate_over_table(interpreter, variables, &*table_borrowed, body)
            },
            Value::String(s) => Self::iterate_over_string(interpreter, variables, &s, body),
            Value::Object(obj) => Self::iterate_over_object(interpreter, variables, &obj, body),
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot iterate over {:?}", iterable_value),
                interpreter.current_line,
            )),
        };

        result
    }

    /// Выполнить блок try/catch/finally
    pub fn execute_try(
        interpreter: &mut Interpreter,
        body: &[String],
        catch_var: &Option<String>,
        catch_body: &Option<Vec<String>>,
        finally_body: &Option<Vec<String>>,
    ) -> Result<()> {
        // Создаем блок try/catch
        let block_id = interpreter.get_next_try_block_id();
        let nesting_level = interpreter.get_try_nesting_level();

        let try_block = super::user_functions::TryBlock::new(
            catch_var.clone(),
            catch_body.clone().unwrap_or_default(),
            finally_body.clone(),
            block_id,
            nesting_level,
        );
        
        interpreter.exception_stack.push(try_block);

        // Создаем область видимости для try-catch блока (для локальных переменных)
        // Это нужно сделать ДО выполнения try блока, чтобы локальные переменные из try попадали в эту область
        interpreter.enter_loop_scope();

        // Выполняем основное тело
        let mut try_result = Ok(());
        for line in body {
            if let Err(e) = execute_line(interpreter, line) {
                try_result = Err(e);
                break;
            }
            // Проверяем return
            if interpreter.return_value.is_some() {
                break;
            }
        }

        // Убираем блок из стека
        let try_block = interpreter.exception_stack.pop().unwrap();

        // Если была ошибка, выполняем catch
        if let Err(error) = try_result {
            if !try_block._catch_body.is_empty() {
                // Устанавливаем переменную ошибки, если указана
                // Теперь она попадет в loop_stack, так как мы уже вошли в область видимости
                if let Some(var_name) = &try_block._catch_var {
                    let error_message = format!("{}", error);
                    interpreter.set_variable(
                        var_name.clone(),
                        Value::String(error_message),
                        false,
                    );
                }

                // Выполняем catch блок
                Self::execute_block(interpreter, &try_block._catch_body)?;
            }
        }

        // Выполняем finally блок, если есть
        if let Some(finally_lines) = &try_block._finally_body {
            Self::execute_block(interpreter, finally_lines)?;
        }

        // Выходим из области видимости try-catch блока (очищаем локальные переменные)
        // Это удалит переменную e и другие локальные переменные из catch блока
        interpreter.exit_loop_scope();

        Ok(())
    }

    /// Выполнить блок кода
    fn execute_block(interpreter: &mut Interpreter, lines: &[String]) -> Result<()> {
        for line in lines {
            execute_line(interpreter, line)?;
            // Проверяем return
            if interpreter.return_value.is_some() {
                break;
            }
        }
        Ok(())
    }

    /// Итерация по массиву
    fn iterate_over_array(
        interpreter: &mut Interpreter,
        variables: &[String],
        array: &[Value],
        body: &[String],
    ) -> Result<()> {
        if variables.len() > 1 {
            // Множественные переменные - деструктуризация элементов массива
            for item in array {
                // Создаём новый scope для этой итерации
                interpreter.enter_loop_scope();
                
                // Проверяем, является ли элемент массивом для деструктуризации
                match item {
                    Value::Array(ref item_arr) => {
                        // Деструктурируем массив в переменные
                        if item_arr.len() != variables.len() {
                            interpreter.exit_loop_scope();
                            return Err(DataCodeError::runtime_error(
                                &format!("Cannot unpack array of length {} into {} variables", item_arr.len(), variables.len()),
                                interpreter.current_line,
                            ));
                        }
                        
                        // Устанавливаем все переменные из массива
                        for (i, var_name) in variables.iter().enumerate() {
                            interpreter.set_loop_variable(var_name.to_string(), item_arr[i].clone());
                        }
                    }
                    _ => {
                        // Элемент не массив - ошибка
                        interpreter.exit_loop_scope();
                        return Err(DataCodeError::runtime_error(
                            &format!("Cannot unpack non-array value into {} variables", variables.len()),
                            interpreter.current_line,
                        ));
                    }
                }
                
                Self::execute_block(interpreter, body)?;
                
                // Удаляем scope этой итерации
                interpreter.exit_loop_scope();
                
                // Если был return, выходим из цикла
                if interpreter.return_value.is_some() {
                    break;
                }
            }
        } else {
            // Одна переменная - обычная итерация
            for item in array {
                // Создаём новый scope для этой итерации
                interpreter.enter_loop_scope();
                
                interpreter.set_loop_variable(variables[0].to_string(), item.clone());
                
                Self::execute_block(interpreter, body)?;
                
                // Удаляем scope этой итерации
                interpreter.exit_loop_scope();
                
                // Если был return, выходим из цикла
                if interpreter.return_value.is_some() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Итерация по таблице
    fn iterate_over_table(
        interpreter: &mut Interpreter,
        variables: &[String],
        table: &crate::value::Table,
        body: &[String],
    ) -> Result<()> {
        for row in &table.rows {
            // Создаём новый scope для этой итерации
            interpreter.enter_loop_scope();
            
            if variables.len() > 1 {
                // Деструктуризация строки таблицы в переменные
                if row.len() != variables.len() {
                    interpreter.exit_loop_scope();
                    return Err(DataCodeError::runtime_error(
                        &format!("Cannot unpack table row of length {} into {} variables", row.len(), variables.len()),
                        interpreter.current_line,
                    ));
                }
                
                // Устанавливаем все переменные из строки
                for (i, var_name) in variables.iter().enumerate() {
                    interpreter.set_loop_variable(var_name.to_string(), row[i].clone());
                }
            } else {
                // Одна переменная - присваиваем весь массив строки
                interpreter.set_loop_variable(variables[0].to_string(), Value::Array(row.clone()));
            }
            
            Self::execute_block(interpreter, body)?;
            
            // Удаляем scope этой итерации
            interpreter.exit_loop_scope();
            
            // Если был return, выходим из цикла
            if interpreter.return_value.is_some() {
                break;
            }
        }
        Ok(())
    }

    /// Итерация по строке (по символам)
    fn iterate_over_string(
        interpreter: &mut Interpreter,
        variables: &[String],
        string: &str,
        body: &[String],
    ) -> Result<()> {
        if variables.len() > 1 {
            return Err(DataCodeError::runtime_error(
                &format!("String iteration supports only 1 variable, got {}", variables.len()),
                interpreter.current_line,
            ));
        }
        
        for ch in string.chars() {
            // Создаём новый scope для этой итерации
            interpreter.enter_loop_scope();
            
            interpreter.set_loop_variable(
                variables[0].to_string(),
                Value::String(ch.to_string()),
            );

            Self::execute_block(interpreter, body)?;

            // Удаляем scope этой итерации
            interpreter.exit_loop_scope();

            // Если был return, выходим из цикла
            if interpreter.return_value.is_some() {
                break;
            }
        }
        Ok(())
    }

    /// Итерация по объекту (по парам ключ-значение)
    fn iterate_over_object(
        interpreter: &mut Interpreter,
        variables: &[String],
        object: &std::collections::HashMap<String, Value>,
        body: &[String],
    ) -> Result<()> {
        // Сортируем ключи для предсказуемого порядка итерации
        let mut keys: Vec<_> = object.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(value) = object.get(key) {
                // Создаём новый scope для этой итерации
                interpreter.enter_loop_scope();
                
                if variables.len() == 1 {
                    // Простое присваивание - создаем массив [ключ, значение]
                    let key_value_pair = Value::Array(vec![
                        Value::String(key.clone()),
                        value.clone(),
                    ]);
                    interpreter.set_loop_variable(variables[0].to_string(), key_value_pair);
                } else if variables.len() == 2 {
                    // Деструктуризация на ключ и значение
                    interpreter.set_loop_variable(variables[0].to_string(), Value::String(key.clone()));
                    interpreter.set_loop_variable(variables[1].to_string(), value.clone());
                } else {
                    interpreter.exit_loop_scope();
                    return Err(DataCodeError::runtime_error(
                        &format!("Object iteration supports 1 or 2 variables, got {}", variables.len()),
                        interpreter.current_line,
                    ));
                }

                Self::execute_block(interpreter, body)?;

                // Удаляем scope этой итерации
                interpreter.exit_loop_scope();

                // Если был return, выходим из цикла
                if interpreter.return_value.is_some() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Обработка вложенных условий
    pub fn execute_nested_if(
        interpreter: &mut Interpreter,
        conditions: &[(crate::parser::Expr, Vec<String>)],
        else_body: &Option<Vec<String>>,
    ) -> Result<()> {
        for (condition, body) in conditions {
            let condition_value = interpreter.evaluate_expression(condition)?;
            if Self::to_bool(&condition_value) {
                Self::execute_block(interpreter, body)?;
                return Ok(());
            }
        }

        // Если ни одно условие не выполнилось, выполняем else
        if let Some(else_lines) = else_body {
            Self::execute_block(interpreter, else_lines)?;
        }

        Ok(())
    }

    /// Обработка циклов с условием while
    pub fn execute_while(
        interpreter: &mut Interpreter,
        condition: &crate::parser::Expr,
        body: &[String],
    ) -> Result<()> {
        interpreter.enter_loop_scope();

        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 1_000_000; // Защита от бесконечных циклов

        loop {
            // Проверяем условие
            let condition_value = interpreter.evaluate_expression(condition)?;
            if !Self::to_bool(&condition_value) {
                break;
            }

            // Защита от бесконечных циклов
            iteration_count += 1;
            if iteration_count > MAX_ITERATIONS {
                interpreter.exit_loop_scope();
                return Err(DataCodeError::runtime_error(
                    "Infinite loop detected (exceeded maximum iterations)",
                    interpreter.current_line,
                ));
            }

            // Выполняем тело цикла
            Self::execute_block(interpreter, body)?;

            // Если был return, выходим из цикла
            if interpreter.return_value.is_some() {
                break;
            }
        }

        interpreter.exit_loop_scope();
        Ok(())
    }

    /// Преобразовать значение в булево
    fn to_bool(value: &Value) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_to_bool() {
        assert_eq!(ControlFlowHandler::to_bool(&Value::Bool(true)), true);
        assert_eq!(ControlFlowHandler::to_bool(&Value::Bool(false)), false);
        assert_eq!(ControlFlowHandler::to_bool(&Value::Number(1.0)), true);
        assert_eq!(ControlFlowHandler::to_bool(&Value::Number(0.0)), false);
        assert_eq!(ControlFlowHandler::to_bool(&Value::Null), false);
    }

    #[test]
    fn test_execute_if_true_condition() {
        let mut interp = Interpreter::new();
        
        // Создаем условие true
        let mut parser = Parser::new("true");
        let condition = parser.parse_expression().unwrap();
        
        let then_body = vec!["global x = 42".to_string()];
        let else_body = None;
        
        let result = ControlFlowHandler::execute_if(&mut interp, &condition, &then_body, &else_body);
        assert!(result.is_ok());
        assert_eq!(interp.get_variable("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_execute_if_false_condition() {
        let mut interp = Interpreter::new();
        
        // Создаем условие false
        let mut parser = Parser::new("false");
        let condition = parser.parse_expression().unwrap();
        
        let then_body = vec!["global x = 42".to_string()];
        let else_body = Some(vec!["global x = 24".to_string()]);
        
        let result = ControlFlowHandler::execute_if(&mut interp, &condition, &then_body, &else_body);
        assert!(result.is_ok());
        assert_eq!(interp.get_variable("x"), Some(&Value::Number(24.0)));
    }
}
