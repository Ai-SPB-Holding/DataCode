use crate::value::Value;
use crate::error::{DataCodeError, Result};
use std::collections::HashMap;

/// Уникальный идентификатор функции
pub type FunctionId = String;

/// Временный идентификатор переменной для хранения результата вызова функции
pub type TempVarId = String;

/// Результат выполнения одной инструкции
#[derive(Debug, Clone)]
pub enum ExecResult {
    /// Продолжить выполнение следующей инструкции
    Continue,
    /// Функция завершена, вернуть значение
    Return(Value),
    /// Вызвать функцию (создать новый фрейм)
    #[allow(dead_code)]
    Call {
        function_id: FunctionId,
        args: Vec<Value>,
    },
    /// Хвостовой вызов (заменить текущий фрейм)
    #[allow(dead_code)]
    TailCall {
        function_id: FunctionId,
        args: Vec<Value>,
    },
}

/// Сигнал выполнения выражения или инструкции
/// Используется для единого событийного цикла без рекурсии Rust
#[derive(Debug, Clone)]
pub enum ExecSignal {
    /// Выражение вычислено, значение готово
    Value(Value),
    /// Нужно вызвать функцию (создать новый фрейм)
    Call {
        function_id: FunctionId,
        args: Vec<Value>,
        return_slot: Option<TempVarId>,
    },
    /// Функция завершена, вернуть значение
    Return(Value),
}

/// Один фрейм вызова функции
/// Содержит всё необходимое для выполнения функции
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Имя функции
    pub function_id: FunctionId,
    /// Индекс текущей инструкции (строка в теле функции)
    pub ip: usize,
    /// Локальные переменные функции
    pub locals: HashMap<String, Value>,
    /// Аргументы функции
    pub args: Vec<Value>,
    /// Слот для сохранения возвращаемого значения (если вызов в выражении)
    pub return_slot: Option<TempVarId>,
    /// Глубина вложенности этого фрейма
    pub depth: usize,
}

impl CallFrame {
    /// Создать новый фрейм вызова
    pub fn new(function_id: FunctionId, args: Vec<Value>, return_slot: Option<TempVarId>, depth: usize) -> Self {
        Self {
            function_id,
            ip: 0,
            locals: HashMap::new(),
            args,
            return_slot,
            depth,
        }
    }

    /// Проверить, завершено ли выполнение функции
    #[allow(dead_code)]
    pub fn is_done(&self, body_len: usize) -> bool {
        self.ip >= body_len
    }

    /// Получить текущую инструкцию
    pub fn current_instruction<'a>(&self, body: &'a [String]) -> Option<&'a String> {
        if self.ip < body.len() {
            Some(&body[self.ip])
        } else {
            None
        }
    }

    /// Перейти к следующей инструкции
    pub fn advance(&mut self) {
        self.ip += 1;
    }

    /// Установить локальную переменную
    pub fn set_local(&mut self, name: String, value: Value) {
        self.locals.insert(name, value);
    }

    /// Получить локальную переменную
    pub fn get_local(&self, name: &str) -> Option<&Value> {
        self.locals.get(name)
    }
}

/// Стек вызовов функций
/// Используется для итеративного выполнения вместо рекурсии Rust
#[derive(Debug)]
pub struct CallStack {
    /// Стек фреймов вызовов
    frames: Vec<CallFrame>,
    /// Максимальная глубина стека (защита от бесконечной рекурсии)
    max_depth: usize,
}

impl CallStack {
    /// Создать новый стек вызовов
    pub fn new(max_depth: usize) -> Self {
        Self {
            frames: Vec::new(),
            max_depth,
        }
    }

    /// Добавить фрейм в стек
    pub fn push(&mut self, frame: CallFrame) -> Result<()> {
        if self.frames.len() >= self.max_depth {
            return Err(DataCodeError::runtime_error(
                &format!(
                    "Превышена максимальная глубина стека вызовов ({}) в функции '{}'",
                    self.max_depth,
                    frame.function_id
                ),
                0
            ));
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Удалить фрейм из стека
    pub fn pop(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }

    /// Получить мутабельную ссылку на последний фрейм
    pub fn last_mut(&mut self) -> Option<&mut CallFrame> {
        self.frames.last_mut()
    }

    /// Получить ссылку на последний фрейм
    pub fn last(&self) -> Option<&CallFrame> {
        self.frames.last()
    }

    /// Получить длину стека
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Проверить, пуст ли стек
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Получить трассировку стека для отладки
    #[allow(dead_code)]
    pub fn get_stack_trace(&self) -> Vec<String> {
        self.frames
            .iter()
            .rev()
            .map(|frame| {
                let args_str = frame
                    .args
                    .iter()
                    .map(|v| format_value_for_trace(v))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", frame.function_id, args_str)
            })
            .collect()
    }

    /// Заменить текущий фрейм (для TCO)
    /// Вместо добавления нового фрейма, заменяем текущий
    pub fn replace_top(&mut self, frame: CallFrame) -> Result<()> {
        if self.frames.is_empty() {
            return Err(DataCodeError::runtime_error(
                "Cannot replace frame: stack is empty",
                0
            ));
        }
        let depth = self.frames.last().unwrap().depth;
        let mut new_frame = frame;
        new_frame.depth = depth; // Сохраняем глубину
        self.frames.pop();
        self.frames.push(new_frame);
        Ok(())
    }
}

/// Форматировать значение для трассировки стека
#[allow(dead_code)]
fn format_value_for_trace(value: &Value) -> String {
    use Value::*;
    match value {
        Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        String(s) => format!("\"{}\"", s),
        Bool(b) => b.to_string(),
        Null => "null".to_string(),
        Array(arr) => {
            if arr.len() <= 3 {
                let items: Vec<std::string::String> = arr.iter().map(format_value_for_trace).collect();
                format!("[{}]", items.join(", "))
            } else {
                format!("[{} items]", arr.len())
            }
        }
        Object(obj) => {
            if obj.len() <= 3 {
                let items: Vec<std::string::String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, format_value_for_trace(v)))
                    .collect();
                format!("{{{}}}", items.join(", "))
            } else {
                format!("{{{} items}}", obj.len())
            }
        }
        _ => format!("{:?}", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_frame_creation() {
        let frame = CallFrame::new(
            "test_func".to_string(),
            vec![Value::Number(42.0)],
            None,
            0,
        );
        assert_eq!(frame.function_id, "test_func");
        assert_eq!(frame.ip, 0);
        assert_eq!(frame.args.len(), 1);
    }

    #[test]
    fn test_call_stack_operations() {
        let mut stack = CallStack::new(1000);
        
        let frame1 = CallFrame::new("func1".to_string(), vec![], None::<String>, 0);
        let frame2 = CallFrame::new("func2".to_string(), vec![], None::<String>, 1);
        
        stack.push(frame1).unwrap();
        stack.push(frame2).unwrap();
        
        assert_eq!(stack.len(), 2);
        assert!(!stack.is_empty());
        
        let popped = stack.pop();
        assert!(popped.is_some());
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_call_stack_max_depth() {
        let mut stack = CallStack::new(2);
        
        let frame1 = CallFrame::new("func1".to_string(), vec![], None::<String>, 0);
        let frame2 = CallFrame::new("func2".to_string(), vec![], None::<String>, 1);
        let frame3 = CallFrame::new("func3".to_string(), vec![], None::<String>, 2);
        
        stack.push(frame1).unwrap();
        stack.push(frame2).unwrap();
        
        // Третий фрейм должен вызвать ошибку
        assert!(stack.push(frame3).is_err());
    }

    #[test]
    fn test_stack_trace() {
        let mut stack = CallStack::new(1000);
        
        let frame1 = CallFrame::new("func1".to_string(), vec![Value::Number(1.0)], None, 0);
        let frame2 = CallFrame::new("func2".to_string(), vec![Value::String("test".to_string())], None, 1);
        
        stack.push(frame1).unwrap();
        stack.push(frame2).unwrap();
        
        let trace = stack.get_stack_trace();
        assert_eq!(trace.len(), 2);
        assert!(trace[0].contains("func2"));
        assert!(trace[1].contains("func1"));
    }
}

