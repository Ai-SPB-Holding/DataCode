// Выполнение одной инструкции с возвратом сигнала
// Используется в едином событийном цикле без рекурсии Rust

use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::parser::Parser;
use super::{Interpreter, ExecSignal};

impl Interpreter {
    /// Выполнить одну инструкцию и вернуть сигнал
    /// НЕ выполняет функции напрямую, а возвращает сигнал для главного цикла
    pub fn execute_instruction_signal(&mut self, instruction: &str) -> Result<ExecSignal> {
        let trimmed = instruction.trim();
        if std::env::var("DATACODE_DEBUG").is_ok() {
            eprintln!("🔍 DEBUG execute_instruction_signal: ENTRY with instruction: '{}' (trimmed: '{}')", instruction, trimmed);
        }
        
        // Пропускаем пустые строки и комментарии
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(ExecSignal::Value(Value::Null));
        }

        // Пропускаем ключевые слова блочных конструкций
        // Они обрабатываются на уровне выше через execute_block_directly
        if trimmed.starts_with("function ") || trimmed.starts_with("global function ") || trimmed.starts_with("local function ") ||
           trimmed == "endfunction" || trimmed.starts_with("if ") || trimmed == "else" || trimmed == "endif" || trimmed == "endeif" ||
           trimmed.starts_with("for ") || trimmed.starts_with("next ") || trimmed == "try" || trimmed == "catch" ||
           trimmed == "finally" || trimmed == "endtry" {
            // Это блочная конструкция - возвращаем Value::Null, так как она обрабатывается на уровне выше
            return Ok(ExecSignal::Value(Value::Null));
        }

        // Обработка return
        if trimmed.starts_with("return") {
            let after_return = trimmed.strip_prefix("return").unwrap().trim();
            if after_return.is_empty() {
                return Ok(ExecSignal::Return(Value::Null));
            }
            
            // Всегда используем evaluate_expression_signal для вычисления выражения
            // Это правильно обработает вызовы функций через единый цикл Call Frame Engine
            let mut parser = Parser::new(after_return);
            let expr = parser.parse_expression()?;
            let mut signal = self.evaluate_expression_signal(&expr)?;
            
            // Если сигнал - это ExecSignal::Call с результатом в кэше, заменяем на ExecSignal::Value
            // Это нужно для продолжения вычисления бинарных операций
            if let ExecSignal::Call { function_id, args, return_slot: _ } = &signal {
                if let Some(cached_result) = self.function_cache.get(function_id, args) {
                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG execute_instruction_signal: Cache HIT for {}({:?}) in return, converting to Value", function_id, args);
                    }
                    // Заменяем ExecSignal::Call на ExecSignal::Value с результатом из кэша
                    signal = ExecSignal::Value(cached_result);
                }
            }
            
            match signal {
                ExecSignal::Value(v) => Ok(ExecSignal::Return(v)),
                ExecSignal::Call { function_id, ref args, return_slot } => {
                    // Вызов функции в return - возвращаем сигнал Call
                    // Главный цикл call_user_function обработает его правильно через единый цикл
                    // НЕ вызываем call_user_function напрямую, чтобы избежать рекурсии Rust
                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG execute_instruction_signal: Returning Call signal for function in return: {}({:?})", function_id, args);
                    }
                    // Возвращаем сигнал Call - главный цикл обработает его
                    Ok(ExecSignal::Call {
                        function_id,
                        args: args.clone(),
                        return_slot: return_slot.clone(),
                    })
                }
                ExecSignal::Return(_) => {
                    Err(DataCodeError::runtime_error(
                        "Nested return statement",
                        self.current_line
                    ))
                }
            }
        }
        // Обработка print - ДОЛЖНА быть ПЕРЕД обработкой присваивания
        // чтобы избежать ложных срабатываний когда строка содержит '=' внутри аргументов print
        else if trimmed.starts_with("print(") {
            // Парсим аргументы print
            if let Some(args_str) = trimmed.strip_prefix("print(") {
                if let Some(close_paren_pos) = args_str.rfind(')') {
                    let args_content = &args_str[..close_paren_pos];
                    
                    // Парсим аргументы как выражение (может быть несколько через запятую)
                    let mut parser = Parser::new(args_content);
                    let expr = parser.parse_expression()?;
                    let signal = self.evaluate_expression_signal(&expr)?;
                    
                    match signal {
                        ExecSignal::Value(v) => {
                            // Выводим значение
                            println!("{}", self.format_value_for_print(&v));
                            Ok(ExecSignal::Value(Value::Null))
                        }
                        ExecSignal::Call { .. } => {
                            // Вызов функции в print - возвращаем Call
                            Ok(signal)
                        }
                        ExecSignal::Return(_) => {
                            Err(DataCodeError::runtime_error(
                                "Return statement cannot be used in print",
                                self.current_line
                            ))
                        }
                    }
                } else {
                    Err(DataCodeError::syntax_error(
                        "Missing closing parenthesis in print",
                        self.current_line,
                        0
                    ))
                }
            } else {
                Err(DataCodeError::syntax_error(
                    "Invalid print statement",
                    self.current_line,
                    0
                ))
            }
        }
        // Обработка присваивания
        // ВАЖНО: Проверяем, что это присваивание (содержит '='), но не оператор сравнения
        // Операторы сравнения: ==, !=, <=, >=
        // Но нужно быть осторожным: правая часть присваивания может содержать !=, <=, >=
        // Поэтому проверяем, что '=' не является частью оператора сравнения
        else if trimmed.contains('=') {
            // Проверяем, что '=' не является частью оператора сравнения
            // Ищем первое вхождение '=' и проверяем, что перед ним нет операторов сравнения
            let first_eq_pos = trimmed.find('=').unwrap_or(0);
            let before_eq = &trimmed[..first_eq_pos];
            
            // Проверяем, что перед '=' нет операторов сравнения
            let is_comparison_op = before_eq.ends_with('=') || before_eq.ends_with('!') 
                || before_eq.ends_with('<') || before_eq.ends_with('>');
            
            if !is_comparison_op {
                if std::env::var("DATACODE_DEBUG").is_ok() {
                    eprintln!("🔍 DEBUG execute_instruction_signal: Entering assignment block for: '{}'", trimmed);
                }
            // Проверяем наличие префикса global/local перед обработкой присваивания
            let (var_name, expr_str, is_global) = if trimmed.starts_with("global ") {
                // Извлекаем код после префикса "global " (7 символов)
                let code = &trimmed[7..];
                let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string(), true)
                } else {
                    return Err(DataCodeError::syntax_error(
                        "Invalid assignment",
                        self.current_line,
                        0
                    ));
                }
            } else if trimmed.starts_with("local ") {
                // Извлекаем код после префикса "local " (6 символов)
                let code = &trimmed[6..];
                let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();
                if std::env::var("DATACODE_DEBUG").is_ok() {
                    eprintln!("🔍 DEBUG execute_instruction_signal: Processing 'local' assignment: '{}'", trimmed);
                    eprintln!("   Code after prefix: '{}'", code);
                    eprintln!("   Parts: {:?}", parts);
                }
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string(), false)
                } else {
                    return Err(DataCodeError::syntax_error(
                        "Invalid assignment",
                        self.current_line,
                        0
                    ));
                }
            } else {
                // Обычное присваивание без префикса
                let parts: Vec<_> = trimmed.splitn(2, '=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string(), false)
                } else {
                    return Ok(ExecSignal::Value(Value::Null));
                }
            };
            
            // Проверяем, что имя переменной - это простой идентификатор
            // Отладка: выводим имя переменной для диагностики
            if std::env::var("DATACODE_DEBUG_ASSIGN").is_ok() {
                eprintln!("🔍 DEBUG: var_name = '{}', expr_str = '{}'", var_name, expr_str);
                eprintln!("🔍 DEBUG: var_name.chars(): {:?}", var_name.chars().collect::<Vec<_>>());
            }
            if var_name.chars().all(|c| c.is_alphanumeric() || c == '_') && !var_name.is_empty() {
                // Парсим и вычисляем выражение
                if std::env::var("DATACODE_DEBUG").is_ok() {
                    eprintln!("🔍 DEBUG execute_instruction_signal: About to create parser with expr_str: '{}'", expr_str);
                }
                let mut parser = Parser::new(&expr_str);
                let expr = parser.parse_expression()?;
                let signal = self.evaluate_expression_signal(&expr)?;
                
                match signal {
                    ExecSignal::Value(val) => {
                        // Сохраняем переменную в текущем фрейме
                        if let Some(frame) = self.call_stack.last_mut() {
                            frame.set_local(var_name.to_string(), val.clone());
                        }
                        // Также синхронизируем с variable_manager
                        if is_global {
                            self.set_variable(var_name.to_string(), val, true);
                        } else {
                            // Для local явно устанавливаем как локальную
                            // Для обычного присваивания используем умное определение области видимости
                            if trimmed.starts_with("local ") {
                                // Явное local присваивание
                                self.set_variable(var_name.to_string(), val, false);
                            } else {
                                // Обычное присваивание без префикса
                                self.set_variable_smart(var_name.to_string(), val);
                            }
                        }
                        Ok(ExecSignal::Value(Value::Null))
                    }
                    ExecSignal::Call { function_id, args, return_slot: _ } => {
                        // Вызов функции в присваивании - создаем return_slot с именем переменной
                        let temp_slot = format!("__assign_{}_{}", var_name, self.call_stack.len());
                        Ok(ExecSignal::Call {
                            function_id,
                            args,
                            return_slot: Some(temp_slot),
                        })
                    }
                    ExecSignal::Return(_) => {
                        Err(DataCodeError::runtime_error(
                            "Return statement cannot be used in assignment",
                            self.current_line
                        ))
                    }
                }
            } else {
                // Сложное присваивание - используем стандартный механизм
                // Но это может вызвать рекурсию, поэтому лучше обработать все случаи выше
                Err(DataCodeError::runtime_error(
                    "Complex assignment not yet supported in signal mode",
                    self.current_line
                ))
            }
            } else {
                // Это оператор сравнения, а не присваивание - обрабатываем как выражение
                // Но сначала проверяем, что это не начинается с 'local' или 'global'
                if trimmed.starts_with("local ") || trimmed.starts_with("global ") {
                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("❌ DEBUG execute_instruction_signal: CRITICAL - Attempting to parse 'local' or 'global' statement as expression!");
                        eprintln!("   This should have been handled as an assignment. Instruction: '{}'", trimmed);
                    }
                    return Err(DataCodeError::syntax_error(
                        &format!("Cannot parse '{}' as expression. Statements starting with 'local' or 'global' must be handled as assignments, not expressions.", trimmed),
                        self.current_line, 0
                    ));
                }
                // Парсим как выражение
                let mut parser = Parser::new(trimmed);
                let expr = parser.parse_expression()?;
                self.evaluate_expression_signal(&expr)
            }
        }
        // Обработка выражений без присваивания
        else {
            // ВАЖНО: Проверяем на function ПЕРЕД парсингом
            if trimmed.starts_with("function ") || trimmed.starts_with("global function ") || trimmed.starts_with("local function ") {
                // Это блочная конструкция - возвращаем Value::Null
                return Ok(ExecSignal::Value(Value::Null));
            }
            
            // КРИТИЧЕСКАЯ ПРОВЕРКА: не парсим строки, начинающиеся с 'local' или 'global' как выражения
            // Они должны были быть обработаны как присваивания выше
            if trimmed.starts_with("local ") || trimmed.starts_with("global ") {
                if std::env::var("DATACODE_DEBUG").is_ok() {
                    eprintln!("❌ DEBUG execute_instruction_signal: CRITICAL - Attempting to parse 'local' or 'global' statement as expression!");
                    eprintln!("   This should have been handled as an assignment. Instruction: '{}'", trimmed);
                }
                return Err(DataCodeError::syntax_error(
                    &format!("Cannot parse '{}' as expression. Statements starting with 'local' or 'global' must be handled as assignments, not expressions.", trimmed),
                    self.current_line, 0
                ));
            }
            
            // Парсим как выражение
            let mut parser = Parser::new(trimmed);
            let expr = parser.parse_expression()?;
            self.evaluate_expression_signal(&expr)
        }
    }
    
    /// Форматировать значение для вывода
    fn format_value_for_print(&self, value: &Value) -> String {
        use crate::value::Value;
        match value {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.format_value_for_print(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_value_for_print(v)))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            _ => format!("{:?}", value),
        }
    }
}

