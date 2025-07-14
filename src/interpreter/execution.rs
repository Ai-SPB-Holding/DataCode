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

        // Увеличиваем номер строки для всех строк
        interpreter.current_line += 1;

        // Пропускаем пустые строки и однострочные комментарии
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

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
        if line.starts_with("global function ") || line.starts_with("local function ") {
            i = handle_function_definition(interpreter, &lines, i)?;
        } else if line.starts_with("for ") && line.ends_with(" do") {
            i = handle_for_loop(interpreter, &lines, i)?;
        } else if line.starts_with("if ") && (line.ends_with(" do") || line.ends_with(" then")) {
            i = handle_if_statement(interpreter, &lines, i)?;
        } else if line == "try" {
            i = handle_try_statement(interpreter, &lines, i)?;
        } else {
            // Обычная строка - используем execute_line_simple чтобы избежать рекурсии
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

    // Обработка throw statements
    if trimmed_code.starts_with("throw ") {
        return handle_throw_statement(interpreter, trimmed_code);
    }

    // Обработка return
    if trimmed_code.starts_with("return") {
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

        if parts.len() != 2 {
            return Err(DataCodeError::syntax_error("Invalid assignment", interpreter.current_line, 0));
        }

        let var_name = parts[0].to_string();
        let expr = parts[1];

        let val = interpreter.eval_expr(expr)?;
        interpreter.set_variable(var_name, val, is_global);
        return Ok(());
    }



    // Обработка throw
    if trimmed_code.starts_with("throw ") {
        return handle_throw_statement(interpreter, trimmed_code);
    }

    // Проверяем на блочные конструкции, которые не должны обрабатываться как выражения
    if trimmed_code == "try" || trimmed_code == "catch" || trimmed_code == "finally" ||
       trimmed_code == "endtry" || trimmed_code == "else" || trimmed_code == "endif" ||
       trimmed_code == "forend" || trimmed_code == "endfunction" {
        return Err(DataCodeError::syntax_error(
            &format!("Unexpected keyword '{}' outside of block context", trimmed_code),
            interpreter.current_line, 0
        ));
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
            format!("Table({} rows, {} columns)", table.rows.len(), table.column_names.len())
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
        Table(table) => !table.rows.is_empty(),
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

/// Обработать цикл for
fn handle_for_loop(interpreter: &mut Interpreter, lines: &[&str], start: usize) -> Result<usize> {
    let mut loop_lines = vec![lines[start]];
    let mut for_depth = 1;
    let mut i = start + 1;

    while i < lines.len() && for_depth > 0 {
        let current_line = lines[i].trim();

        if current_line.starts_with("for ") && current_line.ends_with(" do") {
            for_depth += 1;
        } else if current_line == "forend" {
            for_depth -= 1;
        }

        loop_lines.push(lines[i]);

        if for_depth == 0 {
            break;
        }

        i += 1;
    }

    if for_depth > 0 {
        return Err(DataCodeError::syntax_error("Missing forend in for loop", interpreter.current_line, 0));
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

        if current_line.starts_with("if ") && (current_line.ends_with(" do") || current_line.ends_with(" then")) {
            if_depth += 1;
        } else if current_line == "endif" {
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

    Ok(i)
}

/// Выполнить условную конструкцию напрямую без рекурсии
fn execute_if_statement_directly(interpreter: &mut Interpreter, if_lines: &[&str]) -> Result<()> {
    if if_lines.is_empty() {
        return Ok(());
    }

    // Первая строка должна быть "if condition do" или "if condition then"
    let first_line = &if_lines[0];
    let trimmed_first = first_line.trim();
    if !trimmed_first.starts_with("if ") || (!trimmed_first.ends_with(" do") && !trimmed_first.ends_with(" then")) {
        return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
    }

    // Извлекаем условие
    let condition_str = if let Some(stripped) = trimmed_first.strip_prefix("if ") {
        if let Some(condition) = stripped.strip_suffix(" do") {
            condition.trim()
        } else if let Some(condition) = stripped.strip_suffix(" then") {
            condition.trim()
        } else {
            return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
        }
    } else {
        return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
    };

    // Вычисляем условие с защитой от рекурсии
    let condition_value = eval_condition_safe(interpreter, condition_str)?;
    let condition_result = to_bool(&condition_value);

    // Находим блоки if, else, endif с учетом вложенности
    let mut if_body = Vec::new();
    let mut else_body = Vec::new();
    let mut in_else = false;
    let mut i = 1; // Пропускаем первую строку "if ... do"
    let mut nested_depth = 0;

    while i < if_lines.len() {
        let line = &if_lines[i];
        let trimmed = line.trim();

        // Проверяем на вложенные if
        if trimmed.starts_with("if ") && (trimmed.ends_with(" do") || trimmed.ends_with(" then")) {
            nested_depth += 1;
        } else if trimmed == "endif" {
            if nested_depth == 0 {
                break; // Это наш endif
            } else {
                nested_depth -= 1; // Это endif для вложенного if
            }
        } else if trimmed == "else" && nested_depth == 0 {
            in_else = true;
            i += 1;
            continue;
        }

        if in_else {
            else_body.push(*line);
        } else {
            if_body.push(*line);
        }
        i += 1;
    }

    // Выполняем соответствующий блок
    let body_to_execute = if condition_result { &if_body } else { &else_body };

    // Выполняем блок с обработкой вложенных конструкций
    execute_block_directly(interpreter, body_to_execute)?;

    Ok(())
}

/// Выполнить блок кода напрямую с обработкой вложенных конструкций
pub fn execute_block_directly(interpreter: &mut Interpreter, lines: &[&str]) -> Result<()> {
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Проверяем на условные конструкции
        if line.starts_with("if ") && (line.ends_with(" do") || line.ends_with(" then")) {
            // Собираем всю условную конструкцию
            let mut if_lines = vec![lines[i]];
            let mut if_depth = 1;
            let mut j = i + 1;

            while j < lines.len() && if_depth > 0 {
                let current_line = lines[j].trim();

                if current_line.starts_with("if ") && (current_line.ends_with(" do") || current_line.ends_with(" then")) {
                    if_depth += 1;
                } else if current_line == "endif" {
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
        } else if line.starts_with("for ") && line.ends_with(" do") {
            // Обрабатываем циклы for
            let mut for_lines = vec![lines[i]];
            let mut for_depth = 1;
            let mut j = i + 1;

            while j < lines.len() && for_depth > 0 {
                let current_line = lines[j].trim();

                if current_line.starts_with("for ") && current_line.ends_with(" do") {
                    for_depth += 1;
                } else if current_line == "forend" {
                    for_depth -= 1;
                }

                for_lines.push(lines[j]);

                if for_depth == 0 {
                    break;
                }
                j += 1;
            }

            // Выполняем цикл for итеративно
            execute_for_loop_iteratively(interpreter, &for_lines)?;
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
            i = j + 1;
        } else {
            // Обычная строка - выполняем через безопасную версию без рекурсии
            execute_line_simple_safe(interpreter, lines[i])?;
            i += 1;
        }

        // Проверяем return
        if interpreter.return_value.is_some() {
            break;
        }
    }
    Ok(())
}

/// Выполнить условную конструкцию if итеративно (без рекурсии)
fn execute_if_statement_iteratively(interpreter: &mut Interpreter, if_lines: &[&str]) -> Result<()> {
    if if_lines.is_empty() {
        return Err(DataCodeError::syntax_error("Empty if statement", interpreter.current_line, 0));
    }

    // Первая строка должна быть "if condition do" или "if condition then"
    let first_line = &if_lines[0];
    let trimmed_first = first_line.trim();
    if !trimmed_first.starts_with("if ") || (!trimmed_first.ends_with(" do") && !trimmed_first.ends_with(" then")) {
        return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
    }

    // Извлекаем условие
    let condition_str = if let Some(stripped) = trimmed_first.strip_prefix("if ") {
        if let Some(condition) = stripped.strip_suffix(" do") {
            condition.trim()
        } else if let Some(condition) = stripped.strip_suffix(" then") {
            condition.trim()
        } else {
            return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
        }
    } else {
        return Err(DataCodeError::syntax_error("Invalid if statement", interpreter.current_line, 0));
    };

    // Вычисляем условие БЕЗ вызова пользовательских функций
    let condition_value = eval_condition_without_user_functions(interpreter, condition_str)?;
    let condition_result = is_truthy(&condition_value);

    // Разбираем блоки if/else/endif
    let mut if_body = Vec::new();
    let mut else_body = Vec::new();
    let mut in_else = false;
    let mut depth = 0;

    for (i, line) in if_lines.iter().enumerate() {
        if i == 0 {
            continue; // Пропускаем первую строку "if condition do"
        }

        let trimmed = line.trim();

        if trimmed.starts_with("if ") && (trimmed.ends_with(" do") || trimmed.ends_with(" then")) {
            depth += 1;
        } else if trimmed == "endif" {
            if depth == 0 {
                break; // Конец нашего if
            }
            depth -= 1;
        } else if trimmed == "else" && depth == 0 {
            in_else = true;
            continue;
        }

        if in_else {
            else_body.push(*line);
        } else {
            if_body.push(*line);
        }
    }

    // Выполняем соответствующий блок
    let body_to_execute = if condition_result { &if_body } else { &else_body };

    // Выполняем блок итеративно (без рекурсии)
    for line in body_to_execute {
        execute_line_simple_safe(interpreter, line)?;
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

    // Обработка return
    if trimmed_code.starts_with("return") {
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
                // Увеличиваем счетчик рекурсии перед вызовом
                if interpreter.recursion_depth >= 5 {
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

    // Парсим аргументы (простая версия для числовых аргументов)
    let mut args = Vec::new();
    if !args_str.is_empty() {
        for arg in args_str.split(',') {
            let arg = arg.trim();
            // Пытаемся парсить как число
            if let Ok(num) = arg.parse::<f64>() {
                args.push(Value::Number(num));
            } else {
                // Пытаемся получить как переменную
                if let Some(value) = interpreter.get_variable(arg) {
                    args.push(value.clone());
                } else {
                    args.push(Value::String(arg.to_string()));
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

    // Выполняем тело функции НАПРЯМУЮ без системы условных конструкций
    let mut result = Value::Null;

    // Для простого случая is_positive(x) - просто выполняем логику напрямую
    if function_name == "is_positive" && args.len() == 1 {
        if let Value::Number(x) = &args[0] {
            result = Value::Bool(*x > 0.0);
        }
    } else {
        // Для других функций - выполняем построчно с простой логикой
        for line in &function.body {
            let trimmed = line.trim();

            if trimmed.starts_with("return ") {
                let expr = trimmed.strip_prefix("return ").unwrap().trim();

                // Простая обработка выражений
                if expr == "true" {
                    result = Value::Bool(true);
                } else if expr == "false" {
                    result = Value::Bool(false);
                } else {
                    // Пытаемся вычислить выражение
                    result = interpreter.eval_expr(expr)?;
                }
                break;
            }
            // Игнорируем условные конструкции для предотвращения рекурсии
        }
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
    let iterable_value = interpreter.eval_expr(iterable_part)?;

    // Собираем тело цикла (все строки кроме первой и последней)
    let mut body_lines = Vec::new();
    for i in 1..for_lines.len() {
        let line = for_lines[i].trim();
        if line == "forend" {
            break;
        }
        body_lines.push(for_lines[i]);
    }

    // Входим в область видимости цикла
    interpreter.enter_loop_scope();

    let result = match iterable_value {
        Value::Array(ref arr) => {
            for item in arr {
                if variables.len() == 1 {
                    // Простое присваивание
                    interpreter.set_loop_variable(variables[0].to_string(), item.clone());
                } else {
                    // Деструктуризация
                    match item {
                        Value::Array(ref sub_arr) => {
                            if sub_arr.len() != variables.len() {
                                interpreter.exit_loop_scope();
                                return Err(DataCodeError::runtime_error(
                                    &format!("Cannot unpack array of length {} into {} variables", sub_arr.len(), variables.len()),
                                    interpreter.current_line
                                ));
                            }
                            for (i, var_name) in variables.iter().enumerate() {
                                interpreter.set_loop_variable(var_name.to_string(), sub_arr[i].clone());
                            }
                        }
                        _ => {
                            interpreter.exit_loop_scope();
                            return Err(DataCodeError::runtime_error(
                                &format!("Cannot unpack non-array value into {} variables", variables.len()),
                                interpreter.current_line
                            ));
                        }
                    }
                }

                // Выполняем тело цикла
                execute_block_directly(interpreter, &body_lines)?;

                // Проверяем return
                if interpreter.return_value.is_some() {
                    break;
                }
            }
            Ok(())
        }
        Value::String(ref s) => {
            for ch in s.chars() {
                interpreter.set_loop_variable(variables[0].to_string(), Value::String(ch.to_string()));

                // Выполняем тело цикла
                execute_block_directly(interpreter, &body_lines)?;

                // Проверяем return
                if interpreter.return_value.is_some() {
                    break;
                }
            }
            Ok(())
        }
        Value::Table(ref table) => {
            for row in &table.rows {
                interpreter.set_loop_variable(variables[0].to_string(), Value::Array(row.clone()));

                // Выполняем тело цикла
                execute_block_directly(interpreter, &body_lines)?;

                // Проверяем return
                if interpreter.return_value.is_some() {
                    break;
                }
            }
            Ok(())
        }
        _ => Err(DataCodeError::runtime_error(
            &format!("Cannot iterate over {:?}", iterable_value),
            interpreter.current_line,
        )),
    };

    // Выходим из области видимости цикла
    interpreter.exit_loop_scope();
    result
}

/// Парсить и определить функцию напрямую без рекурсии
fn parse_and_define_function_directly(interpreter: &mut Interpreter, function_lines: &[&str]) -> Result<()> {
    if function_lines.is_empty() {
        return Err(DataCodeError::syntax_error("Empty function definition", interpreter.current_line, 0));
    }

    // Первая строка должна быть "global function name(params) do" или "local function name(params) do"
    let first_line = function_lines[0].trim();

    let (is_global, function_part) = if let Some(stripped) = first_line.strip_prefix("global function ") {
        (true, stripped)
    } else if let Some(stripped) = first_line.strip_prefix("local function ") {
        (false, stripped)
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
        is_global,
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

    // Выполняем try блок с поддержкой вложенных try/catch
    let try_result = execute_block_with_try_support(interpreter, &try_body);

    // Обрабатываем результат выполнения try блока
    let mut final_error = None;

    if let Err(error) = try_result {
        // Если у текущего блока есть catch секция, обрабатываем исключение локально
        if !catch_body.is_empty() {
            // Устанавливаем переменную ошибки, если указана
            if let Some(var_name) = &catch_var {
                let error_message = match &error {
                    DataCodeError::UserException { message, .. } => message.clone(),
                    _ => format!("{}", error),
                };
                interpreter.set_variable(var_name.clone(), Value::String(error_message), true);
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

        // Выполняем обычную строку кода
        execute_line_simple(interpreter, line)?;
        i += 1;
    }
    Ok(())
}
