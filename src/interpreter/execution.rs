use crate::value::Value;
use crate::error::{DataCodeError, Result};
use super::Interpreter;

/// Выполнить строку кода
pub fn execute_line(interpreter: &mut Interpreter, line: &str) -> Result<()> {
    // Если код содержит несколько строк, обрабатываем построчно
    if line.contains('\n') {
        return execute_multiline(interpreter, line);
    }

    let trimmed = line.trim();

    // Пропускаем пустые строки и комментарии (только для однострочного кода)
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(());
    }

    // Используем старую логику выполнения из оригинального интерпретатора
    execute_line_simple(interpreter, line)
}

/// Выполнить многострочный код
pub fn execute_multiline(interpreter: &mut Interpreter, code: &str) -> Result<()> {
    let lines: Vec<&str> = code.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Пропускаем пустые строки и однострочные комментарии
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Увеличиваем номер строки только для непустых строк
        interpreter.current_line += 1;

        // Обрабатываем многострочные комментарии """
        if line.starts_with("\"\"\"") {
            // Ищем закрывающий """
            let mut j = i;
            let mut found_end = false;

            // Проверяем, есть ли закрывающий """ на той же строке
            if line.len() > 3 && line.ends_with("\"\"\"") && line != "\"\"\"" {
                // Комментарий на одной строке
                i += 1;
                continue;
            }

            // Ищем закрывающий """ на следующих строках
            j += 1;
            while j < lines.len() {
                let comment_line = lines[j].trim();
                if comment_line.ends_with("\"\"\"") {
                    found_end = true;
                    break;
                }
                j += 1;
            }

            if found_end {
                // Пропускаем все строки комментария
                i = j + 1;
                continue;
            } else {
                // Незакрытый многострочный комментарий - ошибка
                return Err(DataCodeError::syntax_error("Unclosed multiline comment", interpreter.current_line, 0));
            }
        }

        // Обрабатываем многострочные конструкции
        // Проверяем function с учетом возможных пробелов в начале (хотя line уже trimmed)
        // ВАЖНО: Проверяем function ПЕРВЫМ делом, ДО всех остальных проверок
        if line.starts_with("function ") || line.starts_with("global function ") || line.starts_with("local function ") {
            // Отладка
            if std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                eprintln!("🔍 DEBUG execute_multiline: Handling function definition: '{}'", line);
            }
            i = handle_function_definition(interpreter, &lines, i)?;
            continue; // Продолжаем со следующей строки после обработки функции
        } else if line.starts_with("for ") && line.ends_with(" do") {
            i = handle_for_loop(interpreter, &lines, i)?;
        } else if line.starts_with("if ") && (line.contains(" do") || line.contains(" then")) {
            i = handle_if_statement(interpreter, &lines, i)?;
        } else if line == "try" {
            i = handle_try_statement(interpreter, &lines, i)?;
        } else if is_incomplete_assignment(line) {
            // Обрабатываем многострочные присваивания
            i = handle_multiline_assignment(interpreter, &lines, i)?;
        } else {
            // Обычная строка - используем execute_line_simple чтобы избежать рекурсии
            // Но сначала проверяем, не является ли это function (на случай, если проверка выше не сработала)
            let line_to_exec = lines[i].trim();
            if line_to_exec.starts_with("function ") || line_to_exec.starts_with("global function ") || line_to_exec.starts_with("local function ") {
                // Это должно было быть обработано выше, но на всякий случай пропускаем
                i += 1;
                continue;
            }
            execute_line_simple(interpreter, lines[i])?;
        }

        // Проверяем, был ли выполнен return
        if interpreter.return_value.is_some() {
            break;
        }

        i += 1;
    }
    Ok(())
}

/// Простое выполнение строки кода (без AST)
fn execute_line_simple(interpreter: &mut Interpreter, code: &str) -> Result<()> {
    let trimmed_code = code.trim();

    // Пропускаем пустые строки и комментарии
    if trimmed_code.is_empty() || trimmed_code.starts_with('#') {
        return Ok(());
    }

    // Игнорируем next, так как он уже обрабатывается на уровне парсера циклов
    // Это нужно делать в самом начале, чтобы избежать ошибок
    if trimmed_code.starts_with("next ") {
        return Ok(());  // Игнорируем next, он уже обработан парсером циклов
    }

    // Обработка break (должна быть раньше парсинга выражений)
    if trimmed_code == "break" {
        interpreter.break_requested = true;
        return Ok(());
    }

    // ВАЖНО: Проверяем на блочные конструкции ПЕРВЫМ делом, ДО всех остальных проверок
    // Это нужно делать ДО попытки парсить выражения, чтобы избежать ошибок парсера
    // Также обрабатываем любую строку, начинающуюся с "if", "for" или "function", чтобы избежать попытки парсить её как выражение
    // Для if, for и function просто возвращаем Ok(()), так как они должны обрабатываться execute_multiline или execute_block_directly
    if trimmed_code.starts_with("function ") || trimmed_code.starts_with("global function ") || trimmed_code.starts_with("local function ") {
        // Это должно было быть обработано в execute_multiline, но на всякий случай пропускаем здесь
        return Ok(());  // Эти конструкции обрабатываются execute_multiline
    }
    if trimmed_code.starts_with("if ") || (trimmed_code.starts_with("for ") && trimmed_code.ends_with(" do")) {
        return Ok(());  // Эти конструкции обрабатываются execute_block_directly
    }
    if trimmed_code == "try" || trimmed_code == "catch" || trimmed_code == "finally" ||
       trimmed_code == "endtry" || trimmed_code == "else" || trimmed_code == "endif" || trimmed_code == "endeif" ||
       trimmed_code == "endfunction" {
        // Эти ключевые слова обрабатываются на уровне выше через execute_multiline или execute_block_directly
        // Просто возвращаем Ok(()), чтобы не вызывать ошибку
        return Ok(());
    }

    // Обработка throw statements
    if trimmed_code.starts_with("throw ") {
        return handle_throw_statement(interpreter, trimmed_code);
    }

    // Обработка print statement (должна быть до return и присваивания)
    if trimmed_code.starts_with("print(") {
        // Извлекаем аргументы из print(...)
        if let Some(args_str) = trimmed_code.strip_prefix("print(") {
            if let Some(close_paren_pos) = args_str.rfind(')') {
                let args_content = &args_str[..close_paren_pos];
                
                // Парсим аргументы (разделенные запятыми)
                let args: Vec<Value> = if args_content.trim().is_empty() {
                    Vec::new()
                } else {
                    // Разделяем по запятым, но учитываем вложенные скобки и кавычки
                    let mut args_list = Vec::new();
                    let mut current_arg = String::new();
                    let mut depth = 0;
                    let mut in_string = false;
                    let mut string_char: Option<char> = None;
                    
                    for ch in args_content.chars() {
                        match ch {
                            '\'' | '"' if !in_string => {
                                // Начало строки
                                in_string = true;
                                string_char = Some(ch);
                                current_arg.push(ch);
                            }
                            ch if in_string && Some(ch) == string_char => {
                                // Конец строки
                                in_string = false;
                                string_char = None;
                                current_arg.push(ch);
                            }
                            '(' | '[' | '{' if !in_string => {
                                depth += 1;
                                current_arg.push(ch);
                            }
                            ')' | ']' | '}' if !in_string => {
                                depth -= 1;
                                current_arg.push(ch);
                            }
                            ',' if depth == 0 && !in_string => {
                                if !current_arg.trim().is_empty() {
                                    args_list.push(current_arg.trim().to_string());
                                }
                                current_arg.clear();
                            }
                            _ => {
                                current_arg.push(ch);
                            }
                        }
                    }
                    
                    if !current_arg.trim().is_empty() {
                        args_list.push(current_arg.trim().to_string());
                    }
                    
                    // Вычисляем каждый аргумент
                    // Проверяем, что аргументы не пустые перед парсингом
                    if args_list.is_empty() {
                        return Err(DataCodeError::syntax_error(
                            "print() requires at least one argument",
                            interpreter.current_line, 0
                        ));
                    }
                    
                    // Отладка: выводим аргументы перед парсингом
                    // if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                    //     eprintln!("🔍 DEBUG print: Parsing print arguments at line {}: {:?}", interpreter.current_line, args_list);
                    // }
                    
                    let evaluated_args: Result<Vec<_>> = args_list.into_iter()
                        .enumerate()
                        .map(|(idx, arg)| {
                            if arg.trim().is_empty() {
                                Err(DataCodeError::syntax_error(
                                    "Empty argument in print()",
                                    interpreter.current_line, 0
                                ))
                            } else {
                                // if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                                //     eprintln!("🔍 DEBUG print: Evaluating argument {}: '{}'", idx, arg);
                                // }
                                let result = interpreter.eval_expr(&arg);
                                // if let Ok(ref value) = result {
                                //     if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                                //         eprintln!("🔍 DEBUG print: Argument {} evaluated to: {:?}", idx, value);
                                //     }
                                // } else if let Err(ref e) = result {
                                //     if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                                //         eprintln!("❌ DEBUG print: Error evaluating argument {} '{}': {}", idx, arg, e);
                                //     }
                                // }
                                result
                            }
                        })
                        .collect();
                    
                    let evaluated = evaluated_args?;
                    
                    // if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                    //     eprintln!("🔍 DEBUG print: All arguments evaluated, count: {}", evaluated.len());
                    // }
                    
                    evaluated
                };
                
                // Вызываем встроенную функцию print
                use crate::builtins::system::call_system_function;
                call_system_function("print", args, interpreter.current_line)?;
                return Ok(());
            }
        }
    }

    // Обработка return
    if trimmed_code.starts_with("return") {
        // Проверяем, что мы внутри функции
        if interpreter.variable_manager.call_stack.is_empty() && interpreter.call_stack.is_empty() {
            return Err(DataCodeError::syntax_error(
                "return statement must be inside a function",
                interpreter.current_line,
                0
            ));
        }
        let after_return = trimmed_code.strip_prefix("return").unwrap().trim();
        let value = if after_return.is_empty() {
            Value::Null
        } else {
            interpreter.eval_expr(after_return)?
        };
        interpreter.return_value = Some(value);
        return Ok(());
    }

    // Обработка присваивания переменных
    if trimmed_code.starts_with("global ") || trimmed_code.starts_with("local ") {
        let is_global = trimmed_code.starts_with("global ");
        let code = &trimmed_code[if is_global { 7 } else { 6 }..];
        let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();

        // eprintln!("🔍 DEBUG execute_line_simple: Processing global/local assignment: '{}'", trimmed_code);
        // eprintln!("🔍 DEBUG execute_line_simple: Code after prefix: '{}'", code);
        // eprintln!("🔍 DEBUG execute_line_simple: Parts: {:?}", parts);

        if parts.len() != 2 {
            return Err(DataCodeError::syntax_error("Invalid assignment", interpreter.current_line, 0));
        }

        let var_name = parts[0].to_string();
        let expr = parts[1];

        // eprintln!("🔍 DEBUG execute_line_simple: var_name='{}', expr='{}'", var_name, expr);
        // eprintln!("🔍 DEBUG execute_line_simple: About to call eval_expr with: '{}'", expr);
        let val = interpreter.eval_expr(expr).map_err(|e| {
            // eprintln!("❌ DEBUG execute_line_simple: Error in eval_expr for '{}': {}", expr, e);
            e
        })?;
        interpreter.set_variable(var_name, val, is_global);
        return Ok(());
    }

    // Обработка присваивания без явного global/local префикса
    if trimmed_code.contains('=') && !trimmed_code.contains("==") && !trimmed_code.contains("!=") && !trimmed_code.contains("<=") && !trimmed_code.contains(">=") {
        let parts: Vec<_> = trimmed_code.splitn(2, '=').map(|s| s.trim()).collect();

        if parts.len() == 2 {
            let var_name = parts[0];
            let expr = parts[1];

            // eprintln!("🔍 DEBUG execute_line_simple: Processing assignment: var_name='{}', expr='{}'", var_name, expr);
            // eprintln!("🔍 DEBUG execute_line_simple: Full line being processed: '{}'", trimmed_code);

            // Проверяем, что левая часть - это простой идентификатор (не выражение)
            if var_name.chars().all(|c| c.is_alphanumeric() || c == '_') && !var_name.is_empty() {
                // eprintln!("🔍 DEBUG execute_line_simple: Valid identifier, evaluating expression: '{}'", expr);
                // eprintln!("🔍 DEBUG execute_line_simple: About to call eval_expr with: '{}'", expr);
                let val = interpreter.eval_expr(expr).map_err(|e| {
                    // eprintln!("❌ DEBUG execute_line_simple: Error evaluating expression '{}': {}", expr, e);
                    e
                })?;

                // Определяем, нужно ли обновить существующую переменную или создать новую
                // Сначала проверяем, существует ли переменная в текущих областях видимости
                if interpreter.get_variable(var_name).is_some() {
                    // Переменная существует, обновляем её с умным определением области видимости
                    interpreter.set_variable_smart(var_name.to_string(), val);
                } else {
                    // Переменная не существует, создаем как локальную (если в функции) или глобальную
                    let is_global = interpreter.variable_manager.call_stack.is_empty();
                    interpreter.set_variable(var_name.to_string(), val, is_global);
                }
                return Ok(());
            }
        }
    }



    // Обработка throw
    if trimmed_code.starts_with("throw ") {
        return handle_throw_statement(interpreter, trimmed_code);
    }

    // Все остальное - выражения
    // Но сначала проверяем, что код не пустой (может быть пустым после удаления комментариев лексером)
    if !trimmed_code.is_empty() {
        interpreter.eval_expr(trimmed_code)?;
    }
    Ok(())
}

/// Форматировать значение для вывода
fn format_value_for_print(value: &Value) -> String {
    use Value::*;
    match value {
        Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        String(s) => s.clone(),
        Bool(b) => b.to_string(),
        Currency(c) => c.clone(),
        Array(arr) => {
            let items: Vec<std::string::String> = arr.iter().map(format_value_for_print).collect();
            format!("[{}]", items.join(", "))
        }
        Object(obj) => {
            let items: Vec<std::string::String> = obj.iter()
                .map(|(k, v)| format!("{}: {}", k, format_value_for_print(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        Table(table) => {
            let table_borrowed = table.borrow();
            format!("Table({} rows, {} columns)", table_borrowed.rows.len(), table_borrowed.column_names.len())
        }
        TableColumn(_table, column) => {
            format!("Column({})", column)
        }
        TableIndexer(table) => {
            let table_borrowed = table.borrow();
            format!("TableIndexer({} rows, {} columns)", table_borrowed.rows.len(), table_borrowed.column_names.len())
        }
        Null => "null".to_string(),
        Path(p) => p.display().to_string(),
        PathPattern(p) => format!("Pattern({})", p.display()),
    }
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
        TableColumn(_, _) => true,
        TableIndexer(table) => !table.borrow().rows.is_empty(),
        Null => false,
        Path(p) => p.exists(),
        PathPattern(_) => true,
    }
}

/// Обработать определение функции
fn handle_function_definition(interpreter: &mut Interpreter, lines: &[&str], start: usize) -> Result<usize> {
    let mut function_lines = vec![lines[start]];
    let mut i = start + 1;

    while i < lines.len() {
        let current_line = lines[i].trim();
        function_lines.push(lines[i]);

        if current_line == "endfunction" {
            break;
        }
        i += 1;
    }

    // Парсим определение функции напрямую без рекурсии
    parse_and_define_function_directly(interpreter, &function_lines)?;

    Ok(i)
}

/// Извлечь имя переменной из строки "next variable"
fn parse_next_variable(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("next ") {
        let var_part = trimmed.strip_prefix("next ").unwrap().trim();
        if !var_part.is_empty() {
            return Some(var_part.to_string());
        }
    }
    None
}

/// Извлечь имя переменной из строки "for variable in ... do"
fn parse_for_variable(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("for ") && trimmed.ends_with(" do") {
        let for_part = trimmed.strip_prefix("for ").unwrap().strip_suffix(" do").unwrap();
        let parts: Vec<&str> = for_part.split(" in ").collect();
        if parts.len() == 2 {
            // Берем первую переменную (для деструктуризации типа "i, j" берем "i")
            let var_part = parts[0].trim();
            let first_var = var_part.split(',').next().unwrap_or(var_part).trim();
            if !first_var.is_empty() {
                return Some(first_var.to_string());
            }
        }
    }
    None
}

/// Обработать цикл for
fn handle_for_loop(interpreter: &mut Interpreter, lines: &[&str], start: usize) -> Result<usize> {
    let mut loop_lines = vec![lines[start]];
    
    // Извлекаем имя переменной текущего цикла
    let current_var = parse_for_variable(lines[start])
        .ok_or_else(|| DataCodeError::syntax_error("Invalid for syntax: expected 'for variable in iterable do'", interpreter.current_line, 0))?;
    
    // Стек переменных для отслеживания вложенных циклов
    let mut var_stack: Vec<String> = vec![current_var.clone()];
    let mut i = start + 1;
    
    // Отслеживаем вложенные try/catch и if/endif блоки
    let mut try_depth = 0;
    let mut if_depth = 0;

    while i < lines.len() && !var_stack.is_empty() {
        let current_line = lines[i].trim();

        // Обрабатываем вложенные try/catch блоки
        if current_line == "try" {
            try_depth += 1;
            loop_lines.push(lines[i]);
            i += 1;
            continue;
        } else if current_line == "endtry" {
            if try_depth > 0 {
                try_depth -= 1;
            }
            loop_lines.push(lines[i]);
            i += 1;
            continue;
        } else if current_line.starts_with("catch") && try_depth > 0 {
            // catch внутри try блока - пропускаем
            loop_lines.push(lines[i]);
            i += 1;
            continue;
        }
        
        // Обрабатываем вложенные if/endif блоки
        if current_line.starts_with("if ") && (current_line.contains(" do") || current_line.contains(" then")) {
            if_depth += 1;
        } else if current_line == "endif" || current_line == "endeif" {
            if if_depth > 0 {
                if_depth -= 1;
            }
        }

        // Если мы внутри try/catch или if/endif блока, просто добавляем строку
        // и не проверяем на next (next может быть внутри этих блоков)
        if try_depth > 0 || if_depth > 0 {
            loop_lines.push(lines[i]);
            i += 1;
            continue;
        }

        if current_line.starts_with("for ") && current_line.ends_with(" do") {
            // Новый вложенный цикл
            if let Some(var_name) = parse_for_variable(current_line) {
                var_stack.push(var_name);
            }
        } else if let Some(next_var) = parse_next_variable(current_line) {
            // Проверяем что next соответствует последнему циклу
            // Для множественных переменных берем первую переменную из next
            let next_first_var = next_var.split(',').next().unwrap_or(&next_var).trim();
            if let Some(last_var) = var_stack.last() {
                if next_first_var == *last_var || next_var == *last_var {
                    var_stack.pop();
                } else {
                    return Err(DataCodeError::syntax_error(
                        &format!("Mismatched next: expected 'next {}' but found 'next {}'", last_var, next_var),
                        interpreter.current_line,
                        0
                    ));
                }
            } else {
                return Err(DataCodeError::syntax_error(
                    "Unexpected next statement outside of for loop",
                    interpreter.current_line,
                    0
                ));
            }
        }

        loop_lines.push(lines[i]);

        if var_stack.is_empty() {
            break;
        }

        i += 1;
    }

    if !var_stack.is_empty() {
        return Err(DataCodeError::syntax_error(
            &format!("Missing 'next {}' in for loop", var_stack[0]),
            interpreter.current_line,
            0
        ));
    }

    // Выполняем цикл напрямую без рекурсии
    execute_for_loop_iteratively(interpreter, &loop_lines)?;

    Ok(i)
}

/// Обработать условную конструкцию if
fn handle_if_statement(interpreter: &mut Interpreter, lines: &[&str], start: usize) -> Result<usize> {
    let mut if_lines = vec![lines[start]];
    let mut if_depth = 1;
    let mut i = start + 1;

    while i < lines.len() && if_depth > 0 {
        let current_line = lines[i].trim();

        if current_line.starts_with("if ") && (current_line.contains(" do") || current_line.contains(" then")) {
            if_depth += 1;
        } else if current_line == "endif" || current_line == "endeif" {
            if_depth -= 1;
        }

        if_lines.push(lines[i]);

        if if_depth == 0 {
            break;
        }

        i += 1;
    }

    // Выполняем всю условную конструкцию напрямую
    // Парсим условную конструкцию и выполняем ее
    execute_if_statement_directly(interpreter, &if_lines)?;

    Ok(i)
}

/// Обработать блок try/catch
fn handle_try_statement(interpreter: &mut Interpreter, lines: &[&str], start: usize) -> Result<usize> {
    let mut try_lines = vec![lines[start]];
    let mut try_depth = 1;
    let mut i = start + 1;

    while i < lines.len() && try_depth > 0 {
        let current_line = lines[i].trim();

        if current_line == "try" {
            try_depth += 1;
        } else if current_line == "endtry" {
            try_depth -= 1;
        }

        try_lines.push(lines[i]);

        if try_depth == 0 {
            break;
        }

        i += 1;
    }

    // Выполняем try/catch блок
    execute_try_statement_directly(interpreter, &try_lines)?;

    // Возвращаем индекс строки endtry, чтобы в основном цикле он увеличился на 1
    // и следующая строка после endtry была выполнена
    Ok(i)
}

/// Выполнить условную конструкцию напрямую без рекурсии
/// Поддерживает: if ... do ... else if ... do ... else ... endif
fn execute_if_statement_directly(interpreter: &mut Interpreter, if_lines: &[&str]) -> Result<()> {
    if if_lines.is_empty() {
        return Ok(());
    }

    // Используем ту же логику, что и в execute_if_statement_iteratively
    // Парсим все блоки: if, else if (может быть несколько), else (опционально)
    struct ConditionalBlock<'a> {
        condition: String,
        body: Vec<&'a str>,
    }

    let mut blocks: Vec<ConditionalBlock> = Vec::new();
    let mut else_body: Option<Vec<&str>> = None;
    
    let mut i = 0;
    let mut depth = 0;
    let mut current_block_body: Vec<&str> = Vec::new();
    let mut current_condition: Option<String> = None;
    let mut in_else = false;

    while i < if_lines.len() {
        let line = if_lines[i];
        let trimmed = line.trim();

        // Сначала проверяем else if (должно быть до проверки if)
        if trimmed.starts_with("else if ") && (trimmed.contains(" do") || trimmed.contains(" then")) && depth == 1 {
            // Сохраняем предыдущий блок перед переходом к else if
            if let Some(condition) = current_condition.take() {
                blocks.push(ConditionalBlock {
                    condition,
                    body: current_block_body.clone(),
                });
                current_block_body.clear();
            }

            // Извлекаем условие из else if
            let condition_str = if let Some(stripped) = trimmed.strip_prefix("else if ") {
                if let Some(condition) = stripped.strip_suffix(" do") {
                    condition.trim().to_string()
                } else if let Some(condition) = stripped.strip_suffix(" then") {
                    condition.trim().to_string()
                } else {
                    return Err(DataCodeError::syntax_error("Invalid else if statement", interpreter.current_line, 0));
                }
            } else {
                return Err(DataCodeError::syntax_error("Invalid else if statement", interpreter.current_line, 0));
            };

            current_condition = Some(condition_str);
            i += 1;
            continue;
        }
        // Проверяем на вложенные if
        else if trimmed.starts_with("if ") && (trimmed.contains(" do") || trimmed.contains(" then")) {
            if depth == 0 {
                // Это начало нового блока if (первый if)
                // Сохраняем предыдущий блок, если он был
                if let Some(condition) = current_condition.take() {
                    blocks.push(ConditionalBlock {
                        condition,
                        body: current_block_body.clone(),
                    });
                    current_block_body.clear();
                }

                // Извлекаем условие
                let condition_str = if let Some(stripped) = trimmed.strip_prefix("if ") {
                    if let Some(condition) = stripped.strip_suffix(" do") {
                        condition.trim().to_string()
                    } else if let Some(condition) = stripped.strip_suffix(" then") {
                        condition.trim().to_string()
                    } else {
                        return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
                    }
                } else {
                    return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
                };

                current_condition = Some(condition_str);
            } else {
                // Вложенный if - добавляем в текущий блок
                current_block_body.push(line);
            }
            depth += 1;
        } else if trimmed == "endif" || trimmed == "endeif" {
            if depth == 0 {
                // Конец всей конструкции
                if let Some(condition) = current_condition.take() {
                    blocks.push(ConditionalBlock {
                        condition,
                        body: current_block_body.clone(),
                    });
                } else if in_else {
                    // Сохраняем else блок
                    else_body = Some(current_block_body.clone());
                }
                break;
            }
            depth -= 1;
            if depth > 0 {
                // Вложенный endif/endeif - добавляем в текущий блок
                current_block_body.push(line);
            }
        } else if trimmed == "else" && depth == 1 {
            // Сохраняем текущий блок перед переходом к else
            if let Some(condition) = current_condition.take() {
                blocks.push(ConditionalBlock {
                    condition,
                    body: current_block_body.clone(),
                });
                current_block_body.clear();
            }
            in_else = true;
            i += 1;
            continue;
        } else {
            // Обычная строка - добавляем в текущий блок
            current_block_body.push(line);
        }

        i += 1;
    }

    // Сохраняем последний блок, если он был
    if let Some(condition) = current_condition {
        blocks.push(ConditionalBlock {
            condition,
            body: current_block_body,
        });
    } else if in_else && else_body.is_none() {
        else_body = Some(current_block_body);
    }

    // Проверяем условия по порядку и выполняем первое истинное
    let mut executed = false;
    for block in &blocks {
        let condition_value = eval_condition_safe(interpreter, &block.condition)?;
        if to_bool(&condition_value) {
            // Выполняем этот блок
            execute_block_directly(interpreter, &block.body)?;
            executed = true;
            // Проверяем break после выполнения if блока - НЕ сбрасываем флаг
            if interpreter.break_requested {
                return Ok(());
            }
            // Проверяем continue после выполнения if блока - НЕ сбрасываем флаг
            if interpreter.continue_requested {
                return Ok(());
            }
            break;
        }
    }

    // Если ни одно условие не выполнилось, выполняем else блок
    if !executed {
        if let Some(ref else_body_lines) = else_body {
            execute_block_directly(interpreter, else_body_lines)?;
            // Проверяем break после выполнения else блока - НЕ сбрасываем флаг
            if interpreter.break_requested {
                return Ok(());
            }
            // Проверяем continue после выполнения else блока - НЕ сбрасываем флаг
            if interpreter.continue_requested {
                return Ok(());
            }
        }
    }

    Ok(())
}

/// Выполнить блок кода напрямую с обработкой вложенных конструкций
pub fn execute_block_directly(interpreter: &mut Interpreter, lines: &[&str]) -> Result<()> {
    let debug = std::env::var("DEBUG_FOR_LOOP").is_ok();
    let debug_all = std::env::var("DATACODE_DEBUG").is_ok();
    let mut i = 0;
    while i < lines.len() {
        // Проверяем, был ли выполнен return, break или continue - если да, прекращаем выполнение
        if interpreter.return_value.is_some() {
            return Ok(());
        }
        // break должен пробрасываться наверх к циклу, не сбрасываем флаг здесь
        if interpreter.break_requested {
            return Ok(());
        }
        // continue должен пробрасываться наверх к циклу, не сбрасываем флаг здесь
        if interpreter.continue_requested {
            return Ok(());
        }
        
        let line = lines[i].trim();
        if debug_all {
            eprintln!("🔍 DEBUG execute_block_directly: Processing line {}: '{}'", i, line);
        }
        
        if debug && line.starts_with("for ") {
            eprintln!("🔍 DEBUG: execute_block_directly processing line {}: {}", i, line);
        }
        
        // Пропускаем пустые строки и комментарии
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Обрабатываем next statements - они могут быть маркерами конца цикла или командами для пропуска итерации
        if line.starts_with("next ") {
            // Проверяем, находимся ли мы внутри цикла
            if interpreter.active_loop_count > 0 {
                // Устанавливаем флаг для пропуска текущей итерации
                interpreter.continue_requested = true;
                return Ok(()); // Прерываем выполнение блока, чтобы цикл мог обработать continue
            } else {
                // Не внутри цикла - это просто маркер конца цикла, пропускаем
                i += 1;
                continue;
            }
        }

        // Проверяем на условные конструкции
        // Более гибкая проверка: if должен содержать " do" или " then" где-то в строке
        if line.starts_with("if ") && (line.contains(" do") || line.contains(" then")) {
            // Собираем всю условную конструкцию
            let mut if_lines = vec![lines[i]];
            let mut if_depth = 1;
            let mut j = i + 1;

            while j < lines.len() && if_depth > 0 {
                let current_line = lines[j].trim();

                if current_line.starts_with("if ") && (current_line.contains(" do") || current_line.contains(" then")) {
                    if_depth += 1;
                } else if current_line == "endif" || current_line == "endeif" {
                    if_depth -= 1;
                }

                if_lines.push(lines[j]);

                if if_depth == 0 {
                    break;
                }
                j += 1;
            }

            // Выполняем условную конструкцию итеративно
            execute_if_statement_iteratively(interpreter, &if_lines)?;
            
            // Проверяем return после выполнения if
            if interpreter.return_value.is_some() {
                return Ok(());
            }
            // Проверяем break после выполнения if - НЕ сбрасываем флаг, просто возвращаемся
            // break должен прерывать цикл, а не if блок
            if interpreter.break_requested {
                return Ok(());
            }
            
            i = j + 1;
        } else if line.starts_with("for ") && line.ends_with(" do") {
            // Обрабатываем циклы for
            let debug = std::env::var("DEBUG_FOR_LOOP").is_ok();
            if debug {
                eprintln!("🔍 DEBUG execute_block_directly: Found for loop at line {}: {}", i, line);
            }
            
            let mut for_lines = vec![lines[i]];
            
            // Извлекаем имя переменной текущего цикла
            let current_var = parse_for_variable(lines[i])
                .ok_or_else(|| DataCodeError::syntax_error("Invalid for loop syntax", interpreter.current_line, 0))?;
            
            if debug {
                eprintln!("  Current var: {}", current_var);
            }
            
            // Стек переменных для отслеживания вложенных циклов
            let mut var_stack: Vec<String> = vec![current_var.clone()];
            // Отслеживаем вложенные try/catch и if/endif блоки
            let mut try_depth = 0;
            let mut if_depth = 0;
            let mut j = i + 1;

            while j < lines.len() && !var_stack.is_empty() {
                let current_line = lines[j].trim();

                // Обрабатываем вложенные try/catch блоки
                if current_line == "try" {
                    try_depth += 1;
                    for_lines.push(lines[j]);
                } else if current_line == "endtry" {
                    if try_depth > 0 {
                        try_depth -= 1;
                    }
                    for_lines.push(lines[j]);
                } else if current_line.starts_with("catch") && try_depth > 0 {
                    // catch внутри try блока - добавляем и пропускаем проверку
                    for_lines.push(lines[j]);
                } else if current_line.starts_with("if ") && (current_line.contains(" do") || current_line.contains(" then")) {
                    if_depth += 1;
                    for_lines.push(lines[j]);
                } else if current_line == "endif" || current_line == "endeif" {
                    if if_depth > 0 {
                        if_depth -= 1;
                    }
                    for_lines.push(lines[j]);
                } else if try_depth > 0 || if_depth > 0 {
                    // Мы внутри try/catch или if/endif блока - просто добавляем строку
                    // и не проверяем на next (next может быть внутри этих блоков)
                    for_lines.push(lines[j]);
                } else if current_line.starts_with("for ") && current_line.ends_with(" do") {
                    // Новый вложенный цикл
                    if let Some(var_name) = parse_for_variable(current_line) {
                        var_stack.push(var_name);
                    }
                    // Добавляем строку в for_lines, чтобы она была частью тела цикла
                    for_lines.push(lines[j]);
                } else if let Some(next_var) = parse_next_variable(current_line) {
                    // Проверяем что next соответствует последнему циклу
                    // Для множественных переменных проверяем первую переменную
                    let next_first_var = next_var.split(',').next().unwrap_or(&next_var).trim();
                    if let Some(last_var) = var_stack.last() {
                        if next_first_var == *last_var || next_var == *last_var {
                            var_stack.pop();
                            // Добавляем next в for_lines, но не выполняем его как код
                            // next - это просто маркер конца цикла
                            for_lines.push(lines[j]);
                        } else {
                            // Неправильный next - это ошибка
                            return Err(DataCodeError::syntax_error(
                                &format!("Mismatched next: expected 'next {}' but found 'next {}'", last_var, next_var),
                                interpreter.current_line,
                                0
                            ));
                        }
                    } else {
                        return Err(DataCodeError::syntax_error(
                            "Unexpected next statement outside of for loop",
                            interpreter.current_line,
                            0
                        ));
                    }
                } else {
                    for_lines.push(lines[j]);
                }

                if var_stack.is_empty() {
                    break;
                }
                j += 1;
            }

            if !var_stack.is_empty() {
                return Err(DataCodeError::syntax_error(
                    &format!("Missing 'next {}' in for loop", var_stack[0]),
                    interpreter.current_line,
                    0
                ));
            }

            // Выполняем цикл for итеративно
            execute_for_loop_iteratively(interpreter, &for_lines)?;
            
            // Проверяем return после выполнения цикла
            if interpreter.return_value.is_some() {
                return Ok(());
            }
            
            i = j + 1;
        } else if line == "try" {
            // Обрабатываем try блоки
            let mut try_lines = vec![lines[i]];
            let mut try_depth = 1;
            let mut j = i + 1;

            while j < lines.len() && try_depth > 0 {
                let current_line = lines[j].trim();

                if current_line == "try" {
                    try_depth += 1;
                } else if current_line == "endtry" {
                    try_depth -= 1;
                }

                try_lines.push(lines[j]);

                if try_depth == 0 {
                    break;
                }
                j += 1;
            }

            // Выполняем try блок
            execute_try_statement_directly(interpreter, &try_lines)?;
            
            // Проверяем return после выполнения try
            if interpreter.return_value.is_some() {
                return Ok(());
            }
            
            i = j + 1;
        } else if is_incomplete_assignment(line) {
            // Обрабатываем многострочные присваивания в блоке
            let mut assignment_lines = vec![lines[i]];
            let mut j = i + 1;
            
            while j < lines.len() {
                assignment_lines.push(lines[j]);
                let combined = assignment_lines.join("\n");
                if !is_incomplete_assignment(&combined) {
                    break;
                }
                j += 1;
            }
            
            // Выполняем полное многострочное присваивание
            let combined_assignment = assignment_lines.join("\n");
            // Используем execute_line_simple, так как он правильно обрабатывает многострочные присваивания
            execute_line_simple(interpreter, &combined_assignment)?;
            
            // Проверяем return после выполнения присваивания
            if interpreter.return_value.is_some() {
                return Ok(());
            }
            i = j + 1;
        } else {
            // Обычная строка - проверяем, находимся ли мы внутри Call Frame Engine
            if interpreter.use_call_frame_engine && !interpreter.call_stack.is_empty() {
                // Внутри Call Frame Engine - используем execute_instruction_signal для правильной обработки return
                if std::env::var("DATACODE_DEBUG").is_ok() {
                    eprintln!("🔍 DEBUG execute_block_directly: Using execute_instruction_signal for line: '{}'", line);
                }
                use crate::interpreter::ExecSignal;
                let signal = interpreter.execute_instruction_signal(lines[i])?;
                
                // Проверяем break сразу после выполнения инструкции (независимо от типа сигнала)
                if interpreter.break_requested {
                    interpreter.break_requested = false; // Сбрасываем флаг break
                    return Ok(());
                }
                
                match signal {
                    ExecSignal::Value(_) => {
                        // Инструкция выполнена, продолжаем
                    }
                    ExecSignal::Return(return_value) => {
                        // Return - устанавливаем return_value и прекращаем выполнение
                        if std::env::var("DATACODE_DEBUG").is_ok() {
                            eprintln!("🔍 DEBUG execute_block_directly: Return detected with value: {:?}", return_value);
                        }
                        interpreter.return_value = Some(return_value);
                        return Ok(());
                    }
                    ExecSignal::Call { function_id, args, return_slot } => {
                        // Вызов функции внутри блока - возвращаем сигнал Call обратно в главный цикл
                        // ВАЖНО: НЕ обрабатываем его здесь, а возвращаем через специальный механизм
                        // Сохраняем информацию о вызове функции в специальном поле интерпретатора
                        // и возвращаем ошибку, которая будет обработана главным циклом
                        // Но на самом деле, нам нужно вернуть ExecSignal::Call обратно в главный цикл
                        // Для этого используем специальный механизм: сохраняем ExecSignal::Call в return_value
                        // и возвращаем специальный флаг
                        // Но это сложно. Вместо этого, просто обрабатываем ExecSignal::Call здесь,
                        // создавая новый фрейм и продолжая выполнение
                        // ВАЖНО: Это должно обрабатываться главным циклом call_user_function
                        // Но так как мы уже внутри главного цикла, мы можем обработать это здесь
                        // Создаем новый фрейм и продолжаем выполнение
                        let called_function = interpreter.function_manager.get_function(&function_id)
                            .ok_or_else(|| DataCodeError::function_not_found(&function_id, interpreter.current_line))?;
                        
                        if called_function.parameters.len() != args.len() {
                            return Err(DataCodeError::wrong_argument_count(
                                &function_id,
                                called_function.parameters.len(),
                                args.len(),
                                interpreter.current_line,
                            ));
                        }
                        
                        use crate::interpreter::call_frame::CallFrame;
                        let new_frame = CallFrame::new(
                            function_id.clone(),
                            args,
                            return_slot,
                            interpreter.call_stack.len(),
                        );
                        
                        interpreter.call_stack.push(new_frame)?;
                        interpreter.variable_manager.enter_function_scope();
                        
                        if let Some(frame) = interpreter.call_stack.last_mut() {
                            let args = frame.args.clone();
                            for (param, arg_value) in called_function.parameters.iter().zip(args.iter()) {
                                frame.set_local(param.clone(), arg_value.clone());
                                if let Some(local_vars) = interpreter.variable_manager.call_stack.last_mut() {
                                    local_vars.insert(param.clone(), arg_value.clone());
                                }
                            }
                        }
                        
                        // Возвращаемся из execute_block_directly, чтобы главный цикл мог обработать новый фрейм
                        // Но это сложно, потому что мы теряем контекст выполнения блока
                        // Вместо этого, просто продолжаем выполнение в главном цикле
                        // Но execute_block_directly вызывается из главного цикла, поэтому мы можем просто
                        // вернуть Ok(()), и главный цикл продолжит выполнение нового фрейма
                        // Но проблема в том, что мы теряем контекст выполнения блока
                        // Поэтому нужно использовать другой подход: сохранить состояние выполнения блока
                        // и продолжить его после возврата из функции
                        // Но это очень сложно
                        // Вместо этого, просто обрабатываем ExecSignal::Call здесь, создавая новый фрейм
                        // и продолжая выполнение в главном цикле
                        // Но execute_block_directly вызывается из главного цикла, поэтому мы можем просто
                        // вернуть Ok(()), и главный цикл продолжит выполнение нового фрейма
                        return Ok(());
                    }
                }
            } else {
                // Не внутри Call Frame Engine - используем безопасную версию
                execute_line_simple_safe(interpreter, lines[i])?;
                
                // Проверяем return после выполнения строки
                if interpreter.return_value.is_some() {
                    return Ok(());
                }
                // Проверяем break после выполнения строки - НЕ сбрасываем флаг
                if interpreter.break_requested {
                    return Ok(());
                }
            }
            i += 1;
        }

        // Проверяем return или break
        if interpreter.return_value.is_some() || interpreter.break_requested {
            break;
        }
    }
    Ok(())
}

/// Выполнить условную конструкцию if итеративно (без рекурсии)
/// Поддерживает: if ... do ... else if ... do ... else ... endif
fn execute_if_statement_iteratively(interpreter: &mut Interpreter, if_lines: &[&str]) -> Result<()> {
    if if_lines.is_empty() {
        return Err(DataCodeError::syntax_error("Empty if statement", interpreter.current_line, 0));
    }

    // Парсим все блоки: if, else if (может быть несколько), else (опционально)
    struct ConditionalBlock<'a> {
        condition: String,
        body: Vec<&'a str>,
    }

    let mut blocks: Vec<ConditionalBlock> = Vec::new();
    let mut else_body: Option<Vec<&str>> = None;
    
    let mut i = 0;
    let mut depth = 0;
    let mut current_block_body: Vec<&str> = Vec::new();
    let mut current_condition: Option<String> = None;
    let mut in_else = false;

    while i < if_lines.len() {
        let line = if_lines[i];
        let trimmed = line.trim();

        // Сначала проверяем else if (должно быть до проверки if)
        if trimmed.starts_with("else if ") && (trimmed.contains(" do") || trimmed.contains(" then")) && depth == 1 {
            // Сохраняем предыдущий блок перед переходом к else if
            if let Some(condition) = current_condition.take() {
                blocks.push(ConditionalBlock {
                    condition,
                    body: current_block_body.clone(),
                });
                current_block_body.clear();
            }

            // Извлекаем условие из else if
            let condition_str = if let Some(stripped) = trimmed.strip_prefix("else if ") {
                if let Some(condition) = stripped.strip_suffix(" do") {
                    condition.trim().to_string()
                } else if let Some(condition) = stripped.strip_suffix(" then") {
                    condition.trim().to_string()
                } else {
                    return Err(DataCodeError::syntax_error("Invalid else if statement", interpreter.current_line, 0));
                }
            } else {
                return Err(DataCodeError::syntax_error("Invalid else if statement", interpreter.current_line, 0));
            };

            current_condition = Some(condition_str);
            i += 1;
            continue;
        }
        // Проверяем на вложенные if
        else if trimmed.starts_with("if ") && (trimmed.contains(" do") || trimmed.contains(" then")) {
            if depth == 0 {
                // Это начало нового блока if (первый if)
                // Сохраняем предыдущий блок, если он был
                if let Some(condition) = current_condition.take() {
                    blocks.push(ConditionalBlock {
                        condition,
                        body: current_block_body.clone(),
                    });
                    current_block_body.clear();
                }

                // Извлекаем условие
                let condition_str = if let Some(stripped) = trimmed.strip_prefix("if ") {
                    if let Some(condition) = stripped.strip_suffix(" do") {
                        condition.trim().to_string()
                    } else if let Some(condition) = stripped.strip_suffix(" then") {
                        condition.trim().to_string()
                    } else {
                        return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
                    }
                } else {
                    return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
                };

                current_condition = Some(condition_str);
            } else {
                // Вложенный if - добавляем в текущий блок
                current_block_body.push(line);
            }
            depth += 1;
        } else if trimmed == "endif" || trimmed == "endeif" {
            if depth == 0 {
                // Конец всей конструкции
                if let Some(condition) = current_condition.take() {
                    blocks.push(ConditionalBlock {
                        condition,
                        body: current_block_body.clone(),
                    });
                } else if in_else {
                    // Сохраняем else блок
                    else_body = Some(current_block_body.clone());
                }
                break;
            }
            depth -= 1;
            if depth > 0 {
                // Вложенный endif/endeif - добавляем в текущий блок
                current_block_body.push(line);
            }
        } else if trimmed == "else" && depth == 1 {
            // Сохраняем текущий блок перед переходом к else
            if let Some(condition) = current_condition.take() {
                blocks.push(ConditionalBlock {
                    condition,
                    body: current_block_body.clone(),
                });
                current_block_body.clear();
            }
            in_else = true;
            i += 1;
            continue;
        } else {
            // Обычная строка - добавляем в текущий блок
            current_block_body.push(line);
        }

        i += 1;
    }

    // Сохраняем последний блок, если он был
    if let Some(condition) = current_condition {
        blocks.push(ConditionalBlock {
            condition,
            body: current_block_body,
        });
    } else if in_else && else_body.is_none() {
        else_body = Some(current_block_body);
    }

    // Проверяем условия по порядку и выполняем первое истинное
    let mut executed = false;
    for block in &blocks {
        let condition_value = eval_condition_without_user_functions(interpreter, &block.condition)?;
        if is_truthy(&condition_value) {
            // Выполняем этот блок - используем execute_block_directly для правильной обработки вложенных циклов
            execute_block_directly(interpreter, &block.body)?;
            executed = true;
            // Проверяем break после выполнения if блока - НЕ сбрасываем флаг
            // break должен прерывать цикл, а не if блок
            if interpreter.break_requested {
                return Ok(());
            }
            // Проверяем continue после выполнения if блока - НЕ сбрасываем флаг
            // continue должен пропускать итерацию цикла, а не if блок
            if interpreter.continue_requested {
                return Ok(());
            }
            break;
        }
    }

    // Если ни одно условие не выполнилось, выполняем else блок
    if !executed {
        if let Some(ref else_body_lines) = else_body {
            // Используем execute_block_directly для правильной обработки вложенных циклов
            execute_block_directly(interpreter, else_body_lines)?;
            // Проверяем break после выполнения else блока - НЕ сбрасываем флаг
            // break должен прерывать цикл, а не if блок
            if interpreter.break_requested {
                return Ok(());
            }
            // Проверяем continue после выполнения else блока - НЕ сбрасываем флаг
            // continue должен пропускать итерацию цикла, а не if блок
            if interpreter.continue_requested {
                return Ok(());
            }
        }
    }

    Ok(())
}

/// Безопасное выполнение строки кода без вызова пользовательских функций (для предотвращения рекурсии)
fn execute_line_simple_safe(interpreter: &mut Interpreter, code: &str) -> Result<()> {
    let trimmed_code = code.trim();

    // Пропускаем пустые строки и комментарии
    if trimmed_code.is_empty() || trimmed_code.starts_with('#') {
        return Ok(());
    }

    // Обработка break (должна быть раньше парсинга выражений)
    if trimmed_code == "break" {
        // Проверяем, что мы внутри цикла
        if interpreter.active_loop_count == 0 {
            return Err(DataCodeError::syntax_error(
                "break statement must be inside a loop",
                interpreter.current_line,
                0
            ));
        }
        interpreter.break_requested = true;
        return Ok(());
    }

    // Пропускаем ключевые слова блочных конструкций (они обрабатываются на уровне выше)
    // Также обрабатываем любую строку, начинающуюся с "if", чтобы избежать попытки парсить её как выражение
    if trimmed_code == "else" || trimmed_code == "endif" || trimmed_code == "endeif" || 
       trimmed_code == "endfunction" || trimmed_code.starts_with("next ") ||
       trimmed_code == "try" || trimmed_code == "catch" || trimmed_code == "finally" || trimmed_code == "endtry" ||
       (trimmed_code.starts_with("for ") && trimmed_code.ends_with(" do")) ||
       trimmed_code.starts_with("if ") {
        return Ok(());
    }

    // Обработка return
    // ВАЖНО: Если мы внутри Call Frame Engine, return должен обрабатываться через execute_instruction_signal
    // в главном цикле call_user_function, а не здесь. Поэтому просто пропускаем return здесь.
    if trimmed_code.starts_with("return") {
        // Если мы внутри Call Frame Engine, return обрабатывается в главном цикле
        if interpreter.use_call_frame_engine && !interpreter.call_stack.is_empty() {
            // Пропускаем обработку return здесь - он будет обработан через execute_instruction_signal
            // в главном цикле call_user_function
            return Ok(());
        }
        
        // Если мы не внутри Call Frame Engine, обрабатываем return здесь
        // Проверяем, что мы внутри функции
        if interpreter.variable_manager.call_stack.is_empty() && interpreter.call_stack.is_empty() {
            return Err(DataCodeError::syntax_error(
                "return statement must be inside a function",
                interpreter.current_line,
                0
            ));
        }
        let after_return = trimmed_code.strip_prefix("return").unwrap().trim();
        let value = if after_return.is_empty() {
            Value::Null
        } else {
            // Используем безопасную версию eval_expr, которая не вызывает пользовательские функции
            eval_expr_safe(interpreter, after_return)?
        };
        interpreter.return_value = Some(value);
        return Ok(());
    }

    // Обработка присваивания переменных
    if trimmed_code.starts_with("global ") || trimmed_code.starts_with("local ") {
        let is_global = trimmed_code.starts_with("global ");
        let code = &trimmed_code[if is_global { 7 } else { 6 }..];
        let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();

        if parts.len() != 2 {
            return Err(DataCodeError::syntax_error("Invalid assignment", interpreter.current_line, 0));
        }

        let var_name = parts[0].to_string();
        let expr = parts[1];

        let val = eval_expr_safe(interpreter, expr)?;
        interpreter.set_variable(var_name, val, is_global);
        return Ok(());
    }

    // Обработка присваивания без явного global/local префикса
    if trimmed_code.contains('=') && !trimmed_code.contains("==") && !trimmed_code.contains("!=") && !trimmed_code.contains("<=") && !trimmed_code.contains(">=") {
        let parts: Vec<_> = trimmed_code.splitn(2, '=').map(|s| s.trim()).collect();

        if parts.len() == 2 {
            let var_name = parts[0];
            let expr = parts[1];

            // Проверяем, что левая часть - это простой идентификатор (не выражение)
            if var_name.chars().all(|c| c.is_alphanumeric() || c == '_') && !var_name.is_empty() {
                let val = eval_expr_safe(interpreter, expr)?;

                // Определяем, нужно ли обновить существующую переменную или создать новую
                // Сначала проверяем, существует ли переменная в текущих областях видимости
                if interpreter.get_variable(var_name).is_some() {
                    // Переменная существует, обновляем её с умным определением области видимости
                    interpreter.set_variable_smart(var_name.to_string(), val);
                } else {
                    // Переменная не существует, создаем как локальную (если в функции) или глобальную
                    let is_global = interpreter.variable_manager.call_stack.is_empty();
                    interpreter.set_variable(var_name.to_string(), val, is_global);
                }
                return Ok(());
            }
        }
    }

    // Обработка print statement (должна быть до throw и выражений)
    if trimmed_code.starts_with("print(") {
        // Извлекаем аргументы из print(...)
        if let Some(args_str) = trimmed_code.strip_prefix("print(") {
            if let Some(close_paren_pos) = args_str.rfind(')') {
                let args_content = &args_str[..close_paren_pos];
                
                // Парсим аргументы (разделенные запятыми)
                let args: Vec<Value> = if args_content.trim().is_empty() {
                    Vec::new()
                } else {
                    // Разделяем по запятым, но учитываем вложенные скобки и кавычки
                    let mut args_list = Vec::new();
                    let mut current_arg = String::new();
                    let mut depth = 0;
                    let mut in_string = false;
                    let mut string_char: Option<char> = None;
                    
                    for ch in args_content.chars() {
                        match ch {
                            '\'' | '"' if !in_string => {
                                // Начало строки
                                in_string = true;
                                string_char = Some(ch);
                                current_arg.push(ch);
                            }
                            ch if in_string && Some(ch) == string_char => {
                                // Конец строки
                                in_string = false;
                                string_char = None;
                                current_arg.push(ch);
                            }
                            '(' | '[' | '{' if !in_string => {
                                depth += 1;
                                current_arg.push(ch);
                            }
                            ')' | ']' | '}' if !in_string => {
                                depth -= 1;
                                current_arg.push(ch);
                            }
                            ',' if depth == 0 && !in_string => {
                                if !current_arg.trim().is_empty() {
                                    args_list.push(current_arg.trim().to_string());
                                }
                                current_arg.clear();
                            }
                            _ => {
                                current_arg.push(ch);
                            }
                        }
                    }
                    
                    if !current_arg.trim().is_empty() {
                        args_list.push(current_arg.trim().to_string());
                    }
                    
                    // Вычисляем каждый аргумент
                    args_list.into_iter()
                        .map(|arg| eval_expr_safe(interpreter, &arg))
                        .collect::<Result<Vec<_>>>()?
                };
                
                // Вызываем встроенную функцию print
                use crate::builtins::system::call_system_function;
                call_system_function("print", args, interpreter.current_line)?;
                return Ok(());
            }
        }
    }

    // Обработка throw
    if trimmed_code.starts_with("throw ") {
        return handle_throw_statement(interpreter, trimmed_code);
    }

    // Все остальное - выражения
    // Но сначала проверяем, что код не пустой (может быть пустым после удаления комментариев лексером)
    if !trimmed_code.is_empty() {
        eval_expr_safe(interpreter, trimmed_code)?;
    }
    Ok(())
}

/// Безопасная версия eval_expr, которая использует обычный eval_expr с защитой от рекурсии
fn eval_expr_safe(interpreter: &mut Interpreter, expr: &str) -> Result<Value> {
    // Теперь используем обычный eval_expr, так как у нас есть защита от рекурсии в call_user_function
    interpreter.eval_expr(expr)
}

/// Безопасное вычисление условия с предотвращением рекурсии
fn eval_condition_safe(interpreter: &mut Interpreter, condition_str: &str) -> Result<Value> {
    // Проверяем, содержит ли условие вызов пользовательской функции
    if condition_str.contains('(') && condition_str.contains(')') {
        // Ищем имя функции
        let parts: Vec<&str> = condition_str.split('(').collect();
        if parts.len() >= 2 {
            let function_name = parts[0].trim();

            // Если это пользовательская функция, выполняем ее с ограниченной глубиной
            if interpreter.has_user_function(function_name) {
                // Увеличиваем счетчик рекурсии перед вызовом - увеличиваем лимит до 100
                if interpreter.recursion_depth >= 100 {
                    return Err(DataCodeError::runtime_error(
                        &format!("Maximum recursion depth exceeded in condition evaluation for function '{}'", function_name),
                        interpreter.current_line
                    ));
                }

                // Временно увеличиваем глубину рекурсии
                let old_depth = interpreter.recursion_depth;
                interpreter.recursion_depth += 1;

                let result = interpreter.eval_expr(condition_str);

                // Восстанавливаем глубину рекурсии
                interpreter.recursion_depth = old_depth;

                return result;
            }
        }
    }

    // Если это не пользовательская функция или простое выражение, выполняем обычным способом
    interpreter.eval_expr(condition_str)
}

/// Вычислить условие с безопасным выполнением пользовательских функций
fn eval_condition_without_user_functions(interpreter: &mut Interpreter, condition_str: &str) -> Result<Value> {
    // Проверяем, содержит ли условие вызов пользовательской функции
    if condition_str.contains('(') && condition_str.contains(')') {
        // Ищем имя функции
        let parts: Vec<&str> = condition_str.split('(').collect();
        if parts.len() >= 2 {
            let function_name = parts[0].trim();

            // Если это пользовательская функция, выполняем ее безопасно
            if interpreter.has_user_function(function_name) {
                return execute_user_function_safely(interpreter, condition_str);
            }
        }
    }

    // Если это не пользовательская функция, выполняем обычным способом
    interpreter.eval_expr(condition_str)
}

/// Разделить аргументы функции с учетом строк в кавычках и вложенных скобок
fn split_function_args(args_str: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = None;
    
    for ch in args_str.chars() {
        match ch {
            '"' | '\'' if !in_string => {
                in_string = true;
                string_char = Some(ch);
                current_arg.push(ch);
            }
            ch if in_string && Some(ch) == string_char => {
                in_string = false;
                string_char = None;
                current_arg.push(ch);
            }
            '(' if !in_string => {
                depth += 1;
                current_arg.push(ch);
            }
            ')' if !in_string => {
                depth -= 1;
                current_arg.push(ch);
            }
            ',' if !in_string && depth == 0 => {
                if !current_arg.trim().is_empty() {
                    args.push(current_arg.trim().to_string());
                }
                current_arg.clear();
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }
    
    if !current_arg.trim().is_empty() {
        args.push(current_arg.trim().to_string());
    }
    
    args
}

/// Безопасное выполнение пользовательской функции без рекурсии
fn execute_user_function_safely(interpreter: &mut Interpreter, function_call: &str) -> Result<Value> {
    // Парсим вызов функции
    let parts: Vec<&str> = function_call.split('(').collect();
    if parts.len() != 2 {
        return Err(DataCodeError::syntax_error("Invalid function call", interpreter.current_line, 0));
    }

    let function_name = parts[0].trim();
    let args_str = parts[1].trim_end_matches(')').trim();

    // Получаем функцию
    let function = interpreter.function_manager.get_function(function_name)
        .ok_or_else(|| DataCodeError::function_not_found(function_name, interpreter.current_line))?
        .clone();

    // Парсим аргументы правильно, учитывая строки в кавычках и выражения
    let mut args = Vec::new();
    if !args_str.is_empty() {
        // Разделяем аргументы с учетом строк в кавычках
        let arg_exprs = split_function_args(args_str);
        for arg_expr in arg_exprs {
            // Вычисляем каждое выражение через eval_expr
            match interpreter.eval_expr(&arg_expr) {
                Ok(value) => args.push(value),
                Err(e) => {
                    // Если не удалось вычислить, пытаемся как строку (убираем кавычки)
                    let cleaned = arg_expr.trim().trim_matches('"').trim_matches('\'').to_string();
                    args.push(Value::String(cleaned));
                }
            }
        }
    }

    // Проверяем количество аргументов
    if args.len() != function.parameters.len() {
        return Err(DataCodeError::runtime_error(
            &format!("Function '{}' expects {} arguments, got {}", function_name, function.parameters.len(), args.len()),
            interpreter.current_line
        ));
    }

    // Входим в новую область видимости
    interpreter.variable_manager.enter_function_scope();

    // Устанавливаем параметры
    for (param, arg) in function.parameters.iter().zip(args.iter()) {
        interpreter.variable_manager.set_variable(param.clone(), arg.clone(), false);
    }

    // Выполняем тело функции через execute_block_directly для правильной обработки всех конструкций
    let mut result = Value::Null;
    
    // Используем execute_block_directly для правильной обработки try-catch, if-else и других конструкций
    // Это позволяет функциям работать корректно даже в условиях if
    use crate::interpreter::execution::execute_block_directly;
    
    // Преобразуем Vec<String> в Vec<&str> для execute_block_directly
    let body_lines: Vec<&str> = function.body.iter().map(|s| s.as_str()).collect();
    
    // Выполняем тело функции
    execute_block_directly(interpreter, &body_lines)?;
    
    // Проверяем, был ли установлен return_value
    if let Some(return_val) = interpreter.return_value.take() {
        result = return_val;
    } else {
        // Если функция не вернула значение явно, возвращаем Null
        result = Value::Null;
    }

    // Выходим из области видимости
    interpreter.variable_manager.exit_function_scope();

    Ok(result)
}

/// Проверить, является ли значение истинным
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Null => false,
        _ => true,
    }
}

/// Выполнить цикл for итеративно (без рекурсии)
fn execute_for_loop_iteratively(interpreter: &mut Interpreter, for_lines: &[&str]) -> Result<()> {
    if for_lines.is_empty() {
        return Err(DataCodeError::syntax_error("Empty for loop", interpreter.current_line, 0));
    }

    // Первая строка должна быть "for variable in iterable do"
    let first_line = for_lines[0].trim();
    if !first_line.starts_with("for ") || !first_line.ends_with(" do") {
        return Err(DataCodeError::syntax_error("Invalid for loop syntax", interpreter.current_line, 0));
    }

    // Увеличиваем счетчик активных циклов
    interpreter.active_loop_count += 1;

    // Парсим строку "for variable in iterable do"
    let for_part = first_line.strip_prefix("for ").unwrap().strip_suffix(" do").unwrap();
    let parts: Vec<&str> = for_part.split(" in ").collect();

    if parts.len() != 2 {
        return Err(DataCodeError::syntax_error("Invalid for syntax: expected 'for variable in iterable do'", interpreter.current_line, 0));
    }

    let variable_part = parts[0].trim();
    let iterable_part = parts[1].trim();

    // Проверяем на деструктуризацию (например, "i, data")
    let variables: Vec<&str> = variable_part.split(',').map(|v| v.trim()).collect();

    // Вычисляем итерируемое значение
    let debug = std::env::var("DEBUG_FOR_LOOP").is_ok();
    if debug {
        eprintln!("🔍 DEBUG execute_for_loop_iteratively: Parsing for loop");
        eprintln!("  variable_part = '{}'", variable_part);
        eprintln!("  iterable_part = '{}'", iterable_part);
        eprintln!("  variables = {:?} (count = {})", variables, variables.len());
    }
    
    let iterable_value = interpreter.eval_expr(iterable_part)?;
    
    if debug {
        eprintln!("  iterable_value type = {:?}, len = {:?}", 
            match &iterable_value {
                Value::Array(arr) => format!("Array({})", arr.len()),
                _ => format!("{:?}", iterable_value),
            },
            match &iterable_value {
                Value::Array(arr) => arr.len(),
                _ => 0,
            }
        );
        if let Value::Array(ref arr) = iterable_value {
            if !arr.is_empty() {
                eprintln!("  First element: {:?}", arr[0]);
            }
        }
    }

    // Собираем тело цикла (все строки кроме первой и последней)
    // Нужно правильно обрабатывать вложенные циклы, try блоки и if блоки
    let mut body_lines: Vec<&str> = Vec::new();
    let mut var_stack: Vec<String> = Vec::new();
    let mut try_depth = 0;
    let mut if_depth = 0;
    
    if debug {
        eprintln!("🔍 DEBUG: Starting body collection for loop with {} lines", for_lines.len());
        for (idx, line) in for_lines.iter().enumerate() {
            eprintln!("  Line {}: {}", idx, line);
        }
    }
    
    for i in 1..for_lines.len() {
        let line = for_lines[i].trim();

        // Обрабатываем вложенные try блоки
        if line == "try" {
            try_depth += 1;
            body_lines.push(for_lines[i]);
        } else if line == "endtry" {
            if try_depth > 0 {
                try_depth -= 1;
            }
            body_lines.push(for_lines[i]);
        } else if line.starts_with("catch") && try_depth > 0 {
            // catch внутри try блока - добавляем и пропускаем проверку
            body_lines.push(for_lines[i]);
        } else if line.starts_with("if ") && (line.contains(" do") || line.contains(" then")) {
            if_depth += 1;
            body_lines.push(for_lines[i]);
        } else if line == "endif" || line == "endeif" {
            if if_depth > 0 {
                if_depth -= 1;
            }
            body_lines.push(for_lines[i]);
        } else if try_depth > 0 || if_depth > 0 {
            // Мы внутри try/catch или if/endif блока - просто добавляем строку
            body_lines.push(for_lines[i]);
        } else if line.starts_with("for ") && line.ends_with(" do") {
            // Новый вложенный цикл
            if debug {
                eprintln!("🔍 DEBUG: Found nested for loop in body collection: {}", line);
            }
            if let Some(var_name) = parse_for_variable(line) {
                if debug {
                    eprintln!("  Pushing to var_stack: {}", var_name);
                }
                var_stack.push(var_name);
            }
            body_lines.push(for_lines[i]);
        } else if let Some(next_var) = parse_next_variable(line) {
            if debug {
                eprintln!("🔍 DEBUG: Found next statement: {}", next_var);
                eprintln!("  var_stack = {:?}, variables[0] = {}", var_stack, variables[0]);
            }
            if var_stack.is_empty() {
                // Нет вложенных циклов, проверяем что это next для нашего цикла
                // Для множественных переменных проверяем первую переменную
                let next_first_var = next_var.split(',').next().unwrap_or(&next_var).trim();
                if next_first_var == variables[0] {
                    // Это next для нашего цикла - заканчиваем сбор тела
                    if debug {
                        eprintln!("  ✅ This is next for our loop, breaking");
                    }
                    break;
                } else {
                    // Неправильный next - возможно это next для цикла, который мы еще не обработали
                    // Добавляем в тело и продолжаем (это может быть next для вложенного цикла, который мы пропустили)
                    if debug {
                        eprintln!("  ⚠️ Unexpected next '{}', adding to body and continuing", next_var);
                    }
                    body_lines.push(for_lines[i]);
                    // Не прерываем сбор - продолжаем до правильного next
                }
            } else {
                // Есть вложенные циклы
                if let Some(last_var) = var_stack.last() {
                    let next_first_var = next_var.split(',').next().unwrap_or(&next_var).trim();
                    if next_first_var == last_var {
                        // Это next для вложенного цикла
                        if debug {
                            eprintln!("  ✅ This is next for nested loop: {}", last_var);
                        }
                        var_stack.pop();
                        body_lines.push(for_lines[i]);
                    } else {
                        // Неправильный next - возможно это next для нашего цикла
                        let next_first_var = next_var.split(',').next().unwrap_or(&next_var).trim();
                        if next_first_var == variables[0] {
                            // Это next для нашего цикла - заканчиваем сбор тела
                            if debug {
                                eprintln!("  ✅ This is next for our loop, breaking");
                            }
                            break;
                        } else {
                            // Неправильный next - добавляем в тело и продолжаем
                            if debug {
                                eprintln!("  ⚠️ Wrong next, adding to body");
                            }
                            body_lines.push(for_lines[i]);
                        }
                    }
                } else {
                    body_lines.push(for_lines[i]);
                }
            }
        } else {
            body_lines.push(for_lines[i]);
        }
    }

    // НЕ создаём scope здесь - он будет создаваться для каждой итерации

    let result = match iterable_value {
        Value::Array(ref arr) => {
            if debug {
                eprintln!("🔍 DEBUG: Processing array with {} elements, variables count = {}", arr.len(), variables.len());
            }
            if variables.len() > 1 {
                // Множественные переменные - деструктуризация элементов массива
                // Проверяем, является ли сам массив подходящим для деструктуризации
                // (если длина массива равна количеству переменных, это может быть одна итерация)
                // Но обычно мы итерируемся по элементам массива, где каждый элемент - массив для деструктуризации
                
                // Проверяем первый элемент, чтобы понять структуру
                if arr.is_empty() {
                    // Пустой массив - нет итераций
                    if debug {
                        eprintln!("🔍 DEBUG: Array is empty, no iterations");
                    }
                    Ok(())
                } else if let Some(first_item) = arr.first() {
                    if debug {
                        eprintln!("🔍 DEBUG: First item type = {:?}", 
                            match first_item {
                                Value::Array(_) => "Array",
                                _ => "Not Array",
                            }
                        );
                    }
                    match first_item {
                        Value::Array(_) => {
                            if debug {
                                eprintln!("✅ DEBUG: First item is array, iterating over array elements");
                            }
                            // Элементы массива - это массивы для деструктуризации
                            // Итерируемся по элементам
                            for (iter_idx, item) in arr.iter().enumerate() {
                                if debug {
                                    eprintln!("🔍 DEBUG: Iteration {}: item = {:?}", iter_idx, item);
                                }
                                // Создаём новый scope для этой итерации
                                interpreter.enter_loop_scope();
                                
                                // Проверяем, является ли элемент массивом для деструктуризации
                                let item_arr = match item {
                                    Value::Array(ref item_arr) => {
                                        if debug {
                                            eprintln!("  Item is array with length {}, variables count = {}", item_arr.len(), variables.len());
                                        }
                                        // Элемент - массив, проверяем длину
                                        if item_arr.len() != variables.len() {
                                            interpreter.exit_loop_scope();
                                            return Err(DataCodeError::runtime_error(
                                                &format!("Cannot unpack array of length {} into {} variables", item_arr.len(), variables.len()),
                                                interpreter.current_line
                                            ));
                                        }
                                        item_arr
                                    }
                                    _ => {
                                        // Элемент не массив - ошибка
                                        interpreter.exit_loop_scope();
                                        return Err(DataCodeError::runtime_error(
                                            &format!("Cannot unpack non-array value into {} variables", variables.len()),
                                            interpreter.current_line
                                        ));
                                    }
                                };
                                
                                // Устанавливаем все переменные из массива
                                if debug {
                                    eprintln!("🔍 DEBUG: Setting {} variables from array", variables.len());
                                }
                                for (i, var_name) in variables.iter().enumerate() {
                                    let value = item_arr[i].clone();
                                    if debug {
                                        eprintln!("  Setting variable '{}' = {:?}", var_name, value);
                                    }
                                    interpreter.set_loop_variable(var_name.to_string(), value);
                                    // Проверяем, что переменная установлена
                                    if debug {
                                        if let Some(set_value) = interpreter.get_variable(var_name) {
                                            eprintln!("  ✅ Variable '{}' is now set to {:?}", var_name, set_value);
                                        } else {
                                            eprintln!("  ❌ Variable '{}' is NOT set after set_loop_variable!", var_name);
                                        }
                                    }
                                }

                                // Выполняем тело цикла
                                if debug {
                                    eprintln!("🔍 DEBUG: Executing body of loop with {} lines", body_lines.len());
                                    for (idx, line) in body_lines.iter().enumerate() {
                                        eprintln!("  Body line {}: {}", idx, line);
                                    }
                                }
                                execute_block_directly(interpreter, &body_lines)?;

                                // Удаляем scope этой итерации
                                interpreter.exit_loop_scope();

                                // Проверяем return, break или continue
                                if interpreter.return_value.is_some() || interpreter.break_requested {
                                    if interpreter.break_requested {
                                        interpreter.break_requested = false; // Сбрасываем флаг break
                                    }
                                    break;
                                }
                                
                                // Проверяем continue (пропуск итерации)
                                if interpreter.continue_requested {
                                    interpreter.continue_requested = false; // Сбрасываем флаг continue
                                    continue; // Пропускаем текущую итерацию и переходим к следующей
                                }
                            }
                            Ok(())
                        }
                        _ => {
                            // Первый элемент не массив - возможно, сам массив нужно деструктурировать
                            // Но это не цикл, это одна итерация
                            let debug = std::env::var("DEBUG_FOR_LOOP").is_ok();
                            if debug {
                                eprintln!("🔍 DEBUG: First element is not array, checking direct unpacking");
                                eprintln!("  arr.len() = {}, variables.len() = {}", arr.len(), variables.len());
                                eprintln!("  variables = {:?}", variables);
                            }
                            
                            if arr.len() == variables.len() {
                                if debug {
                                    eprintln!("✅ DEBUG: Array length matches variables, unpacking directly");
                                }
                                // Деструктурируем сам массив
                                interpreter.enter_loop_scope();
                                
                                // Устанавливаем все переменные из массива
                                for (i, var_name) in variables.iter().enumerate() {
                                    let value = arr[i].clone();
                                    if debug {
                                        eprintln!("  Setting variable '{}' = {:?}", var_name, value);
                                    }
                                    interpreter.set_loop_variable(var_name.to_string(), value);
                                }

                                // Выполняем тело цикла
                                let result = execute_block_directly(interpreter, &body_lines);
                                
                                // Удаляем scope этой итерации
                                interpreter.exit_loop_scope();
                                
                                // Проверяем continue (пропуск итерации) - если установлен, просто возвращаем Ok(())
                                // Это одноразовый блок, так что continue здесь не имеет смысла
                                if interpreter.continue_requested {
                                    interpreter.continue_requested = false; // Сбрасываем флаг continue
                                    // В одноразовом блоке continue просто завершает выполнение
                                    return Ok(());
                                }
                                
                                result
                            } else {
                                // Массив не подходит для прямой деструктуризации
                                // Попробуем итерироваться по элементам, если они массивы
                                // Но это не должно происходить, так как первый элемент не массив
                                Err(DataCodeError::runtime_error(
                                    &format!("Cannot unpack array of length {} into {} variables. For iteration over array elements, each element must be an array.", arr.len(), variables.len()),
                                    interpreter.current_line
                                ))
                            }
                        }
                    }
                } else {
                    Ok(())
                }
            } else {
                // Обычная итерация по элементам массива (одна переменная)
                for item in arr {
                    // Создаём новый scope для этой итерации
                    interpreter.enter_loop_scope();
                    
                    // Простое присваивание
                    interpreter.set_loop_variable(variables[0].to_string(), item.clone());

                    // Выполняем тело цикла
                    execute_block_directly(interpreter, &body_lines)?;

                    // Удаляем scope этой итерации
                    interpreter.exit_loop_scope();

                    // Проверяем return или break
                    if interpreter.return_value.is_some() || interpreter.break_requested {
                        if interpreter.break_requested {
                            interpreter.break_requested = false; // Сбрасываем флаг break
                        }
                        break;
                    }
                    
                    // Проверяем continue (пропуск итерации)
                    if interpreter.continue_requested {
                        interpreter.continue_requested = false; // Сбрасываем флаг continue
                        continue; // Пропускаем текущую итерацию и переходим к следующей
                    }
                }
                Ok(())
            }
        }
        Value::String(ref s) => {
            for ch in s.chars() {
                // Создаём новый scope для этой итерации
                interpreter.enter_loop_scope();
                
                interpreter.set_loop_variable(variables[0].to_string(), Value::String(ch.to_string()));

                // Выполняем тело цикла
                execute_block_directly(interpreter, &body_lines)?;

                // Удаляем scope этой итерации
                interpreter.exit_loop_scope();

                // Проверяем return или break
                if interpreter.return_value.is_some() || interpreter.break_requested {
                    if interpreter.break_requested {
                        interpreter.break_requested = false; // Сбрасываем флаг break
                    }
                    break;
                }
                
                // Проверяем continue (пропуск итерации)
                if interpreter.continue_requested {
                    interpreter.continue_requested = false; // Сбрасываем флаг continue
                    continue; // Пропускаем текущую итерацию и переходим к следующей
                }
            }
            Ok(())
        }
        Value::Table(ref table) => {
            let table_borrowed = table.borrow();
            for row in &table_borrowed.rows {
                // Создаём новый scope для этой итерации
                interpreter.enter_loop_scope();
                
                if variables.len() > 1 {
                    // Деструктуризация строки таблицы в переменные
                    if row.len() != variables.len() {
                        interpreter.exit_loop_scope();
                        return Err(DataCodeError::runtime_error(
                            &format!("Cannot unpack table row of length {} into {} variables", row.len(), variables.len()),
                            interpreter.current_line
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

                // Выполняем тело цикла
                execute_block_directly(interpreter, &body_lines)?;

                // Удаляем scope этой итерации
                interpreter.exit_loop_scope();

                // Проверяем return или break
                if interpreter.return_value.is_some() || interpreter.break_requested {
                    if interpreter.break_requested {
                        interpreter.break_requested = false; // Сбрасываем флаг break
                    }
                    break;
                }
                
                // Проверяем continue (пропуск итерации)
                if interpreter.continue_requested {
                    interpreter.continue_requested = false; // Сбрасываем флаг continue
                    continue; // Пропускаем текущую итерацию и переходим к следующей
                }
            }
            Ok(())
        }
        Value::Object(ref obj) => {
            // Сортируем ключи для предсказуемого порядка итерации
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();

            for key in keys {
                if let Some(value) = obj.get(key) {
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
                            interpreter.current_line
                        ));
                    }

                    // Выполняем тело цикла
                    execute_block_directly(interpreter, &body_lines)?;

                    // Удаляем scope этой итерации
                    interpreter.exit_loop_scope();

                    // Проверяем return или break
                    if interpreter.return_value.is_some() || interpreter.break_requested {
                        if interpreter.break_requested {
                            interpreter.break_requested = false; // Сбрасываем флаг break
                        }
                        break;
                    }
                    
                    // Проверяем continue (пропуск итерации)
                    if interpreter.continue_requested {
                        interpreter.continue_requested = false; // Сбрасываем флаг continue
                        continue; // Пропускаем текущую итерацию и переходим к следующей
                    }
                }
            }
            Ok(())
        }
        _ => {
            // Уменьшаем счетчик при ошибке
            if interpreter.active_loop_count > 0 {
                interpreter.active_loop_count -= 1;
            }
            return Err(DataCodeError::runtime_error(
                &format!("Cannot iterate over {:?}", iterable_value),
                interpreter.current_line,
            ));
        },
    };

    // Уменьшаем счетчик активных циклов после завершения цикла
    if interpreter.active_loop_count > 0 {
        interpreter.active_loop_count -= 1;
    }

    result
}

/// Парсить и определить функцию напрямую без рекурсии
fn parse_and_define_function_directly(interpreter: &mut Interpreter, function_lines: &[&str]) -> Result<()> {
    if function_lines.is_empty() {
        return Err(DataCodeError::syntax_error("Empty function definition", interpreter.current_line, 0));
    }

    // Первая строка должна быть "function name(params) do", "global function name(params) do" или "local function name(params) do"
    let first_line = function_lines[0].trim();

    let (is_global, function_part) = if let Some(stripped) = first_line.strip_prefix("global function ") {
        (true, stripped)
    } else if let Some(stripped) = first_line.strip_prefix("local function ") {
        (false, stripped)
    } else if let Some(stripped) = first_line.strip_prefix("function ") {
        // По умолчанию функция глобальная, если не указан префикс
        (true, stripped)
    } else {
        return Err(DataCodeError::syntax_error("Invalid function definition", interpreter.current_line, 0));
    };

    if !function_part.ends_with(" do") {
        return Err(DataCodeError::syntax_error("Function definition must end with 'do'", interpreter.current_line, 0));
    }

    let function_signature = function_part.strip_suffix(" do").unwrap();

    // Парсим имя функции и параметры
    let (function_name, parameters) = if let Some(paren_pos) = function_signature.find('(') {
        let name = function_signature[..paren_pos].trim();
        let params_part = &function_signature[paren_pos..];

        if !params_part.ends_with(')') {
            return Err(DataCodeError::syntax_error("Missing closing parenthesis in function definition", interpreter.current_line, 0));
        }

        let params_str = &params_part[1..params_part.len()-1]; // Убираем скобки
        let parameters: Vec<String> = if params_str.trim().is_empty() {
            Vec::new()
        } else {
            params_str.split(',').map(|p| p.trim().to_string()).collect()
        };

        (name.to_string(), parameters)
    } else {
        return Err(DataCodeError::syntax_error("Missing parentheses in function definition", interpreter.current_line, 0));
    };

    // Собираем тело функции (все строки кроме первой и последней)
    let mut body_lines = Vec::new();
    for i in 1..function_lines.len() {
        let line = function_lines[i].trim();
        if line == "endfunction" {
            break;
        }
        body_lines.push(function_lines[i].to_string());
    }

    // Создаем и добавляем функцию
    let function = crate::interpreter::user_functions::UserFunction {
        name: function_name.clone(),
        parameters,
        body: body_lines,
        _is_global: is_global,
    };

    interpreter.function_manager.add_function(function);
    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_assignment() {
        let mut interp = Interpreter::new();
        
        let result = execute_line(&mut interp, "global x = 42");
        assert!(result.is_ok());
        assert_eq!(interp.get_variable("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_execute_expression() {
        let mut interp = Interpreter::new();
        
        // Устанавливаем переменную
        interp.set_variable("x".to_string(), Value::Number(10.0), true);
        
        // Выполняем выражение
        let result = execute_line(&mut interp, "x + 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_value_for_print() {
        assert_eq!(format_value_for_print(&Value::Number(42.0)), "42");
        assert_eq!(format_value_for_print(&Value::Number(42.5)), "42.5");
        assert_eq!(format_value_for_print(&Value::String("hello".to_string())), "hello");
        assert_eq!(format_value_for_print(&Value::Bool(true)), "true");
        assert_eq!(format_value_for_print(&Value::Null), "null");
    }

    #[test]
    fn test_to_bool() {
        assert_eq!(to_bool(&Value::Bool(true)), true);
        assert_eq!(to_bool(&Value::Bool(false)), false);
        assert_eq!(to_bool(&Value::Number(1.0)), true);
        assert_eq!(to_bool(&Value::Number(0.0)), false);
        assert_eq!(to_bool(&Value::String("hello".to_string())), true);
        assert_eq!(to_bool(&Value::String("".to_string())), false);
        assert_eq!(to_bool(&Value::Null), false);
    }
}

/// Обработать throw statement
fn handle_throw_statement(interpreter: &mut Interpreter, code: &str) -> Result<()> {
    let expression = code.strip_prefix("throw ").unwrap().trim();
    let value = eval_expr_safe(interpreter, expression)?;

    let message = match value {
        Value::String(s) => s,
        _ => format_value_for_print(&value),
    };

    Err(DataCodeError::user_exception(&message, interpreter.current_line))
}



/// Выполнить try/catch/finally блок напрямую с поддержкой стека исключений
fn execute_try_statement_directly(interpreter: &mut Interpreter, try_lines: &[&str]) -> Result<()> {

    if try_lines.is_empty() {
        return Ok(());
    }

    // Парсим структуру try/catch/finally блока
    let mut try_body = Vec::new();
    let mut catch_body = Vec::new();
    let mut finally_body = Vec::new();
    let mut catch_var: Option<String> = None;

    let mut current_section = "try";
    let mut i = 1; // Пропускаем первую строку "try"
    let mut try_nesting_level = 0; // Отслеживаем уровень вложенности try блоков

    while i < try_lines.len() {
        let line = try_lines[i].trim();

        // Отслеживаем вложенные try блоки
        if line == "try" {
            try_nesting_level += 1;
        } else if line == "endtry" {
            if try_nesting_level > 0 {
                try_nesting_level -= 1;
            } else {
                // Это наш endtry, выходим
                break;
            }
        } else if line.starts_with("catch") && try_nesting_level == 0 {
            // Это наш catch блок (не вложенный)
            current_section = "catch";
            // Парсим переменную catch (если есть)
            if line.len() > 5 {
                let catch_part = line[5..].trim();
                if !catch_part.is_empty() {
                    catch_var = Some(catch_part.to_string());
                }
            }
        } else if line == "finally" && try_nesting_level == 0 {
            // Это наш finally блок (не вложенный)
            current_section = "finally";
        } else {
            // Добавляем строку в соответствующую секцию
            match current_section {
                "try" => try_body.push(try_lines[i]),
                "catch" => catch_body.push(try_lines[i]),
                "finally" => finally_body.push(try_lines[i]),
                _ => {}
            }
        }

        // Также добавляем вложенные try/endtry в try body
        if current_section == "try" && (line == "try" || line == "endtry") && try_nesting_level > 0 {
            try_body.push(try_lines[i]);
        }

        i += 1;
    }

    // Создаем блок try/catch и добавляем его в стек
    let block_id = interpreter.get_next_try_block_id();
    let nesting_level = interpreter.get_try_nesting_level();

    let try_block = crate::interpreter::user_functions::TryBlock::new(
        catch_var.clone(),
        catch_body.iter().map(|s| s.to_string()).collect(),
        if finally_body.is_empty() { None } else { Some(finally_body.iter().map(|s| s.to_string()).collect()) },
        block_id,
        nesting_level,
    );

    interpreter.push_try_block(try_block);

    // Создаем область видимости для try-catch блока (для локальных переменных)
    // Это нужно сделать ДО выполнения try блока, чтобы локальные переменные из try попадали в эту область
    interpreter.enter_loop_scope();

    // Выполняем try блок с поддержкой вложенных try/catch
    let try_result = execute_block_with_try_support(interpreter, &try_body);

    // Обрабатываем результат выполнения try блока
    let mut final_error = None;

    if let Err(error) = try_result {
        // Если у текущего блока есть catch секция, обрабатываем исключение локально
        if !catch_body.is_empty() {
            // Устанавливаем переменную ошибки, если указана
            // Теперь она попадет в loop_stack, так как мы уже вошли в область видимости
            if let Some(var_name) = &catch_var {
                let error_message = match &error {
                    DataCodeError::UserException { message, .. } => message.clone(),
                    _ => format!("{}", error),
                };
                interpreter.set_variable(var_name.clone(), Value::String(error_message), false);
            }

            // Выполняем catch блок с поддержкой вложенных try/catch
            if let Err(catch_err) = execute_block_with_try_support(interpreter, &catch_body) {
                // Исключение из catch блока пробрасываем дальше для обработки внешними блоками
                final_error = Some(catch_err);
            }
        } else {
            // Нет catch блока в текущем try - пробрасываем исключение
            final_error = Some(error);
        }
    }

    // Удаляем текущий блок из стека
    interpreter.pop_try_block();

    // Всегда выполняем finally блок (если есть)
    if !finally_body.is_empty() {
        if let Err(finally_err) = execute_block_with_try_support(interpreter, &finally_body) {
            // Ошибка в finally блоке имеет приоритет
            final_error = Some(finally_err);
        }
    }

    // Выходим из области видимости try-catch блока (очищаем локальные переменные)
    // Это удалит переменную e и другие локальные переменные из catch блока
    interpreter.exit_loop_scope();

    // Если есть ошибка, пробрасываем ее дальше для обработки внешними блоками
    if let Some(error) = final_error {
        return Err(error);
    }

    Ok(())
}

/// Выполнить блок кода с поддержкой вложенных try/catch блоков
fn execute_block_with_try_support(interpreter: &mut Interpreter, lines: &[&str]) -> Result<()> {
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        interpreter.current_line += 1;

        // Пропускаем пустые строки и комментарии
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Пропускаем next, если он уже был обработан в цикле for
        // Это нужно для случаев, когда цикл for находится внутри try блока
        if let Some(_) = parse_next_variable(line) {
            i += 1;
            continue;
        }

        // Обрабатываем циклы for ПЕРЕД try блоками (важно для правильного порядка)
        if line.starts_with("for ") && line.ends_with(" do") {
            // Находим соответствующий next с правильным учетом вложенности
            let mut for_lines = vec![lines[i]];
            
            // Извлекаем имя переменной текущего цикла
            let current_var = parse_for_variable(line)
                .ok_or_else(|| DataCodeError::syntax_error("Invalid for loop syntax", interpreter.current_line, 0))?;
            
            // Стек переменных для отслеживания вложенных циклов
            let mut var_stack: Vec<String> = vec![current_var.clone()];
            // Начинаем со следующей строки после "for ... do"
            let mut j = i + 1;
            
            // Отслеживаем вложенные try/catch и if/endif блоки
            let mut try_depth = 0;
            let mut if_depth = 0;

            while j < lines.len() && !var_stack.is_empty() {
                let current_line = lines[j].trim();

                // Пропускаем пустые строки и комментарии
                if current_line.is_empty() || current_line.starts_with('#') {
                    for_lines.push(lines[j]);
                    j += 1;
                    continue;
                }

                // Обрабатываем вложенные try/catch блоки
                if current_line == "try" {
                    try_depth += 1;
                    for_lines.push(lines[j]);
                    j += 1;
                    continue;
                } else if current_line == "endtry" {
                    if try_depth > 0 {
                        try_depth -= 1;
                    }
                    for_lines.push(lines[j]);
                    j += 1;
                    continue;
                } else if current_line.starts_with("catch") && try_depth > 0 {
                    // catch внутри try блока - добавляем и пропускаем проверку
                    for_lines.push(lines[j]);
                    j += 1;
                    continue;
                }
                
                // Обрабатываем вложенные if/endif блоки
                if current_line.starts_with("if ") && (current_line.contains(" do") || current_line.contains(" then")) {
                    if_depth += 1;
                } else if current_line == "endif" || current_line == "endeif" {
                    if if_depth > 0 {
                        if_depth -= 1;
                    }
                }

                // Проверяем на вложенные циклы даже внутри try/catch или if/endif блоков
                if current_line.starts_with("for ") && current_line.ends_with(" do") {
                    // Новый вложенный цикл - добавляем в стек
                    if let Some(var_name) = parse_for_variable(current_line) {
                        var_stack.push(var_name);
                    }
                    // Добавляем строку for в for_lines
                    for_lines.push(lines[j]);
                    j += 1;
                    continue;
                }

                // Если мы внутри try/catch или if/endif блока, проверяем next только для вложенных циклов
                if try_depth > 0 || if_depth > 0 {
                    // Проверяем next только для вложенных циклов (когда стек не пуст)
                    if let Some(next_var) = parse_next_variable(current_line) {
                        let next_first_var = next_var.split(',').next().unwrap_or(&next_var).trim();
                        if let Some(last_var) = var_stack.last() {
                            if next_first_var == *last_var || next_var == *last_var {
                                // Это next для вложенного цикла - удаляем из стека
                                var_stack.pop();
                                // Добавляем next в for_lines
                                for_lines.push(lines[j]);
                            } else {
                                // Неправильное имя переменной в next
                                return Err(DataCodeError::syntax_error(
                                    &format!("Mismatched next: expected 'next {}' but found 'next {}'", last_var, next_var),
                                    interpreter.current_line,
                                    0
                                ));
                            }
                        } else {
                            // next найден, но стек пуст - это может быть next для внешнего цикла
                            // но мы внутри try/if блока, поэтому просто добавляем строку
                            for_lines.push(lines[j]);
                        }
                    } else {
                        // Обычная строка внутри try/if блока
                        for_lines.push(lines[j]);
                    }
                    j += 1;
                    continue;
                }

                // Проверяем на next только когда мы НЕ внутри других блоков
                if let Some(next_var) = parse_next_variable(current_line) {
                    // Нашли next - проверяем, соответствует ли он последнему циклу
                    // Для множественных переменных берем первую переменную из next
                    let next_first_var = next_var.split(',').next().unwrap_or(&next_var).trim();
                    if let Some(last_var) = var_stack.last() {
                        if next_first_var == *last_var || next_var == *last_var {
                            // Это next для вложенного цикла - удаляем из стека
                            var_stack.pop();
                            // Добавляем next в for_lines
                            for_lines.push(lines[j]);
                        } else {
                            // Неправильное имя переменной в next
                            return Err(DataCodeError::syntax_error(
                                &format!("Mismatched next: expected 'next {}' but found 'next {}'", last_var, next_var),
                                interpreter.current_line,
                                0
                            ));
                        }
                    } else {
                        // next найден, но стек пуст - это может быть next для внешнего цикла
                        // но мы не знаем переменную внешнего цикла в этом контексте
                        // просто добавляем строку и продолжаем (это обработается на более высоком уровне)
                        for_lines.push(lines[j]);
                    }
                } else {
                    // Добавляем строку
                    for_lines.push(lines[j]);
                }

                // Если стек пуст, мы нашли все next'ы для всех циклов
                if var_stack.is_empty() {
                    break;
                }

                j += 1;
            }

            if !var_stack.is_empty() {
                return Err(DataCodeError::syntax_error(
                    &format!("Missing 'next {}' in for loop", var_stack[0]),
                    interpreter.current_line,
                    0
                ));
            }

            // Выполняем for цикл
            execute_for_loop_iteratively(interpreter, &for_lines)?;
            i = j + 1;  // Пропускаем строку next, которая уже была обработана
            continue;
        }

        // Обрабатываем try блоки
        if line.trim() == "try" {
            // Находим соответствующий endtry
            let mut try_lines = Vec::new();
            let mut j = i;
            let mut try_count = 0;

            while j < lines.len() {
                let current_line = lines[j].trim();
                try_lines.push(current_line);


                if current_line == "try" {
                    try_count += 1;
                } else if current_line == "endtry" {
                    try_count -= 1;
                    if try_count == 0 {
                        break;
                    }
                }
                j += 1;
            }



            // Выполняем try/catch блок рекурсивно
            // НЕ используем ? чтобы ошибки могли быть пойманы внешними try блоками
            if let Err(e) = execute_try_statement_directly(interpreter, &try_lines) {
                return Err(e);
            }

            i = j + 1;
            continue;
        }

        // Пропускаем next, если он уже был обработан в цикле for
        if let Some(_) = parse_next_variable(line) {
            i += 1;
            continue;
        }

        // Обрабатываем условные конструкции if
        if line.starts_with("if ") && (line.contains(" do") || line.contains(" then")) {
            // Собираем всю условную конструкцию
            let mut if_lines = vec![lines[i]];
            let mut if_depth = 1;
            let mut j = i + 1;

            while j < lines.len() && if_depth > 0 {
                let current_line = lines[j].trim();

                if current_line.starts_with("if ") && (current_line.contains(" do") || current_line.contains(" then")) {
                    if_depth += 1;
                } else if current_line == "endif" || current_line == "endeif" {
                    if_depth -= 1;
                }

                if_lines.push(lines[j]);

                if if_depth == 0 {
                    break;
                }
                j += 1;
            }

            // Выполняем условную конструкцию итеративно
            execute_if_statement_iteratively(interpreter, &if_lines)?;
            i = j + 1;
            continue;
        }

        // Проверяем на многострочные присваивания
        if is_incomplete_assignment(line) {
            // Обрабатываем многострочные присваивания
            i = handle_multiline_assignment_in_try_block(interpreter, lines, i)?;
        } else {
            // Выполняем обычную строку кода (используем safe версию, чтобы игнорировать next и другие блочные ключевые слова)
            execute_line_simple_safe(interpreter, line)?;
        }
        i += 1;
    }
    Ok(())
}

/// Проверить, является ли строка неполным присваиванием (содержит незакрытые скобки)
fn is_incomplete_assignment(line: &str) -> bool {
    // Проверяем, что это присваивание
    // Должно начинаться с global/local И содержать =
    // Для многострочных присваиваний первая строка может быть обрезана, поэтому проверяем trimmed версию
    let trimmed = line.trim();
    let is_declaration = trimmed.starts_with("global ") || trimmed.starts_with("local ");
    if !is_declaration || !line.contains('=') {
        if std::env::var("DATACODE_DEBUG_PARSE").is_ok() && (trimmed.starts_with("global ") || trimmed.starts_with("local ")) {
            eprintln!("🔍 DEBUG: is_incomplete_assignment('{}'): is_declaration={}, contains='='={}", 
                line, is_declaration, line.contains('='));
        }
        return false;
    }
    
    if std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
        eprintln!("🔍 DEBUG: Checking incomplete assignment: '{}'", line);
    }

    // Подсчитываем открытые и закрытые скобки
    let mut bracket_count = 0;
    let mut paren_count = 0;
    let mut brace_count = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in line.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' | '\'' => in_string = !in_string,
            '[' if !in_string => bracket_count += 1,
            ']' if !in_string => bracket_count -= 1,
            '(' if !in_string => paren_count += 1,
            ')' if !in_string => paren_count -= 1,
            '{' if !in_string => brace_count += 1,
            '}' if !in_string => brace_count -= 1,
            _ => {}
        }
    }

    // Если есть незакрытые скобки, это неполное присваивание
    let is_incomplete = bracket_count > 0 || paren_count > 0 || brace_count > 0;
    
    if std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
        eprintln!("🔍 DEBUG: is_incomplete_assignment result: bracket_count={}, paren_count={}, brace_count={}, is_incomplete={}", 
            bracket_count, paren_count, brace_count, is_incomplete);
    }
    
    is_incomplete
}

/// Обработать многострочное присваивание
fn handle_multiline_assignment(interpreter: &mut Interpreter, lines: &[&str], start_index: usize) -> Result<usize> {
    let mut assignment_lines = vec![lines[start_index]];
    let mut i = start_index + 1;

    // Собираем строки до тех пор, пока присваивание не станет полным
    while i < lines.len() {
        assignment_lines.push(lines[i]);

        // Объединяем все строки и проверяем, полное ли присваивание
        let combined = assignment_lines.join("\n");
        if !is_incomplete_assignment(&combined) {
            break;
        }

        i += 1;
    }

    // Выполняем полное многострочное присваивание
    let combined_assignment = assignment_lines.join("\n");
    execute_line_simple(interpreter, &combined_assignment)?;

    Ok(i)
}

/// Обработать многострочное присваивание в try блоке
fn handle_multiline_assignment_in_try_block(interpreter: &mut Interpreter, lines: &[&str], start_index: usize) -> Result<usize> {
    let mut assignment_lines = vec![lines[start_index]];
    let mut i = start_index + 1;

    // Собираем строки до тех пор, пока присваивание не станет полным
    while i < lines.len() {
        assignment_lines.push(lines[i]);

        // Объединяем все строки и проверяем, полное ли присваивание
        let combined = assignment_lines.join("\n");
        if !is_incomplete_assignment(&combined) {
            break;
        }

        i += 1;
    }

    // Выполняем полное многострочное присваивание
    let combined_assignment = assignment_lines.join("\n");
    execute_line_simple(interpreter, &combined_assignment)?;

    Ok(i)
}
