// Единый тип значений для VM

use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::common::table::Table;

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Function(usize), // Индекс функции в массиве функций
    NativeFunction(usize), // Индекс нативной функции
    Path(PathBuf), // Путь к файлу или директории
    Table(Rc<RefCell<Table>>),
    Object(HashMap<String, Value>), // Словарь/объект: ключ-значение
    ColumnReference {
        table: Rc<RefCell<Table>>,
        column_name: String,
    },
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => *a.borrow() == *b.borrow(),
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => a == b,
            (Value::Path(a), Value::Path(b)) => a == b,
            (Value::Table(a), Value::Table(b)) => *a.borrow() == *b.borrow(),
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::ColumnReference { table: a, column_name: col_a }, Value::ColumnReference { table: b, column_name: col_b }) => {
                Rc::ptr_eq(a, b) && col_a == col_b
            },
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Value {
    /// Проверяет, можно ли использовать это значение как ключ кэша
    /// (только простые типы: Number, Bool, String, Null)
    pub fn is_hashable(&self) -> bool {
        matches!(self, Value::Number(_) | Value::Bool(_) | Value::String(_) | Value::Null)
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(false) => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),  // Пустая строка = false
            Value::Array(arr) => !arr.borrow().is_empty(),
            Value::Path(p) => !p.as_os_str().is_empty(),  // Путь не пустой = true
            Value::Table(table) => table.borrow().len() > 0,  // Таблица не пустая = true
            Value::Object(map) => !map.is_empty(),  // Объект не пустой = true
            Value::ColumnReference { table, column_name } => {
                let table_ref = table.borrow();
                if let Some(column) = table_ref.get_column(column_name) {
                    !column.is_empty()
                } else {
                    false
                }
            },
            _ => true,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(b) => format!("{}", b),
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let arr_ref = arr.borrow();
                let elements: Vec<String> = arr_ref.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Function(_) => "<function>".to_string(),
            Value::NativeFunction(_) => "<native function>".to_string(),
            Value::Path(p) => {
                // В режиме --use-ve показываем относительные пути
                use crate::websocket::{get_use_ve, get_user_session_path};
                if get_use_ve() {
                    if let Some(session_path) = get_user_session_path() {
                        // Канонизируем оба пути для корректного сравнения
                        let canonical_session = session_path.canonicalize().ok().unwrap_or(session_path);
                        let canonical_path = p.canonicalize().ok().unwrap_or(p.clone());
                        
                        // Проверяем, начинается ли путь с пути сессии
                        if let Ok(stripped) = canonical_path.strip_prefix(&canonical_session) {
                            // Формируем относительный путь с префиксом ./
                            let relative = stripped.to_string_lossy().to_string();
                            if relative.is_empty() || relative == "." {
                                "./".to_string()
                            } else {
                                // Убираем начальные слеши и добавляем ./
                                let trimmed = relative.trim_start_matches(|c| c == '/' || c == '\\');
                                if trimmed.is_empty() {
                                    "./".to_string()
                                } else {
                                    format!("./{}", trimmed)
                                }
                            }
                        } else {
                            // Путь вне сессии - возвращаем как есть (не канонизированный для сохранения оригинального формата)
                            p.to_string_lossy().to_string()
                        }
                    } else {
                        // Нет пути сессии - возвращаем как есть
                        p.to_string_lossy().to_string()
                    }
                } else {
                    // Не режим --use-ve - возвращаем полный путь
                    p.to_string_lossy().to_string()
                }
            },
            Value::Table(table) => {
                let t = table.borrow();
                format!("<table: {} rows, {} columns>", t.len(), t.column_count())
            }
            Value::ColumnReference { table, column_name } => {
                let t = table.borrow();
                if let Some(column) = t.get_column(column_name) {
                    format!("<column: {}.{} ({} values)>", 
                        t.name.as_ref().map(|n| n.as_str()).unwrap_or("table"),
                        column_name,
                        column.len())
                } else {
                    format!("<column: {}.{} (not found)>",
                        t.name.as_ref().map(|n| n.as_str()).unwrap_or("table"),
                        column_name)
                }
            }
            Value::Object(map) => {
                let pairs: Vec<String> = map.iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Null => "null".to_string(),
        }
    }
}

// Реализуем Hash только для простых типов
// Для сложных типов Hash не реализован - они не могут быть ключами кэша
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Number(n) => {
                // Хешируем число как байты для точности
                state.write_u8(0); // Тег для Number
                state.write_u64(n.to_bits());
            }
            Value::Bool(b) => {
                state.write_u8(1); // Тег для Bool
                state.write_u8(if *b { 1 } else { 0 });
            }
            Value::String(s) => {
                state.write_u8(2); // Тег для String
                s.hash(state);
            }
            Value::Null => {
                state.write_u8(3); // Тег для Null
            }
            // Для остальных типов не реализуем Hash - они не могут быть ключами кэша
            _ => {
                panic!("Cannot hash complex types (Array, Table, Object, Function, Path)");
            }
        }
    }
}

impl Eq for Value {}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Number(n) => Value::Number(*n),
            Value::Bool(b) => Value::Bool(*b),
            Value::String(s) => Value::String(s.clone()),
            Value::Array(arr) => {
                // Создаем глубокую копию массива, рекурсивно клонируя все элементы
                let arr_ref = arr.borrow();
                let cloned_vec: Vec<Value> = arr_ref.iter().map(|v| v.clone()).collect();
                Value::Array(Rc::new(RefCell::new(cloned_vec)))
            },
            Value::Function(idx) => Value::Function(*idx),
            Value::NativeFunction(idx) => Value::NativeFunction(*idx),
            Value::Path(p) => Value::Path(p.clone()),
            Value::Table(table) => {
                // Создаем новый Rc с глубокой копией таблицы
                Value::Table(Rc::new(RefCell::new(table.borrow().clone())))
            },
            Value::ColumnReference { table, column_name } => {
                // Для ColumnReference клонируем ссылку на таблицу и имя колонки
                Value::ColumnReference {
                    table: table.clone(),
                    column_name: column_name.clone(),
                }
            },
            Value::Object(map) => {
                // Создаем глубокую копию объекта (клонируем каждое значение)
                let mut cloned_map = HashMap::new();
                for (k, v) in map {
                    cloned_map.insert(k.clone(), v.clone());
                }
                Value::Object(cloned_map)
            },
            Value::Null => Value::Null,
        }
    }
}

