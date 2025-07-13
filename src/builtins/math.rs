use crate::value::Value;
use crate::error::{DataCodeError, Result};

/// Mathematical functions
pub fn call_math_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "abs" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("abs", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(n.abs())),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        
        "sqrt" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sqrt", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => {
                    if *n < 0.0 {
                        Err(DataCodeError::runtime_error("Cannot take square root of negative number", line))
                    } else {
                        Ok(Number(n.sqrt()))
                    }
                }
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        
        "pow" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("pow", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Number(base), Number(exp)) => Ok(Number(base.powf(*exp))),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        
        "min" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("min", 1, 0, line));
            }
            let mut min_val = match &args[0] {
                Number(n) => *n,
                _ => return Err(DataCodeError::type_error("Number", "other", line)),
            };
            for arg in &args[1..] {
                match arg {
                    Number(n) => {
                        if *n < min_val {
                            min_val = *n;
                        }
                    }
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            }
            Ok(Number(min_val))
        }
        
        "max" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("max", 1, 0, line));
            }
            let mut max_val = match &args[0] {
                Number(n) => *n,
                _ => return Err(DataCodeError::type_error("Number", "other", line)),
            };
            for arg in &args[1..] {
                match arg {
                    Number(n) => {
                        if *n > max_val {
                            max_val = *n;
                        }
                    }
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            }
            Ok(Number(max_val))
        }
        
        "round" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("round", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(n.round())),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        
        "div" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("div", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Number(a), Number(b)) => {
                    if *b == 0.0 {
                        Err(DataCodeError::runtime_error("Division by zero", line))
                    } else {
                        Ok(Number(a / b))
                    }
                }
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to math functions
pub fn is_math_function(name: &str) -> bool {
    matches!(name, "abs" | "sqrt" | "pow" | "min" | "max" | "round" | "div")
}
