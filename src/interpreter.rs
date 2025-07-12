use crate::value::Value;
use crate::error::{DataCodeError, Result};
use std::collections::HashMap;

pub struct Interpreter {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, (Vec<String>, Vec<String>)>, // (params, body) - заглушка
    pub return_value: Option<Value>,
    pub current_line: usize,
}

impl Interpreter {

    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            return_value: None,
            current_line: 1,
        }
    }

    pub fn set_variable(&mut self, name: &str, val: Value) {
        self.variables.insert(name.to_string(), val);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    fn eval_expr(&self, expr: &str) -> Result<Value> {
        crate::evaluator::parse_and_evaluate(expr, &self.variables, self.current_line)
    }

    pub fn exec(&mut self, code: &str) -> Result<Option<Value>> {
        if code.starts_with("global ") || code.starts_with("local ") {
            let is_global = code.starts_with("global ");
            let code = &code[if is_global { 7 } else { 6 }..];
            let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();

            if parts.len() != 2 {
                return Err(DataCodeError::syntax_error("Invalid assignment", self.current_line, 0));
            }

            let var_name = parts[0];
            let expr = parts[1];

            let val = self.eval_expr(expr)?;
            self.set_variable(var_name, val.clone());
            return Ok(Some(val)); // Возвращаем присвоенное значение
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
                    self.set_variable(var_name, item);
                    for line in body_code.lines() {
                        self.exec(line.trim())?;
                    }
                }
                Ok(None) // Циклы не возвращают значение
            } else {
                Err(DataCodeError::type_error("Array", "other", self.current_line))
            }
        } else {
            // Это выражение - вычисляем и возвращаем результат
            match self.eval_expr(code.trim()) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e),
            }
        }
    }
}