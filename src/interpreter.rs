use crate::value::Value;
use crate::builtins::call_function;
use std::collections::HashMap;

pub struct Interpreter {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, (Vec<String>, Vec<String>)>, // (params, body) - заглушка
    pub return_value: Option<Value>,
}

impl Interpreter {

    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            return_value: None,
        }
    }

    pub fn set_variable(&mut self, name: &str, val: Value) {
        self.variables.insert(name.to_string(), val);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    fn eval_expr(&self, expr: &str) -> Result<Value, String> {
        println!("eval_expr: {}", expr);
        if let Some(paren_start) = expr.find('(') {
            let func_name = &expr[..paren_start].trim();
            let args_str = expr[paren_start + 1..].trim_end_matches(')').trim();

            let raw_args: Vec<&str> = if args_str.is_empty() {
                vec![]
            } else {
                args_str.split(',').map(|s| s.trim()).collect()
            };

            let mut args = vec![];

            for arg in raw_args {
                let val = if arg.contains('/') {
                    let parts: Vec<_> = arg.splitn(2, '/').map(|s| s.trim()).collect();
                    if parts.len() != 2 {
                        return Err("Invalid / expression in function argument".to_string());
                    }

                    let left_val = self.get_variable(parts[0])
                        .ok_or(format!("Unknown variable: {}", parts[0]))?
                        .clone();

                    let right_raw = parts[1].trim_matches('\'');
                    let right_val = Value::String(right_raw.to_string());

                    left_val.add(&right_val)?
                } else if arg.contains('+') {
                    let parts: Vec<_> = arg.splitn(2, '+').map(|s| s.trim()).collect();
                    if parts.len() != 2 {
                        return Err("Invalid + expression in function argument".to_string());
                    }

                    let left_val = self.get_variable(parts[0])
                        .ok_or(format!("Unknown variable: {}", parts[0]))?
                        .clone();

                    let right_raw = parts[1].trim_matches('\'');
                    let right_val = Value::String(right_raw.to_string());

                    left_val.add(&right_val)?
                } else if arg.starts_with('\'') && arg.ends_with('\'') {
                    Value::String(arg.trim_matches('\'').to_string())
                } else if let Some(var) = self.get_variable(arg) {
                    var.clone()
                } else {
                    return Err(format!("Unknown variable: {}", arg));
                };

                args.push(val);
            }

            call_function(func_name, args)
        } else if expr.contains('/') {
            let parts: Vec<_> = expr.split('/').map(str::trim).collect();
            if parts.is_empty() {
                return Err("Invalid / expression".to_string());
            }

            let mut result = self.eval_expr(parts[0])?;

            for part in &parts[1..] {
                let next_val = self.eval_expr(part)?;
                result = result.add(&next_val)?;
            }

            Ok(result)
        } else if let Some(var) = self.get_variable(expr) {
            Ok(var.clone())
        } else {
            Err("Unsupported expression".to_string())
        }
    }

    pub fn exec(&mut self, code: &str) -> Result<(), String> {
        if code.starts_with("global ") || code.starts_with("local ") {
            let is_global = code.starts_with("global ");
            let code = &code[if is_global { 7 } else { 6 }..];
            let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();

            if parts.len() != 2 {
                return Err("Invalid assignment".to_string());
            }

            let var_name = parts[0];
            let expr = parts[1];

            let val = self.eval_expr(expr)?; 
            self.set_variable(var_name, val);
            return Ok(());
        }
        else if code.trim_start().starts_with("for ") {
            let lines: Vec<&str> = code.lines().collect();
            let header = lines.first().ok_or("Empty for loop")?;
            let footer = lines.last().ok_or("Missing forend")?;
            if !footer.trim().eq("forend") {
                return Err("Missing forend in for loop".to_string());
            }

            let body_lines = &lines[1..lines.len()-1];
            let body_code = body_lines.join("\n");

            // parse: for var in collection do
            let header_parts: Vec<&str> = header.trim().split_whitespace().collect();
            if header_parts.len() < 5 || header_parts[2] != "in" || header_parts[4] != "do" {
                return Err("Invalid for syntax".to_string());
            }
            let var_name = header_parts[1];
            let collection_expr = header_parts[3];
            let collection_val = self.eval_expr(collection_expr)?;

            if let Value::Array(items) = collection_val {
                for item in items {
                    self.set_variable(var_name, item);
                    for line in body_code.lines() {
                        self.exec(line)?;
                    }
                }
                Ok(())
            } else {
                Err("for loop expects array to iterate".to_string())
            }
        } else {
            match self.eval_expr(code.trim()) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Unsupported code: {}", e)),
            }
        }
    }
}