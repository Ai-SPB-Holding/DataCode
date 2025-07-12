use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::builtins::call_function;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<String>,
    pub is_global: bool,
}

impl UserFunction {
    pub fn new(name: String, parameters: Vec<String>, body: Vec<String>, is_global: bool) -> Self {
        Self {
            name,
            parameters,
            body,
            is_global,
        }
    }
}

pub struct Interpreter {
    pub variables: HashMap<String, Value>,
    pub user_functions: HashMap<String, UserFunction>,
    pub return_value: Option<Value>,
    pub current_line: usize,
    pub call_stack: Vec<HashMap<String, Value>>, // Стек локальных переменных для функций
    pub loop_stack: Vec<HashMap<String, Value>>, // Стек локальных переменных для циклов
}

impl Interpreter {

    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            user_functions: HashMap::new(),
            return_value: None,
            current_line: 1,
            call_stack: Vec::new(),
            loop_stack: Vec::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        // Сначала ищем в локальных переменных циклов (проверяем все уровни, начиная с последнего)
        for loop_vars in self.loop_stack.iter().rev() {
            if let Some(value) = loop_vars.get(name) {
                return Some(value);
            }
        }

        // Затем ищем в локальных переменных функций (стек вызовов)
        if let Some(local_vars) = self.call_stack.last() {
            if let Some(value) = local_vars.get(name) {
                return Some(value);
            }
        }

        // Затем в глобальных переменных
        self.variables.get(name)
    }

    pub fn get_all_variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    pub fn set_variable(&mut self, name: String, value: Value, is_global: bool) {
        if is_global {
            // Явно глобальная переменная
            self.variables.insert(name, value);
        } else if !self.loop_stack.is_empty() {
            // Мы в цикле - устанавливаем в локальном контексте цикла
            if let Some(loop_vars) = self.loop_stack.last_mut() {
                loop_vars.insert(name, value);
            }
        } else if !self.call_stack.is_empty() {
            // Мы в функции - устанавливаем в локальном контексте функции
            if let Some(local_vars) = self.call_stack.last_mut() {
                local_vars.insert(name, value);
            }
        } else {
            // Глобальная переменная (не в функции и не в цикле)
            self.variables.insert(name, value);
        }
    }

    // Специальный метод для установки переменной цикла
    pub fn set_loop_variable(&mut self, name: String, value: Value) {
        if let Some(loop_vars) = self.loop_stack.last_mut() {
            loop_vars.insert(name, value);
        }
    }

    fn eval_expr(&mut self, expr: &str) -> Result<Value> {
        // Парсим выражение
        let mut parser = crate::parser::Parser::new(expr);
        let parsed_expr = parser.parse_expression()?;

        // Вычисляем с поддержкой пользовательских функций
        self.evaluate_expression(&parsed_expr)
    }

    fn evaluate_expression(&mut self, expr: &crate::parser::Expr) -> Result<Value> {
        use crate::parser::Expr;

        match expr {
            Expr::Literal(value) => Ok(value.clone()),

            Expr::Variable(name) => {
                self.get_variable(name)
                    .cloned()
                    .ok_or_else(|| DataCodeError::variable_not_found(name, self.current_line))
            }

            Expr::FunctionCall { name, args } => {
                // Сначала вычисляем аргументы
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg)?);
                }

                // Проверяем, является ли это пользовательской функцией
                if self.user_functions.contains_key(name) {
                    self.call_user_function(name, arg_values)
                } else {
                    // Встроенная функция
                    call_function(name, arg_values, self.current_line)
                }
            }

            Expr::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.evaluate_binary_op(&left_val, operator, &right_val)
            }

            Expr::Unary { operator, operand } => {
                let operand_val = self.evaluate_expression(operand)?;
                self.evaluate_unary_op(operator, &operand_val)
            }

            Expr::Index { object, index } => {
                let obj_val = self.evaluate_expression(object)?;
                let idx_val = self.evaluate_expression(index)?;
                self.evaluate_index(&obj_val, &idx_val)
            }

            Expr::Member { object, member } => {
                let obj_val = self.evaluate_expression(object)?;
                self.evaluate_member(&obj_val, member)
            }
        }
    }

    pub fn define_function(&mut self, name: String, parameters: Vec<String>, body: Vec<String>, is_global: bool) -> Result<()> {
        let function = UserFunction::new(name.clone(), parameters, body, is_global);
        self.user_functions.insert(name, function);
        Ok(())
    }

    pub fn call_user_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Получаем функцию
        let function = self.user_functions.get(name)
            .ok_or_else(|| DataCodeError::function_not_found(name, self.current_line))?
            .clone();

        // Проверяем количество аргументов
        if args.len() != function.parameters.len() {
            return Err(DataCodeError::wrong_argument_count(
                name,
                function.parameters.len(),
                args.len(),
                self.current_line
            ));
        }

        // Создаем новый локальный контекст
        let mut local_vars = HashMap::new();

        // Связываем параметры с аргументами
        for (param, arg) in function.parameters.iter().zip(args.iter()) {
            local_vars.insert(param.clone(), arg.clone());
        }

        // Добавляем локальный контекст в стек
        self.call_stack.push(local_vars);

        // Сбрасываем return_value
        self.return_value = None;

        // Выполняем тело функции как многострочный код
        let mut result = Value::Null;
        let function_body = function.body.join("\n");
        match self.exec(&function_body) {
            Ok(_) => {
                // Проверяем, был ли return
                if let Some(return_val) = &self.return_value {
                    result = return_val.clone();
                }
            }
            Err(e) => {
                // Убираем локальный контекст при ошибке
                self.call_stack.pop();
                return Err(e);
            }
        }

        // Убираем локальный контекст
        self.call_stack.pop();

        // Сбрасываем return_value
        self.return_value = None;

        Ok(result)
    }

    pub fn exec(&mut self, code: &str) -> Result<()> {
        // Если код содержит несколько строк, обрабатываем построчно
        if code.contains('\n') {
            return self.exec_multiline(code);
        }

        // Обрабатываем одну строку
        self.exec_single_line(code)
    }

    fn exec_multiline(&mut self, code: &str) -> Result<()> {
        let lines: Vec<&str> = code.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Пропускаем пустые строки и комментарии
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            // Проверяем многострочные конструкции
            if (line.starts_with("global function ") || line.starts_with("local function ")) && line.ends_with(" do") {
                // Собираем всю функцию
                let mut function_lines = vec![lines[i]];
                i += 1;

                while i < lines.len() {
                    let current_line = lines[i].trim();
                    function_lines.push(lines[i]);

                    if current_line == "endfunction" {
                        break;
                    }
                    i += 1;
                }

                // Выполняем всю функцию как одну команду
                let function_code = function_lines.join("\n");
                self.exec_single_line(&function_code)?;

                // Проверяем, был ли выполнен return
                if self.return_value.is_some() {
                    break;
                }
            } else if line.starts_with("for ") && line.ends_with(" do") {
                // Собираем весь цикл с учетом вложенности
                let mut loop_lines = vec![lines[i]];
                let mut for_depth = 1; // Счетчик вложенности for
                i += 1;

                while i < lines.len() && for_depth > 0 {
                    let current_line = lines[i].trim();

                    // Увеличиваем глубину при встрече нового for
                    if current_line.starts_with("for ") && current_line.ends_with(" do") {
                        for_depth += 1;
                    }
                    // Уменьшаем глубину при встрече forend
                    else if current_line == "forend" {
                        for_depth -= 1;
                    }

                    loop_lines.push(lines[i]);

                    // Если глубина стала 0, мы закончили основной блок for
                    if for_depth == 0 {
                        break;
                    }

                    i += 1;
                }

                // Проверяем, что цикл правильно закрыт
                if for_depth > 0 {
                    return Err(DataCodeError::syntax_error("Missing forend in for loop", self.current_line, 0));
                }

                // Выполняем весь цикл как одну команду
                let loop_code = loop_lines.join("\n");
                self.exec_single_line(&loop_code)?;

                // Проверяем, был ли выполнен return
                if self.return_value.is_some() {
                    break;
                }
            } else if line.starts_with("if ") && (line.ends_with(" do") || line.ends_with(" then")) {
                // Собираем всю условную конструкцию с учетом вложенности
                let mut if_lines = vec![lines[i]];
                let mut if_depth = 1; // Счетчик вложенности if
                i += 1;

                while i < lines.len() && if_depth > 0 {
                    let current_line = lines[i].trim();

                    // Увеличиваем глубину при встрече нового if
                    if current_line.starts_with("if ") && (current_line.ends_with(" do") || current_line.ends_with(" then")) {
                        if_depth += 1;
                    }
                    // Уменьшаем глубину при встрече endif
                    else if current_line == "endif" {
                        if_depth -= 1;
                    }

                    if_lines.push(lines[i]);

                    // Если глубина стала 0, мы закончили основной блок if
                    if if_depth == 0 {
                        break;
                    }

                    i += 1;
                }

                // Выполняем всю условную конструкцию как одну команду
                let if_code = if_lines.join("\n");
                self.exec_single_line(&if_code)?;

                // Проверяем, был ли выполнен return
                if self.return_value.is_some() {
                    break;
                }
            } else {
                // Обычная строка
                self.exec_single_line(lines[i])?;
            }

            // Проверяем, был ли выполнен return
            if self.return_value.is_some() {
                break;
            }

            i += 1;
        }
        Ok(())
    }

    fn exec_single_line(&mut self, code: &str) -> Result<()> {
        self.current_line += 1;

        // Пропускаем пустые строки и комментарии
        let trimmed_code = code.trim();
        if trimmed_code.is_empty() || trimmed_code.starts_with('#') {
            return Ok(());
        }

        // Обработка определения функций
        if trimmed_code.starts_with("global function ") || trimmed_code.starts_with("local function ") {
            return self.parse_function_definition(code);
        }

        // Обработка return
        if code.trim_start().starts_with("return") {
            let after_return = code.trim_start().strip_prefix("return").unwrap().trim();
            let value = if after_return.is_empty() {
                Value::Null
            } else {
                self.eval_expr(after_return)?
            };
            self.return_value = Some(value);
            return Ok(());
        }

        let trimmed_code = code.trim();
        if trimmed_code.starts_with("global ") || trimmed_code.starts_with("local ") {
            let is_global = trimmed_code.starts_with("global ");
            let code = &trimmed_code[if is_global { 7 } else { 6 }..];
            let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();

            if parts.len() != 2 {
                return Err(DataCodeError::syntax_error("Invalid assignment", self.current_line, 0));
            }

            let var_name = parts[0].to_string();
            let expr = parts[1];

            let val = self.eval_expr(expr)?;
            self.set_variable(var_name.to_string(), val, is_global);
            return Ok(());
        }
        else if code.trim_start().starts_with("for ") {
            let lines: Vec<&str> = code.lines().collect();
            let header = lines.first().ok_or_else(|| DataCodeError::syntax_error("Empty for loop", self.current_line, 0))?;
            let footer = lines.last().ok_or_else(|| DataCodeError::syntax_error("Missing forend", self.current_line, 0))?;
            if !footer.trim().eq("forend") {
                return Err(DataCodeError::syntax_error("Missing forend in for loop", self.current_line, 0));
            }

            let body_lines = &lines[1..lines.len()-1];
            let body_code = body_lines.join("\n");

            let header_parts: Vec<&str> = header.trim().split_whitespace().collect();
            if header_parts.len() < 5 || header_parts[2] != "in" || header_parts[4] != "do" {
                return Err(DataCodeError::syntax_error("Invalid for syntax", self.current_line, 0));
            }
            let var_name = header_parts[1];
            let collection_expr = header_parts[3];
            let collection_val = self.eval_expr(collection_expr)?;

            if let Value::Array(items) = collection_val {
                for item in items {
                    // Создаем новый локальный контекст для цикла
                    self.loop_stack.push(HashMap::new());

                    // Устанавливаем переменную цикла в локальном контексте
                    self.set_loop_variable(var_name.to_string(), item);

                    // Выполняем тело цикла - используем exec() для обработки многострочных конструкций
                    if !body_code.trim().is_empty() {
                        self.exec(&body_code)?;
                        // Проверяем return в цикле
                        if self.return_value.is_some() {
                            // Убираем локальный контекст цикла перед выходом
                            self.loop_stack.pop();
                            return Ok(());
                        }
                    }

                    // Убираем локальный контекст цикла после завершения итерации
                    self.loop_stack.pop();
                }
                Ok(())
            } else {
                Err(DataCodeError::type_error("Array", "other", self.current_line))
            }
        }
        else if code.trim_start().starts_with("if ") {
            return self.parse_if_statement(code);
        } else {
            // Это выражение - вычисляем
            match self.eval_expr(code.trim()) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
    }

    fn parse_function_definition(&mut self, code: &str) -> Result<()> {
        let lines: Vec<&str> = code.lines().collect();

        // Проверяем, что функция заканчивается на endfunction
        let last_line = lines.last()
            .ok_or_else(|| DataCodeError::syntax_error("Empty function definition", self.current_line, 0))?;

        if !last_line.trim().eq("endfunction") {
            return Err(DataCodeError::syntax_error("Missing endfunction", self.current_line, 0));
        }

        // Парсим заголовок функции
        let header = lines.first()
            .ok_or_else(|| DataCodeError::syntax_error("Empty function definition", self.current_line, 0))?;

        let (is_global, function_name, parameters) = self.parse_function_header(header)?;

        // Извлекаем тело функции (все строки кроме первой и последней)
        let body: Vec<String> = lines[1..lines.len()-1]
            .iter()
            .map(|line| line.to_string())
            .collect();

        // Определяем функцию
        self.define_function(function_name, parameters, body, is_global)?;

        Ok(())
    }

    fn parse_function_header(&self, header: &str) -> Result<(bool, String, Vec<String>)> {
        let header = header.trim();

        // Определяем, глобальная или локальная функция
        let (is_global, rest) = if header.starts_with("global function ") {
            (true, header.strip_prefix("global function ").unwrap())
        } else if header.starts_with("local function ") {
            (false, header.strip_prefix("local function ").unwrap())
        } else {
            return Err(DataCodeError::syntax_error("Invalid function definition", self.current_line, 0));
        };

        // Ищем скобки для параметров
        let paren_start = rest.find('(')
            .ok_or_else(|| DataCodeError::syntax_error("Missing opening parenthesis in function definition", self.current_line, 0))?;

        let paren_end = rest.find(')')
            .ok_or_else(|| DataCodeError::syntax_error("Missing closing parenthesis in function definition", self.current_line, 0))?;

        if paren_end <= paren_start {
            return Err(DataCodeError::syntax_error("Invalid parentheses in function definition", self.current_line, 0));
        }

        // Извлекаем имя функции
        let function_name = rest[..paren_start].trim().to_string();
        if function_name.is_empty() {
            return Err(DataCodeError::syntax_error("Missing function name", self.current_line, 0));
        }

        // Извлекаем параметры
        let params_str = &rest[paren_start + 1..paren_end];
        let parameters: Vec<String> = if params_str.trim().is_empty() {
            Vec::new()
        } else {
            params_str
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect()
        };

        // Проверяем, что заголовок заканчивается на "do"
        let after_params = &rest[paren_end + 1..].trim();
        if *after_params != "do" {
            return Err(DataCodeError::syntax_error("Function definition must end with 'do'", self.current_line, 0));
        }

        Ok((is_global, function_name, parameters))
    }

    fn evaluate_binary_op(&self, left: &Value, op: &crate::parser::BinaryOp, right: &Value) -> Result<Value> {
        use crate::parser::BinaryOp;
        use Value::*;

        match op {
            BinaryOp::Add => left.add(right).map_err(|e| DataCodeError::runtime_error(&e, self.current_line)),

            BinaryOp::PathJoin => left.add(right).map_err(|e| DataCodeError::runtime_error(&e, self.current_line)),

            BinaryOp::Subtract => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a - b)),
                _ => Err(DataCodeError::type_error("Number", "other", self.current_line)),
            },

            BinaryOp::Multiply => match (left, right) {
                (Number(a), Number(b)) => Ok(Number(a * b)),
                (String(s), Number(n)) => {
                    let count = *n as usize;
                    Ok(String(s.repeat(count)))
                }
                _ => Err(DataCodeError::type_error("Number", "other", self.current_line)),
            },

            BinaryOp::Divide => match (left, right) {
                (Number(a), Number(b)) => {
                    if *b == 0.0 {
                        Err(DataCodeError::runtime_error("Division by zero", self.current_line))
                    } else {
                        Ok(Number(a / b))
                    }
                }
                _ => Err(DataCodeError::type_error("Number", "other", self.current_line)),
            },

            BinaryOp::Equal => Ok(Bool(self.values_equal(left, right))),

            BinaryOp::NotEqual => Ok(Bool(!self.values_equal(left, right))),

            BinaryOp::Less => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a < b)),
                (String(a), String(b)) => Ok(Bool(a < b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.current_line)),
            },

            BinaryOp::Greater => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a > b)),
                (String(a), String(b)) => Ok(Bool(a > b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.current_line)),
            },

            BinaryOp::LessEqual => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a <= b)),
                (String(a), String(b)) => Ok(Bool(a <= b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.current_line)),
            },

            BinaryOp::GreaterEqual => match (left, right) {
                (Number(a), Number(b)) => Ok(Bool(a >= b)),
                (String(a), String(b)) => Ok(Bool(a >= b)),
                _ => Err(DataCodeError::type_error("comparable types", "other", self.current_line)),
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

    fn evaluate_unary_op(&self, op: &crate::parser::UnaryOp, operand: &Value) -> Result<Value> {
        use crate::parser::UnaryOp;

        match op {
            UnaryOp::Not => Ok(Value::Bool(!self.to_bool(operand))),
            UnaryOp::Minus => match operand {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(DataCodeError::type_error("Number", "other", self.current_line)),
            },
        }
    }

    fn evaluate_index(&self, object: &Value, index: &Value) -> Result<Value> {
        match (object, index) {
            (Value::Array(arr), Value::Number(n)) => {
                let idx = *n as usize;
                arr.get(idx)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error("Index out of bounds", self.current_line))
            }
            (Value::Object(obj), Value::String(key)) => {
                obj.get(key)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error(&format!("Key '{}' not found", key), self.current_line))
            }
            _ => Err(DataCodeError::type_error("indexable type", "other", self.current_line)),
        }
    }

    fn evaluate_member(&self, object: &Value, member: &str) -> Result<Value> {
        match object {
            Value::Object(obj) => {
                obj.get(member)
                    .cloned()
                    .ok_or_else(|| DataCodeError::runtime_error(&format!("Member '{}' not found", member), self.current_line))
            }
            Value::Array(arr) => {
                match member {
                    "length" => Ok(Value::Number(arr.len() as f64)),
                    _ => Err(DataCodeError::runtime_error(&format!("Array has no member '{}'", member), self.current_line)),
                }
            }
            Value::String(s) => {
                match member {
                    "length" => Ok(Value::Number(s.len() as f64)),
                    _ => Err(DataCodeError::runtime_error(&format!("String has no member '{}'", member), self.current_line)),
                }
            }
            _ => Err(DataCodeError::type_error("object with members", "other", self.current_line)),
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
            (String(a), String(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Null, Null) => true,
            (Array(a), Array(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y)),
            _ => false,
        }
    }

    fn to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Null => false,
            Value::Path(p) => p.exists(),
        }
    }

    fn parse_if_statement(&mut self, code: &str) -> Result<()> {
        let lines: Vec<&str> = code.lines().collect();

        // Проверяем, что условная конструкция заканчивается на endif
        let last_line = lines.last()
            .ok_or_else(|| DataCodeError::syntax_error("Empty if statement", self.current_line, 0))?;

        if !last_line.trim().eq("endif") {
            return Err(DataCodeError::syntax_error("Missing endif", self.current_line, 0));
        }

        // Парсим заголовок условия
        let header = lines.first()
            .ok_or_else(|| DataCodeError::syntax_error("Empty if statement", self.current_line, 0))?;

        let condition_expr = self.parse_if_header(header)?;

        // Ищем else на том же уровне вложенности
        let mut else_index = None;
        let mut if_depth = 1; // Начинаем с 1, так как мы уже внутри первого if

        for (i, line) in lines.iter().enumerate() {
            if i == 0 { continue; } // Пропускаем первую строку (заголовок if)

            let trimmed = line.trim();

            // Увеличиваем глубину при встрече if
            if trimmed.starts_with("if ") && (trimmed.ends_with(" do") || trimmed.ends_with(" then")) {
                if_depth += 1;
            }
            // Уменьшаем глубину при встрече endif
            else if trimmed == "endif" {
                if_depth -= 1;
            }
            // Ищем else только на уровне 1 (наш основной if)
            else if trimmed == "else" && if_depth == 1 {
                else_index = Some(i);
                break;
            }
        }

        // Извлекаем блоки кода
        let (if_body, else_body) = if let Some(else_idx) = else_index {
            // Есть блок else
            let if_lines = &lines[1..else_idx];
            let else_lines = &lines[else_idx + 1..lines.len() - 1];
            (
                if_lines.iter().map(|line| line.to_string()).collect::<Vec<String>>(),
                Some(else_lines.iter().map(|line| line.to_string()).collect::<Vec<String>>())
            )
        } else {
            // Только блок if
            let if_lines = &lines[1..lines.len() - 1];
            (
                if_lines.iter().map(|line| line.to_string()).collect::<Vec<String>>(),
                None
            )
        };

        // Вычисляем условие
        let condition_value = self.eval_expr(&condition_expr)?;
        let condition_result = self.to_bool(&condition_value);

        // Выполняем соответствующий блок
        if condition_result {
            // Выполняем блок if
            if !if_body.is_empty() {
                let if_code = if_body.join("\n");
                self.exec(&if_code)?;
            }
        } else if let Some(else_lines) = else_body {
            // Выполняем блок else
            if !else_lines.is_empty() {
                let else_code = else_lines.join("\n");
                self.exec(&else_code)?;
            }
        }

        Ok(())
    }

    fn parse_if_header(&self, header: &str) -> Result<String> {
        let header = header.trim();

        // Проверяем, что начинается с "if "
        if !header.starts_with("if ") {
            return Err(DataCodeError::syntax_error("Invalid if statement", self.current_line, 0));
        }

        // Убираем "if " в начале
        let rest = header.strip_prefix("if ").unwrap();

        // Проверяем, что заканчивается на "do" или "then"
        let condition_expr = if rest.ends_with(" do") {
            rest.strip_suffix(" do").unwrap()
        } else if rest.ends_with(" then") {
            rest.strip_suffix(" then").unwrap()
        } else {
            return Err(DataCodeError::syntax_error("If statement must end with 'do' or 'then'", self.current_line, 0));
        };

        if condition_expr.trim().is_empty() {
            return Err(DataCodeError::syntax_error("Missing condition in if statement", self.current_line, 0));
        }

        Ok(condition_expr.to_string())
    }
}