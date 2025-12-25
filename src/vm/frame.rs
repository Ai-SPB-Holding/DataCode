// CallFrame для виртуальной машины

use crate::bytecode::Function;
use crate::common::value::Value;

pub struct CallFrame {
    pub function: Function,
    pub ip: usize,           // Instruction pointer
    pub slots: Vec<Value>,   // Локальные переменные и стек для этой функции
    pub stack_start: usize,  // Начало стека для этой функции в общем стеке VM
    pub cached_args: Option<Vec<Value>>, // Аргументы для кэширования (только для кэшируемых функций)
}

impl CallFrame {
    pub fn new(function: Function, stack_start: usize) -> Self {
        // Оптимизация: резервируем только необходимое количество слотов
        let initial_slots = function.arity.max(8); // Минимум 8 слотов для локальных переменных
        Self {
            slots: Vec::with_capacity(initial_slots + 64), // Динамическое расширение при необходимости
            ip: 0,
            function,
            stack_start,
            cached_args: None,
        }
    }
    
    pub fn new_with_cache(function: Function, stack_start: usize, args: Vec<Value>) -> Self {
        let mut frame = Self::new(function, stack_start);
        frame.cached_args = Some(args);
        frame
    }
}

