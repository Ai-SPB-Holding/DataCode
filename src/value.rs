use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
    Path(PathBuf),
}

impl Value {
    // Пример операции сложения для Path + String
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Path(p), String(s)) => {
                let mut new_path = p.clone();
                let relative = s.trim_start_matches('/');
                new_path.push(relative);
                Ok(Path(new_path))
            }
            (String(s), Path(p)) => {
                let mut new_str = s.clone();
                new_str.push_str(p.to_str().ok_or("Invalid path to string conversion")?);
                Ok(String(new_str))
            }
            (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            (Number(a), Number(b)) => Ok(Number(a + b)),
            _ => Err(format!("Unsupported add operation between {:?} and {:?}", self, other)),
        }
    }
}