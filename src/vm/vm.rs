// Виртуальная машина

use crate::bytecode::{Chunk, OpCode};
use crate::common::{error::{LangError, StackTraceEntry, ErrorType}, value::Value};
use crate::vm::frame::CallFrame;
use crate::vm::natives;
use std::rc::Rc;
use std::cell::RefCell;

pub type NativeFn = fn(&[Value]) -> Value;

/// Структура для хранения явной связи между колонками таблиц
#[derive(Debug, Clone)]
pub struct ExplicitRelation {
    pub source_table_name: String,
    pub source_column_name: String,
    pub target_table_name: String,
    pub target_column_name: String,
}

/// Структура для хранения явного первичного ключа таблицы
#[derive(Debug, Clone)]
pub struct ExplicitPrimaryKey {
    pub table_name: String,
    pub column_name: String,
}

// Структура для обработчика исключений в VM
struct ExceptionHandler {
    catch_ips: Vec<usize>,           // IP начала каждого catch блока
    error_types: Vec<Option<usize>>, // Типы ошибок для каждого catch (None для catch всех)
    error_var_slots: Vec<Option<usize>>, // Слоты для переменных ошибок
    else_ip: Option<usize>,         // IP начала else блока
    stack_height: usize,             // Высота стека при входе в try
    had_error: bool,                 // Флаг, указывающий, была ли ошибка в try блоке
    frame_index: usize,              // Индекс фрейма, к которому относится этот обработчик
}

pub struct Vm {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: Vec<Value>,
    functions: Vec<crate::bytecode::Function>,
    natives: Vec<NativeFn>,
    exception_handlers: Vec<ExceptionHandler>, // Стек обработчиков исключений
    error_type_table: Vec<String>, // Таблица типов ошибок для текущей функции
    global_names: std::collections::HashMap<usize, String>, // Маппинг индексов глобальных переменных на их имена
    explicit_global_names: std::collections::HashMap<usize, String>, // Маппинг индексов переменных, явно объявленных с ключевым словом 'global'
    explicit_relations: Vec<ExplicitRelation>, // Явные связи, созданные через relate()
    explicit_primary_keys: Vec<ExplicitPrimaryKey>, // Явные первичные ключи, созданные через primary_key()
}

impl Vm {
    pub fn new() -> Self {
        let mut vm = Self {
            stack: Vec::new(),
            frames: Vec::new(),
            globals: Vec::new(),
            functions: Vec::new(),
            natives: Vec::new(),
            exception_handlers: Vec::new(),
            error_type_table: Vec::new(),
            global_names: std::collections::HashMap::new(),
            explicit_global_names: std::collections::HashMap::new(),
            explicit_relations: Vec::new(),
            explicit_primary_keys: Vec::new(),
        };
        vm.register_natives();
        vm
    }

    fn register_natives(&mut self) {
        // Регистрируем нативные функции
        // Порядок важен - индексы должны соответствовать register_native_globals
        self.natives.push(natives::native_print);      // 0
        self.natives.push(natives::native_len);        // 1
        self.natives.push(natives::native_range);      // 2
        self.natives.push(natives::native_int);        // 3
        self.natives.push(natives::native_float);        // 4
        self.natives.push(natives::native_bool);        // 5
        self.natives.push(natives::native_str);        // 6
        self.natives.push(natives::native_array);      // 7
        self.natives.push(natives::native_typeof);     // 8
        self.natives.push(natives::native_isinstance); // 9
        self.natives.push(natives::native_date);      // 10
        self.natives.push(natives::native_money);     // 11
        self.natives.push(natives::native_path);     // 12
        self.natives.push(natives::native_path_name);     // 13
        self.natives.push(natives::native_path_parent);   // 14
        self.natives.push(natives::native_path_exists);   // 15
        self.natives.push(natives::native_path_is_file); // 16
        self.natives.push(natives::native_path_is_dir);  // 17
        self.natives.push(natives::native_path_extension); // 18
        self.natives.push(natives::native_path_stem);    // 19
        self.natives.push(natives::native_path_len);     // 20
        // Математические функции
        self.natives.push(natives::native_abs);          // 21
        self.natives.push(natives::native_sqrt);         // 22
        self.natives.push(natives::native_pow);          // 23
        self.natives.push(natives::native_min);          // 24
        self.natives.push(natives::native_max);          // 25
        self.natives.push(natives::native_round);        // 26
        // Строковые функции
        self.natives.push(natives::native_upper);        // 27
        self.natives.push(natives::native_lower);        // 28
        self.natives.push(natives::native_trim);         // 29
        self.natives.push(natives::native_split);        // 30
        self.natives.push(natives::native_join);         // 31
        self.natives.push(natives::native_contains);     // 32
        // Функции массивов
        self.natives.push(natives::native_push);         // 33
        self.natives.push(natives::native_pop);          // 34
        self.natives.push(natives::native_unique);       // 35
        self.natives.push(natives::native_reverse);      // 36
        self.natives.push(natives::native_sort);        // 37
        self.natives.push(natives::native_sum);          // 38
        self.natives.push(natives::native_average);     // 39
        self.natives.push(natives::native_count);        // 40
        self.natives.push(natives::native_any);          // 41
        self.natives.push(natives::native_all);          // 42
        // Функции для работы с таблицами
        self.natives.push(natives::native_table);        // 43
        self.natives.push(natives::native_read_file);    // 44
        self.natives.push(natives::native_table_info);   // 45
        self.natives.push(natives::native_table_head);   // 46
        self.natives.push(natives::native_table_tail);   // 47
        self.natives.push(natives::native_table_select); // 48
        self.natives.push(natives::native_table_sort);   // 49
        self.natives.push(natives::native_table_where);  // 50
        self.natives.push(natives::native_show_table);   // 51
        self.natives.push(natives::native_merge_tables); // 52
        self.natives.push(natives::native_now);          // 53
        self.natives.push(natives::native_getcwd);       // 54
        self.natives.push(natives::native_list_files);   // 55
        // JOIN операции
        self.natives.push(natives::native_inner_join);   // 56
        self.natives.push(natives::native_left_join);    // 57
        self.natives.push(natives::native_right_join);  // 58
        self.natives.push(natives::native_full_join);    // 59
        self.natives.push(natives::native_cross_join);   // 60
        self.natives.push(natives::native_semi_join);   // 61
        self.natives.push(natives::native_anti_join);   // 62
        self.natives.push(natives::native_zip_join);    // 63
        self.natives.push(natives::native_asof_join);   // 64
        self.natives.push(natives::native_apply_join);   // 65
        self.natives.push(natives::native_join_on);     // 66
        self.natives.push(natives::native_table_suffixes); // 67
        self.natives.push(natives::native_relate);      // 68
        self.natives.push(natives::native_primary_key); // 69
    }

    pub fn set_functions(&mut self, functions: Vec<crate::bytecode::Function>) {
        self.functions = functions;
        // Заполняем имена глобальных переменных из chunk главной функции (первая функция)
        if let Some(main_function) = self.functions.first() {
            self.global_names = main_function.chunk.global_names.clone();
            self.explicit_global_names = main_function.chunk.explicit_global_names.clone();
        }
    }

    pub fn register_native_globals(&mut self) {
        // Регистрируем нативные функции в глобальных переменных
        // Порядок должен соответствовать register_natives()
        self.globals.resize(70, Value::Null);
        
        self.globals[0] = Value::NativeFunction(0);  // print
        self.globals[1] = Value::NativeFunction(1);  // len
        self.globals[2] = Value::NativeFunction(2);  // range
        self.globals[3] = Value::NativeFunction(3);  // int
        self.globals[4] = Value::NativeFunction(4);  // float
        self.globals[5] = Value::NativeFunction(5);  // bool
        self.globals[6] = Value::NativeFunction(6);  // str
        self.globals[7] = Value::NativeFunction(7);  // array
        self.globals[8] = Value::NativeFunction(8);  // typeof
        self.globals[9] = Value::NativeFunction(9);  // isinstance
        self.globals[10] = Value::NativeFunction(10);  // date
        self.globals[11] = Value::NativeFunction(11);  // money
        self.globals[12] = Value::NativeFunction(12);  // path
        self.globals[13] = Value::NativeFunction(13);  // path_name
        self.globals[14] = Value::NativeFunction(14);  // path_parent
        self.globals[15] = Value::NativeFunction(15);  // path_exists
        self.globals[16] = Value::NativeFunction(16);  // path_is_file
        self.globals[17] = Value::NativeFunction(17);  // path_is_dir
        self.globals[18] = Value::NativeFunction(18);  // path_extension
        self.globals[19] = Value::NativeFunction(19);  // path_stem
        self.globals[20] = Value::NativeFunction(20);  // path_len
        // Математические функции
        self.globals[21] = Value::NativeFunction(21);  // abs
        self.globals[22] = Value::NativeFunction(22);  // sqrt
        self.globals[23] = Value::NativeFunction(23);  // pow
        self.globals[24] = Value::NativeFunction(24);  // min
        self.globals[25] = Value::NativeFunction(25);  // max
        self.globals[26] = Value::NativeFunction(26);  // round
        // Строковые функции
        self.globals[27] = Value::NativeFunction(27);  // upper
        self.globals[28] = Value::NativeFunction(28);  // lower
        self.globals[29] = Value::NativeFunction(29);  // trim
        self.globals[30] = Value::NativeFunction(30);  // split
        self.globals[31] = Value::NativeFunction(31);  // join
        self.globals[32] = Value::NativeFunction(32);  // contains
        // Функции массивов
        self.globals[33] = Value::NativeFunction(33);  // push
        self.globals[34] = Value::NativeFunction(34);  // pop
        self.globals[35] = Value::NativeFunction(35);  // unique
        self.globals[36] = Value::NativeFunction(36);  // reverse
        self.globals[37] = Value::NativeFunction(37);  // sort
        self.globals[38] = Value::NativeFunction(38);  // sum
        self.globals[39] = Value::NativeFunction(39);  // average
        self.globals[40] = Value::NativeFunction(40);  // count
        self.globals[41] = Value::NativeFunction(41);  // any
        self.globals[42] = Value::NativeFunction(42);  // all
        // Функции для работы с таблицами
        self.globals[43] = Value::NativeFunction(43);  // table
        self.globals[44] = Value::NativeFunction(44);  // read_file
        self.globals[45] = Value::NativeFunction(45);  // table_info
        self.globals[46] = Value::NativeFunction(46);  // table_head
        self.globals[47] = Value::NativeFunction(47);  // table_tail
        self.globals[48] = Value::NativeFunction(48);  // table_select
        self.globals[49] = Value::NativeFunction(49);  // table_sort
        self.globals[50] = Value::NativeFunction(50);  // table_where
        self.globals[51] = Value::NativeFunction(51);  // show_table
        self.globals[52] = Value::NativeFunction(52);  // merge_tables
        self.globals[53] = Value::NativeFunction(53);  // now
        self.globals[54] = Value::NativeFunction(54);  // getcwd
        self.globals[55] = Value::NativeFunction(55);  // list_files
        // JOIN операции
        self.globals[56] = Value::NativeFunction(56);  // inner_join
        self.globals[57] = Value::NativeFunction(57);  // left_join
        self.globals[58] = Value::NativeFunction(58);  // right_join
        self.globals[59] = Value::NativeFunction(59);  // full_join
        self.globals[60] = Value::NativeFunction(60);  // cross_join
        self.globals[61] = Value::NativeFunction(61);  // semi_join
        self.globals[62] = Value::NativeFunction(62);  // anti_join
        self.globals[63] = Value::NativeFunction(63);  // zip_join
        self.globals[64] = Value::NativeFunction(64);  // asof_join
        self.globals[65] = Value::NativeFunction(65);  // apply_join
        self.globals[66] = Value::NativeFunction(66);  // join_on
        self.globals[67] = Value::NativeFunction(67);  // table_suffixes
        self.globals[68] = Value::NativeFunction(68);  // relate
        self.globals[69] = Value::NativeFunction(69);  // primary_key
    }

    fn build_stack_trace(&self) -> Vec<StackTraceEntry> {
        let mut trace = Vec::new();
        for frame in &self.frames {
            let line = if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            };
            trace.push(StackTraceEntry {
                function_name: frame.function.name.clone(),
                line,
            });
        }
        trace.reverse(); // Начинаем с самой глубокой функции
        trace
    }

    fn runtime_error(&self, message: String, line: usize) -> LangError {
        LangError::runtime_error_with_trace(message, line, self.build_stack_trace())
    }

    fn runtime_error_with_type(&self, message: String, line: usize, error_type: ErrorType) -> LangError {
        LangError::runtime_error_with_type_and_trace(message, line, error_type, self.build_stack_trace())
    }

    /// Обрабатывает исключение - проверяет стек обработчиков и переходит к соответствующему catch блоку
    fn handle_exception(&mut self, error: LangError) -> Result<(), LangError> {
        // Получаем текущий IP для проверки, не находимся ли мы уже внутри catch блока
        let current_ip = if let Some(frame) = self.frames.last() {
            frame.ip
        } else {
            0
        };
        
        // Проверяем стек обработчиков (сверху вниз)
        // Обработчики привязаны к конкретным фреймам через frame_index
        for handler in self.exception_handlers.iter_mut().rev() {
            let handler_frame_index = handler.frame_index;
            
            if handler_frame_index >= self.frames.len() {
                continue;
            }
            
            // Получаем chunk функции для этого фрейма
            let frame = &self.frames[handler_frame_index];
            let chunk = &frame.function.chunk;
            
            // Проверяем, не находимся ли мы уже внутри catch блока этого обработчика
            // Если мы в том же фрейме и current_ip >= catch_ip для какого-то catch блока,
            // и current_ip < следующего catch_ip (или else_ip, если это последний catch),
            // значит мы внутри catch блока
            if handler_frame_index == self.frames.len() - 1 {
                let is_inside_catch = handler.catch_ips.iter().enumerate().any(|(i, &catch_ip)| {
                    if current_ip < catch_ip {
                        return false;
                    }
                    // Проверяем, не прошли ли мы этот catch блок
                    // Если есть следующий catch блок, проверяем, что current_ip < следующего catch_ip
                    if let Some(&next_catch_ip) = handler.catch_ips.get(i + 1) {
                        current_ip < next_catch_ip
                    } else {
                        // Это последний catch блок, проверяем, что current_ip < else_ip (если есть)
                        // или просто что мы >= catch_ip (если else_ip нет, значит catch блок последний)
                        if let Some(else_ip) = handler.else_ip {
                            current_ip < else_ip
                        } else {
                            true // Нет else блока, значит catch блок последний, и мы внутри него
                        }
                    }
                });
                
                // Если мы находимся внутри catch блока этого обработчика, пропускаем его
                // и ищем следующий обработчик выше по стеку
                if is_inside_catch {
                    continue;
                }
            }
            
            // Проверяем каждый catch блок
            for (i, catch_ip) in handler.catch_ips.iter().enumerate() {
                let error_type = handler.error_types.get(i);
                let error_var_slot = handler.error_var_slots.get(i);
                
                // Используем таблицу типов ошибок из chunk функции
                let error_type_table = &chunk.error_type_table;
                
                // Проверяем, подходит ли этот catch блок для данной ошибки
                let matches = match error_type {
                    Some(Some(expected_type_index)) => {
                        // Типизированный catch - проверяем тип ошибки
                        if let Some(error_type_name) = error_type_table.get(*expected_type_index) {
                            if let Some(et) = ErrorType::from_name(error_type_name) {
                                error.is_instance_of(&et)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    Some(None) => {
                        // catch всех
                        true
                    }
                    None => false,
                };
                
                if matches {
                    // Нашли подходящий catch блок
                    // Устанавливаем флаг ошибки
                    handler.had_error = true;
                    
                    // Очищаем стек до нужной высоты
                    while self.stack.len() > handler.stack_height {
                        self.stack.pop();
                    }
                    
                    // Удаляем все фреймы до фрейма с обработчиком
                    while self.frames.len() > handler_frame_index + 1 {
                        self.frames.pop();
                    }
                    
                    // Сохраняем ошибку в переменную (если указана)
                    if let Some(Some(slot)) = error_var_slot {
                        // Преобразуем ошибку в строку для сохранения в переменную
                        let error_string = format!("{}", error);
                        let frame = self.frames.last_mut().unwrap();
                        if *slot >= frame.slots.len() {
                            frame.slots.resize(*slot + 1, Value::Null);
                        }
                        frame.slots[*slot] = Value::String(error_string);
                    }
                    
                    // Переходим к catch блоку в правильном фрейме
                    let frame = self.frames.last_mut().unwrap();
                    frame.ip = *catch_ip;
                    
                    return Ok(());
                }
            }
        }
        
        // Обработчик не найден - возвращаем ошибку
        Err(error)
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<Value, LangError> {
        // Заполняем имена глобальных переменных из chunk
        self.global_names = chunk.global_names.clone();
        self.explicit_global_names = chunk.explicit_global_names.clone();
        
        // Создаем начальный frame
        let function = crate::bytecode::Function::new("<main>".to_string(), 0);
        let mut function = function;
        function.chunk = chunk.clone();
        let frame = CallFrame::new(function, 0);
        self.frames.push(frame);

        loop {
            // Проверяем, что есть frame
            if self.frames.is_empty() {
                break;
            }

            let (instruction, line) = {
                let frame = self.frames.last().unwrap();
                if frame.ip >= frame.function.chunk.code.len() {
                    break;
                }
                let ip = frame.ip;
                let instruction = frame.function.chunk.code[ip].clone();
                let line = frame.function.chunk.get_line(ip);
                (instruction, line)
            };

            let frame = self.frames.last_mut().unwrap();
            frame.ip += 1;

            match instruction {
                OpCode::Constant(index) => {
                    let value = frame.function.chunk.constants[index].clone();
                    self.push(value);
                }
                OpCode::LoadLocal(index) => {
                    // Для сложных типов (Array, Table) возвращаем ссылку (shallow copy Rc)
                    // Для простых типов клонируем значение
                    let value = &frame.slots[index];
                    let loaded_value = match value {
                        Value::Array(arr_rc) => Value::Array(Rc::clone(arr_rc)),
                        Value::Table(table_rc) => Value::Table(Rc::clone(table_rc)),
                        _ => value.clone(), // Простые типы клонируем
                    };
                    self.push(loaded_value);
                }
                OpCode::StoreLocal(index) => {
                    let value = self.pop()?;
                    // Clone уже создает глубокую копию для массивов и таблиц
                    let frame = self.frames.last_mut().unwrap();
                    if index >= frame.slots.len() {
                        frame.slots.resize(index + 1, Value::Null);
                    }
                    frame.slots[index] = value;
                }
                OpCode::LoadGlobal(index) => {
                    if index >= self.globals.len() {
                        let error = self.runtime_error(
                            format!("Undefined variable"),
                            line,
                        );
                        match self.handle_exception(error) {
                            Ok(()) => {
                                // Исключение обработано, кладем Null на стек
                                self.push(Value::Null);
                            }
                            Err(e) => return Err(e), // Исключение не обработано
                        }
                    } else {
                        // Для сложных типов (Array, Table) возвращаем ссылку (shallow copy Rc)
                        // Для простых типов клонируем значение
                        let value = &self.globals[index];
                        let loaded_value = match value {
                            Value::Array(arr_rc) => Value::Array(Rc::clone(arr_rc)),
                            Value::Table(table_rc) => Value::Table(Rc::clone(table_rc)),
                            _ => value.clone(), // Простые типы клонируем
                        };
                        self.push(loaded_value);
                    }
                }
                OpCode::StoreGlobal(index) => {
                    let mut value = self.pop()?;
                    // Если значение - таблица, устанавливаем её имя из global_names
                    if let Value::Table(table_rc) = &mut value {
                        if let Some(var_name) = self.global_names.get(&index) {
                            table_rc.borrow_mut().set_name(var_name.clone());
                        }
                    }
                    // Clone уже создает глубокую копию для массивов и таблиц
                    if index >= self.globals.len() {
                        self.globals.resize(index + 1, Value::Null);
                    }
                    // Важно: присваиваем value после установки имени, чтобы имя сохранилось
                    self.globals[index] = value;
                }
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_add(&a, &b)?;
                    self.push(result);
                }
                OpCode::Sub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_sub(&a, &b)?;
                    self.push(result);
                }
                OpCode::Mul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_mul(&a, &b)?;
                    self.push(result);
                }
                OpCode::Div => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_div(&a, &b)?;
                    self.push(result);
                }
                OpCode::IntDiv => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_int_div(&a, &b)?;
                    self.push(result);
                }
                OpCode::Mod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_mod(&a, &b)?;
                    self.push(result);
                }
                OpCode::Pow => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_pow(&a, &b)?;
                    self.push(result);
                }
                OpCode::Negate => {
                    let value = self.pop()?;
                    match value {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        _ => {
                            let error = self.runtime_error(
                                "Operand must be a number".to_string(),
                                line,
                            );
                            match self.handle_exception(error) {
                                Ok(()) => continue, // Исключение обработано, продолжаем выполнение
                                Err(e) => return Err(e), // Исключение не обработано
                            }
                        }
                    }
                }
                OpCode::Not => {
                    let value = self.pop()?;
                    self.push(Value::Bool(!value.is_truthy()));
                }
                OpCode::Or => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    // Если a истинно, возвращаем a, иначе возвращаем b
                    if a.is_truthy() {
                        self.push(a);
                    } else {
                        self.push(b);
                    }
                }
                OpCode::And => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    // Если a ложно, возвращаем a, иначе возвращаем b
                    if !a.is_truthy() {
                        self.push(a);
                    } else {
                        self.push(b);
                    }
                }
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b));
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a != b));
                }
                OpCode::Greater => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_greater(&a, &b)?;
                    self.push(result);
                }
                OpCode::Less => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_less(&a, &b)?;
                    self.push(result);
                }
                OpCode::GreaterEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_greater_equal(&a, &b)?;
                    self.push(result);
                }
                OpCode::LessEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_less_equal(&a, &b)?;
                    self.push(result);
                }
                OpCode::In => {
                    let array = self.pop()?; // Правый операнд - массив
                    let value = self.pop()?; // Левый операнд - значение для поиска
                    
                    match array {
                        Value::Array(arr) => {
                            let arr_ref = arr.borrow();
                            let found = arr_ref.iter().any(|item| item == &value);
                            self.push(Value::Bool(found));
                        }
                        _ => {
                            let error = self.runtime_error(
                                "Right operand of 'in' operator must be an array".to_string(),
                                line,
                            );
                            match self.handle_exception(error) {
                                Ok(()) => continue,
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
                OpCode::Jump8(offset) => {
                    frame.ip = (frame.ip as i32 + offset as i32) as usize;
                }
                OpCode::Jump16(offset) => {
                    frame.ip = (frame.ip as i32 + offset as i32) as usize;
                }
                OpCode::Jump32(offset) => {
                    frame.ip = (frame.ip as i64 + offset as i64) as usize;
                }
                OpCode::JumpIfFalse8(offset) => {
                    let condition = self.pop()?;
                    let frame = self.frames.last_mut().unwrap();
                    if !condition.is_truthy() {
                        frame.ip = (frame.ip as i32 + offset as i32) as usize;
                    }
                }
                OpCode::JumpIfFalse16(offset) => {
                    let condition = self.pop()?;
                    let frame = self.frames.last_mut().unwrap();
                    if !condition.is_truthy() {
                        frame.ip = (frame.ip as i32 + offset as i32) as usize;
                    }
                }
                OpCode::JumpIfFalse32(offset) => {
                    let condition = self.pop()?;
                    let frame = self.frames.last_mut().unwrap();
                    if !condition.is_truthy() {
                        frame.ip = (frame.ip as i64 + offset as i64) as usize;
                    }
                }
                OpCode::JumpLabel(_) | OpCode::JumpIfFalseLabel(_) => {
                    return Err(crate::common::error::LangError::runtime_error(
                        "JumpLabel found in VM - compilation not finalized".to_string(),
                        frame.function.chunk.get_line(frame.ip),
                    ));
                }
                OpCode::Call(arity) => {
                    // Получаем функцию со стека
                    let function_value = self.pop()?;
                    match function_value {
                        Value::Function(function_index) => {
                            if function_index >= self.functions.len() {
                                let error = self.runtime_error(
                                    format!("Function index {} out of bounds", function_index),
                                    line,
                                );
                                match self.handle_exception(error) {
                                    Ok(()) => continue,
                                    Err(e) => return Err(e),
                                }
                            }
                            
                            let function = self.functions[function_index].clone();
                            
                            // Проверяем количество аргументов
                            if arity != function.arity {
                                let error = self.runtime_error(
                                    format!(
                                        "Expected {} arguments but got {}",
                                        function.arity, arity
                                    ),
                                    line,
                                );
                                match self.handle_exception(error) {
                                    Ok(()) => continue,
                                    Err(e) => return Err(e),
                                }
                            }
                            
                            // Собираем аргументы со стека (в обратном порядке, так как они были положены последними)
                            let mut args = Vec::new();
                            for _ in 0..arity {
                                args.push(self.pop()?);
                            }
                            args.reverse(); // Теперь args[0] - первый аргумент
                            
                            // Проверяем кэш, если функция помечена как кэшируемая
                            if function.is_cached {
                                use crate::bytecode::function::CacheKey;
                                
                                // Пытаемся создать ключ кэша
                                if let Some(cache_key) = CacheKey::new(&args) {
                                    // Получаем доступ к кэшу функции
                                    if let Some(cache_rc) = &function.cache {
                                        let cache = cache_rc.borrow();
                                        
                                        // Проверяем, есть ли результат в кэше
                                        if let Some(cached_result) = cache.map.get(&cache_key) {
                                            // Результат найден в кэше - возвращаем его без выполнения функции
                                            self.push(cached_result.clone());
                                            continue; // Пропускаем выполнение функции
                                        }
                                        
                                        // Результат не найден - освобождаем borrow и продолжим выполнение
                                        drop(cache);
                                        
                                        // Выполним функцию и сохраним результат в кэш
                                        // (продолжаем выполнение ниже)
                                    }
                                }
                                // Если ключ не удалось создать (не-hashable аргументы),
                                // просто выполняем функцию без кэширования
                            }
                            
                            // Создаем новый CallFrame
                            let stack_start = self.stack.len();
                            let mut new_frame = if function.is_cached {
                                // Сохраняем аргументы для кэширования
                                CallFrame::new_with_cache(function.clone(), stack_start, args.clone())
                            } else {
                                CallFrame::new(function.clone(), stack_start)
                            };
                            
                            // Копируем таблицу типов ошибок из chunk функции в VM
                            if !function.chunk.error_type_table.is_empty() {
                                self.error_type_table = function.chunk.error_type_table.clone();
                            }
                            
                            // Копируем захваченные переменные из родительских frames (если есть)
                            // Используем ancestor_depth для поиска переменной в правильном предке
                            if !self.frames.is_empty() && !function.captured_vars.is_empty() {
                                #[cfg(debug_assertions)]
                                eprintln!("[DEBUG] Function '{}' has {} captured vars, frames.len() = {}", 
                                    function.name, function.captured_vars.len(), self.frames.len());
                                
                                for captured_var in &function.captured_vars {
                                    // Убеждаемся, что слот существует в новом frame
                                    if captured_var.local_slot_index >= new_frame.slots.len() {
                                        new_frame.slots.resize(captured_var.local_slot_index + 1, Value::Null);
                                    }
                                    
                                    // Находим предка на нужной глубине
                                    // ancestor_depth = 0 означает ближайший родитель (последний frame в стеке)
                                    // ancestor_depth = 1 означает дедушку (предпоследний frame) и т.д.
                                    let ancestor_index = self.frames.len().saturating_sub(1 + captured_var.ancestor_depth);
                                    
                                    #[cfg(debug_assertions)]
                                    eprintln!("[DEBUG] Captured var '{}': ancestor_depth={}, ancestor_index={}, parent_slot={}, local_slot={}", 
                                        captured_var.name, captured_var.ancestor_depth, ancestor_index, 
                                        captured_var.parent_slot_index, captured_var.local_slot_index);
                                    
                                    if ancestor_index < self.frames.len() {
                                        let ancestor_frame = &self.frames[ancestor_index];
                                        
                                        #[cfg(debug_assertions)]
                                        eprintln!("[DEBUG] Ancestor frame '{}' has {} slots", 
                                            ancestor_frame.function.name, ancestor_frame.slots.len());
                                        
                                        // Копируем значение из предка
                                        if captured_var.parent_slot_index < ancestor_frame.slots.len() {
                                            let captured_value = ancestor_frame.slots[captured_var.parent_slot_index].clone();
                                            
                                            #[cfg(debug_assertions)]
                                            eprintln!("[DEBUG] Copying value {:?} from ancestor slot {} to local slot {}", 
                                                captured_value, captured_var.parent_slot_index, captured_var.local_slot_index);
                                            
                                            new_frame.slots[captured_var.local_slot_index] = captured_value;
                                        } else {
                                            // Если слот не существует в предке, используем Null
                                            #[cfg(debug_assertions)]
                                            eprintln!("[DEBUG] WARNING: parent_slot {} >= ancestor slots.len() {}", 
                                                captured_var.parent_slot_index, ancestor_frame.slots.len());
                                            new_frame.slots[captured_var.local_slot_index] = Value::Null;
                                        }
                                    } else {
                                        // Если предок не существует, используем Null
                                        #[cfg(debug_assertions)]
                                        eprintln!("[DEBUG] WARNING: ancestor_index {} >= frames.len() {}", 
                                            ancestor_index, self.frames.len());
                                        new_frame.slots[captured_var.local_slot_index] = Value::Null;
                                    }
                                }
                            }
                            
                            // Инициализируем параметры функции в slots (после захваченных переменных)
                            let param_start_index = function.captured_vars.len();
                            for (i, arg) in args.iter().enumerate() {
                                let slot_index = param_start_index + i;
                                if slot_index >= new_frame.slots.len() {
                                    new_frame.slots.resize(slot_index + 1, Value::Null);
                                }
                                new_frame.slots[slot_index] = arg.clone();
                            }
                            
                            // Добавляем новый frame
                            self.frames.push(new_frame);
                        }
                        Value::NativeFunction(native_index) => {
                            if native_index >= self.natives.len() {
                                let error = self.runtime_error(
                                    format!("Native function index {} out of bounds", native_index),
                                    line,
                                );
                                match self.handle_exception(error) {
                                    Ok(()) => continue,
                                    Err(e) => return Err(e),
                                }
                            }
                            
                            // Собираем аргументы со стека
                            let mut args = Vec::new();
                            for _ in 0..arity {
                                args.push(self.pop()?);
                            }
                            args.reverse(); // Теперь args[0] - первый аргумент
                            
                            // Специальная проверка для range (принимает 1, 2 или 3 аргумента)
                            if native_index == 2 {
                                // range - индекс 2
                                if arity < 1 || arity > 3 {
                                    let error = self.runtime_error(
                                        format!("range() expects 1, 2, or 3 arguments, got {}", arity),
                                        line,
                                    );
                                    match self.handle_exception(error) {
                                        Ok(()) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                                // Проверяем типы аргументов - все должны быть числами
                                for arg in &args {
                                    if !matches!(arg, Value::Number(_)) {
                                        let error = self.runtime_error(
                                            "range() arguments must be numbers".to_string(),
                                            line,
                                        );
                                        match self.handle_exception(error) {
                                            Ok(()) => continue,
                                            Err(e) => return Err(e),
                                        }
                                    }
                                }
                                // Проверяем, что step не равен 0 (если передан)
                                if arity == 3 {
                                    if let Value::Number(step) = &args[2] {
                                        if *step == 0.0 {
                                            let error = self.runtime_error(
                                                "range() step cannot be zero".to_string(),
                                                line,
                                            );
                                            match self.handle_exception(error) {
                                                Ok(()) => continue,
                                                Err(e) => return Err(e),
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Вызываем нативную функцию
                            let native_fn = self.natives[native_index];
                            let result = native_fn(&args);
                            
                            // Если это relate(), получаем связи из thread-local storage
                            if native_index == 65 {
                                // relate() - индекс 65
                                use crate::vm::natives::take_relations;
                                let relations = take_relations();
                                
                                // Находим имена таблиц по указателям
                                for (table1_ptr, col1_name, table2_ptr, col2_name) in relations {
                                    let mut found_table1_name = None;
                                    let mut found_table2_name = None;
                                    
                                    // Ищем таблицы в глобальных переменных
                                    for (index, value) in self.globals.iter().enumerate() {
                                        if let Value::Table(table) = value {
                                            if Rc::as_ptr(table) == table1_ptr {
                                                if let Some(var_name) = self.explicit_global_names.get(&index) {
                                                    found_table1_name = Some(var_name.clone());
                                                }
                                            }
                                            if Rc::as_ptr(table) == table2_ptr {
                                                if let Some(var_name) = self.explicit_global_names.get(&index) {
                                                    found_table2_name = Some(var_name.clone());
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Если нашли обе таблицы, сохраняем связь
                                    // relate(pk_table["pk_column"], fk_table["fk_column"])
                                    // Первый аргумент - первичный ключ (целевая таблица)
                                    // Второй аргумент - внешний ключ (таблица, которая ссылается)
                                    if let (Some(table1_name), Some(table2_name)) = (found_table1_name, found_table2_name) {
                                        self.explicit_relations.push(ExplicitRelation {
                                            source_table_name: table2_name, // Таблица с внешним ключом
                                            source_column_name: col2_name,  // Внешний ключ
                                            target_table_name: table1_name, // Таблица с первичным ключом
                                            target_column_name: col1_name,   // Первичный ключ
                                        });
                                    }
                                }
                            }
                            
                            // Если это primary_key(), получаем первичные ключи из thread-local storage
                            if native_index == 66 {
                                // primary_key() - индекс 66
                                use crate::vm::natives::take_primary_keys;
                                let primary_keys = take_primary_keys();
                                
                                // Находим имена таблиц по указателям
                                for (table_ptr, col_name) in primary_keys {
                                    let mut found_table_name = None;
                                    
                                    // Ищем таблицу в глобальных переменных
                                    for (index, value) in self.globals.iter().enumerate() {
                                        if let Value::Table(table) = value {
                                            if Rc::as_ptr(table) == table_ptr {
                                                if let Some(var_name) = self.explicit_global_names.get(&index) {
                                                    found_table_name = Some(var_name.clone());
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Если нашли таблицу, сохраняем первичный ключ
                                    if let Some(table_name) = found_table_name {
                                        self.explicit_primary_keys.push(ExplicitPrimaryKey {
                                            table_name,
                                            column_name: col_name,
                                        });
                                    }
                                }
                            }
                            
                            // Проверяем, не было ли ошибки в нативной функции (например, path traversal)
                            use crate::websocket::take_native_error;
                            if let Some(error_msg) = take_native_error() {
                                let error = self.runtime_error_with_type(
                                    error_msg,
                                    line,
                                    crate::common::error::ErrorType::IOError,
                                );
                                match self.handle_exception(error) {
                                    Ok(()) => continue, // Исключение обработано
                                    Err(e) => return Err(e), // Исключение не обработано
                                }
                            }
                            
                            // Помещаем результат на стек
                            self.push(result);
                        }
                        _ => {
                            let error = self.runtime_error(
                                "Can only call functions".to_string(),
                                line,
                            );
                            match self.handle_exception(error) {
                                Ok(()) => continue,
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
                OpCode::Return => {
                    // Получаем возвращаемое значение (если есть)
                    let return_value = if !self.stack.is_empty() {
                        self.pop().ok()
                    } else {
                        Some(Value::Null)
                    };
                    
                    let frames_count = self.frames.len();
                    if frames_count > 1 {
                        // Сохраняем результат в кэш, если функция кэшируемая
                        if let Some(frame) = self.frames.last() {
                            if frame.function.is_cached {
                                if let Some(ref cached_args) = frame.cached_args {
                                    use crate::bytecode::function::CacheKey;
                                    
                                    // Пытаемся создать ключ кэша
                                    if let Some(cache_key) = CacheKey::new(cached_args) {
                                        // Получаем доступ к кэшу функции
                                        if let Some(cache_rc) = &frame.function.cache {
                                            let mut cache = cache_rc.borrow_mut();
                                            
                                            // Сохраняем результат в кэш
                                            if let Some(ref result) = return_value {
                                                cache.map.insert(cache_key, result.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Возврат из функции - удаляем текущий frame
                        self.frames.pop();
                        
                        // Помещаем возвращаемое значение на стек для вызывающей функции
                        if let Some(value) = return_value {
                            self.push(value);
                        }
                    } else {
                        // Возврат из главной функции - завершаем выполнение
                        // Возвращаем значение со стека, если есть
                        if let Some(value) = return_value {
                            return Ok(value);
                        } else {
                            return Ok(Value::Null);
                        }
                    }
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::MakeArray(count) => {
                    let mut elements = Vec::new();
                    for _ in 0..count {
                        elements.push(self.pop()?);
                    }
                    elements.reverse(); // Восстанавливаем правильный порядок
                    self.push(Value::Array(Rc::new(RefCell::new(elements))));
                }
                OpCode::GetArrayLength => {
                    let array = self.pop()?;
                    match array {
                        Value::Array(arr) => {
                            self.push(Value::Number(arr.borrow().len() as f64));
                        }
                        Value::ColumnReference { table, column_name } => {
                            let table_ref = table.borrow();
                            if let Some(column) = table_ref.get_column(&column_name) {
                                self.push(Value::Number(column.len() as f64));
                            } else {
                                let error = self.runtime_error(
                                    format!("Column '{}' not found", column_name),
                                    line,
                                );
                                match self.handle_exception(error) {
                                    Ok(()) => continue,
                                    Err(e) => return Err(e),
                                }
                            }
                        }
                        _ => {
                            let error = self.runtime_error(
                                "Expected array or column reference for GetArrayLength".to_string(),
                                line,
                            );
                            match self.handle_exception(error) {
                                Ok(()) => continue, // Исключение обработано, продолжаем выполнение
                                Err(e) => return Err(e), // Исключение не обработано
                            }
                        }
                    }
                }
                OpCode::GetArrayElement => {
                    let index_value = self.pop()?;
                    let container = self.pop()?;
                    
                    match container {
                        Value::Array(arr) => {
                            let index = match index_value {
                                Value::Number(n) => {
                                    let idx = n as i64;
                                    if idx < 0 {
                                        let error = self.runtime_error(
                                            "Array index must be non-negative".to_string(),
                                            line,
                                        );
                                        match self.handle_exception(error) {
                                            Ok(()) => continue,
                                            Err(e) => return Err(e),
                                        }
                                    }
                                    idx as usize
                                }
                                _ => {
                                    let error = self.runtime_error(
                                        "Array index must be a number".to_string(),
                                        line,
                                    );
                                    match self.handle_exception(error) {
                                        Ok(()) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                            };
                            
                            let arr_ref = arr.borrow();
                            if index >= arr_ref.len() {
                                let error = self.runtime_error_with_type(
                                    format!("Array index {} out of bounds (length: {})", index, arr_ref.len()),
                                    line,
                                    ErrorType::IndexError,
                                );
                                match self.handle_exception(error) {
                                    Ok(()) => continue,
                                    Err(e) => return Err(e),
                                }
                            }
                            // Для сложных типов (Array, Table, Object) возвращаем ссылку (shallow copy Rc)
                            // Для простых типов клонируем значение
                            let element = &arr_ref[index];
                            let value = match element {
                                Value::Array(arr_rc) => Value::Array(Rc::clone(arr_rc)),
                                Value::Table(table_rc) => Value::Table(Rc::clone(table_rc)),
                                Value::Object(_) => element.clone(), // Object uses HashMap, clone is needed
                                _ => element.clone(), // Простые типы клонируем
                            };
                            self.push(value);
                        }
                        Value::Table(table) => {
                            // Доступ к колонке таблицы по имени или строке по индексу
                            match index_value {
                                Value::String(property) => {
                                    let table_ref = table.borrow();
                                    
                                    // Специальные свойства таблицы
                                    if property == "rows" {
                                        // Возвращаем массив строк (каждая строка - массив значений)
                                        let rows: Vec<Value> = table_ref.rows.iter()
                                            .map(|row| {
                                                Value::Array(Rc::new(RefCell::new(row.clone())))
                                            })
                                            .collect();
                                        self.push(Value::Array(Rc::new(RefCell::new(rows))));
                                    } else if property == "columns" {
                                        // Возвращаем массив имен колонок (заголовки)
                                        let columns: Vec<Value> = table_ref.headers.iter()
                                            .map(|header| Value::String(header.clone()))
                                            .collect();
                                        self.push(Value::Array(Rc::new(RefCell::new(columns))));
                                    } else {
                                        // Доступ к колонке по имени
                                        if table_ref.get_column(&property).is_some() {
                                            // Возвращаем ColumnReference для использования в relate()
                                            self.push(Value::ColumnReference {
                                                table: table.clone(),
                                                column_name: property,
                                            });
                                        } else {
                                            let error = self.runtime_error_with_type(
                                                format!("Column '{}' not found in table", property),
                                                line,
                                                ErrorType::KeyError,
                                            );
                                            match self.handle_exception(error) {
                                                Ok(()) => continue,
                                                Err(e) => return Err(e),
                                            }
                                        }
                                    }
                                }
                                Value::Number(n) => {
                                    // Доступ к строке по индексу
                                    let idx = n as i64;
                                    if idx < 0 {
                                        let error = self.runtime_error(
                                            "Table row index must be non-negative".to_string(),
                                            line,
                                        );
                                        match self.handle_exception(error) {
                                            Ok(()) => continue,
                                            Err(e) => return Err(e),
                                        }
                                    }
                                    let table_ref = table.borrow();
                                    if idx as usize >= table_ref.rows.len() {
                                        let error = self.runtime_error_with_type(
                                            format!("Row index {} out of bounds (length: {})", idx, table_ref.rows.len()),
                                            line,
                                            ErrorType::IndexError,
                                        );
                                        match self.handle_exception(error) {
                                            Ok(()) => continue,
                                            Err(e) => return Err(e),
                                        }
                                    }
                                    if let Some(row) = table_ref.get_row(idx as usize) {
                                        // Создаем словарь из строки таблицы
                                        use std::collections::HashMap;
                                        let mut row_dict = HashMap::new();
                                        for (i, header) in table_ref.headers.iter().enumerate() {
                                            if i < row.len() {
                                                row_dict.insert(header.clone(), row[i].clone());
                                            }
                                        }
                                        self.push(Value::Object(row_dict));
                                    } else {
                                        let error = self.runtime_error_with_type(
                                            format!("Row index {} out of bounds", idx),
                                            line,
                                            ErrorType::IndexError,
                                        );
                                        match self.handle_exception(error) {
                                            Ok(()) => continue,
                                            Err(e) => return Err(e),
                                        }
                                    }
                                }
                                _ => {
                                    let error = self.runtime_error(
                                        "Table index must be a string (column name) or number (row index)".to_string(),
                                        line,
                                    );
                                    match self.handle_exception(error) {
                                        Ok(()) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                            }
                        }
                        Value::Object(map) => {
                            // Доступ к значению объекта по строковому ключу
                            match index_value {
                                Value::String(key) => {
                                    if let Some(value) = map.get(&key) {
                                        self.push(value.clone());
                                    } else {
                                        let error = self.runtime_error_with_type(
                                            format!("Key '{}' not found in object", key),
                                            line,
                                            ErrorType::KeyError,
                                        );
                                        match self.handle_exception(error) {
                                            Ok(()) => continue,
                                            Err(e) => return Err(e),
                                        }
                                    }
                                }
                                _ => {
                                    let error = self.runtime_error(
                                        "Object index must be a string".to_string(),
                                        line,
                                    );
                                    match self.handle_exception(error) {
                                        Ok(()) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                            }
                        }
                        Value::ColumnReference { table, column_name } => {
                            // Доступ к элементу колонки по индексу (как массив)
                            let index = match index_value {
                                Value::Number(n) => {
                                    let idx = n as i64;
                                    if idx < 0 {
                                        let error = self.runtime_error(
                                            "Column index must be non-negative".to_string(),
                                            line,
                                        );
                                        match self.handle_exception(error) {
                                            Ok(()) => continue,
                                            Err(e) => return Err(e),
                                        }
                                    }
                                    idx as usize
                                }
                                _ => {
                                    let error = self.runtime_error(
                                        "Column index must be a number".to_string(),
                                        line,
                                    );
                                    match self.handle_exception(error) {
                                        Ok(()) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                            };
                            
                            let table_ref = table.borrow();
                            if let Some(column) = table_ref.get_column(&column_name) {
                                if index >= column.len() {
                                    let error = self.runtime_error_with_type(
                                        format!("Column index {} out of bounds (length: {})", index, column.len()),
                                        line,
                                        ErrorType::IndexError,
                                    );
                                    match self.handle_exception(error) {
                                        Ok(()) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                                self.push(column[index].clone());
                            } else {
                                let error = self.runtime_error_with_type(
                                    format!("Column '{}' not found", column_name),
                                    line,
                                    ErrorType::KeyError,
                                );
                                match self.handle_exception(error) {
                                    Ok(()) => continue,
                                    Err(e) => return Err(e),
                                }
                            }
                        }
                        Value::Path(path) => {
                            // Доступ к свойствам Path по строковому ключу
                            match index_value {
                                Value::String(property_name) => {
                                    match property_name.as_str() {
                                        "is_file" => {
                                            self.push(Value::Bool(path.is_file()));
                                        }
                                        "is_dir" => {
                                            self.push(Value::Bool(path.is_dir()));
                                        }
                                        "extension" => {
                                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                                self.push(Value::String(ext.to_string()));
                                            } else {
                                                self.push(Value::Null);
                                            }
                                        }
                                        "name" => {
                                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                                self.push(Value::String(name.to_string()));
                                            } else {
                                                self.push(Value::Null);
                                            }
                                        }
                                        "parent" => {
                                            // Используем безопасную функцию для получения parent
                                            use crate::vm::natives::safe_path_parent;
                                            match safe_path_parent(&path) {
                                                Some(parent) => self.push(Value::Path(parent)),
                                                None => self.push(Value::Null),
                                            }
                                        }
                                        "exists" => {
                                            self.push(Value::Bool(path.exists()));
                                        }
                                        _ => {
                                            let error = self.runtime_error(
                                                format!("Property '{}' not found on Path", property_name),
                                                line,
                                            );
                                            match self.handle_exception(error) {
                                                Ok(()) => continue,
                                                Err(e) => return Err(e),
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    let error = self.runtime_error(
                                        "Path property access requires string index".to_string(),
                                        line,
                                    );
                                    match self.handle_exception(error) {
                                        Ok(()) => continue,
                                        Err(e) => return Err(e),
                                    }
                                }
                            }
                        }
                        _ => {
                            let error = self.runtime_error(
                                "Expected array, column reference, table, object, or path for GetArrayElement".to_string(),
                                line,
                            );
                            match self.handle_exception(error) {
                                Ok(()) => continue,
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
                OpCode::Clone => {
                    // Глубокое клонирование значения на стеке
                    let value = self.pop()?;
                    let cloned = value.clone(); // Используем реализованный Clone для Value
                    self.push(cloned);
                }
                OpCode::BeginTry(handler_index) => {
                    // Начало try блока - загружаем обработчик из chunk
                    let frame = self.frames.last().unwrap();
                    let chunk = &frame.function.chunk;
                    
                    // Загружаем информацию об обработчике из chunk
                    if handler_index < chunk.exception_handlers.len() {
                        let handler_info = &chunk.exception_handlers[handler_index];
                        
                        // Копируем таблицу типов ошибок в VM (если еще не скопирована)
                        if self.error_type_table.is_empty() {
                            self.error_type_table = chunk.error_type_table.clone();
                        }
                        
                        // Сохраняем текущую высоту стека
                        let stack_height = self.stack.len();
                        
                        // Создаем обработчик с информацией из chunk
                        let frame_index = self.frames.len() - 1;
                        let handler = ExceptionHandler {
                            catch_ips: handler_info.catch_ips.clone(),
                            error_types: handler_info.error_types.clone(),
                            error_var_slots: handler_info.error_var_slots.clone(),
                            else_ip: handler_info.else_ip,
                            stack_height,
                            had_error: false,
                            frame_index,
                        };
                        self.exception_handlers.push(handler);
                    } else {
                        // Если обработчик не найден, создаем пустой (fallback)
                        let stack_height = self.stack.len();
                        let frame_index = self.frames.len() - 1;
                        let handler = ExceptionHandler {
                            catch_ips: Vec::new(),
                            error_types: Vec::new(),
                            error_var_slots: Vec::new(),
                            else_ip: None,
                            stack_height,
                            had_error: false,
                            frame_index,
                        };
                        self.exception_handlers.push(handler);
                    }
                }
                OpCode::EndTry => {
                    // Конец try блока - если выполнение дошло сюда без ошибок
                    // Проверяем, была ли ошибка
                    if let Some(handler) = self.exception_handlers.last_mut() {
                        // Если не было ошибки и есть else блок, переходим к нему
                        if !handler.had_error {
                            if let Some(else_ip) = handler.else_ip {
                                let frame = self.frames.last_mut().unwrap();
                                frame.ip = else_ip;
                            }
                        }
                        // Удаляем обработчик из стека
                        self.exception_handlers.pop();
                    }
                }
                OpCode::Catch(_) => {
                    // Начало catch блока - этот опкод используется только для маркировки
                    // Реальная логика обработки выполняется в handle_exception()
                    // Здесь просто продолжаем выполнение
                }
                OpCode::EndCatch => {
                    // Конец catch блока - продолжаем выполнение после catch
                    // Обработчик будет удален при PopExceptionHandler
                }
                OpCode::Throw(_) => {
                    // Выбрасывание исключения
                    // Получаем значение со стека (сообщение об ошибке)
                    let error_value = self.pop()?;
                    
                    // Преобразуем значение в строку
                    let error_message = error_value.to_string();
                    
                    // Создаем LangError
                    let error = LangError::runtime_error(error_message, line);
                    
                    // Пытаемся найти обработчик исключения
                    match self.handle_exception(error) {
                        Ok(()) => {
                            // Обработчик найден, выполнение продолжается в catch блоке
                            // handle_exception уже настроил стек и фреймы
                        }
                        Err(e) => {
                            // Обработчик не найден - возвращаем ошибку (программа завершается)
                            return Err(e);
                        }
                    }
                }
                OpCode::PopExceptionHandler => {
                    // Удаление обработчика исключений со стека
                    self.exception_handlers.pop();
                }
            }
        }

        // После завершения выполнения возвращаем последнее значение на стеке
        if !self.stack.is_empty() {
            Ok(self.stack.pop().unwrap())
        } else {
            Ok(Value::Null)
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        self.stack.pop().ok_or_else(|| self.runtime_error(
            "Stack underflow".to_string(),
            line,
        ))
    }

    fn binary_add(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
            (Value::String(s1), Value::String(s2)) => Ok(Value::String(format!("{}{}", s1, s2))),
            (Value::String(s), Value::Number(n)) => Ok(Value::String(format!("{}{}", s, n))),
            (Value::Number(n), Value::String(s)) => Ok(Value::String(format!("{}{}", n, s))),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers or strings".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_sub(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 - n2)),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_mul(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 * n2)),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_div(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => {
                if *n2 == 0.0 {
                    let error = self.runtime_error(
                        "Division by zero".to_string(),
                        line,
                    );
                    match self.handle_exception(error) {
                        Ok(()) => {
                            // Исключение обработано, возвращаем Null как значение после обработки
                            Ok(Value::Null)
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(Value::Number(n1 / n2))
                }
            }
            // Конкатенация путей: Path / String -> Path
            (Value::Path(p), Value::String(s)) => {
                let mut new_path = p.clone();
                new_path.push(s);
                Ok(Value::Path(new_path))
            }
            // Конкатенация путей: String / String -> Path (если контекст предполагает путь)
            (Value::String(s1), Value::String(s2)) => {
                use std::path::PathBuf;
                let mut path = PathBuf::from(s1);
                path.push(s2);
                Ok(Value::Path(path))
            }
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers or paths".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_int_div(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => {
                if *n2 == 0.0 {
                    let error = self.runtime_error(
                        "Division by zero".to_string(),
                        line,
                    );
                    match self.handle_exception(error) {
                        Ok(()) => Ok(Value::Null),
                        Err(e) => Err(e),
                    }
                } else {
                    // Целочисленное деление: отбрасываем дробную часть
                    Ok(Value::Number((n1 / n2).floor()))
                }
            }
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_mod(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => {
                if *n2 == 0.0 {
                    let error = self.runtime_error(
                        "Modulo by zero".to_string(),
                        line,
                    );
                    match self.handle_exception(error) {
                        Ok(()) => Ok(Value::Null),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(Value::Number(n1 % n2))
                }
            }
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_pow(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1.powf(*n2))),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_greater(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 > n2)),
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 > s2)),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers or strings".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_less(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 < n2)),
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 < s2)),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers or strings".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_greater_equal(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 >= n2)),
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 >= s2)),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers or strings".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn binary_less_equal(&mut self, a: &Value, b: &Value) -> Result<Value, LangError> {
        let line = if let Some(frame) = self.frames.last() {
            if frame.ip > 0 {
                frame.function.chunk.get_line(frame.ip - 1)
            } else {
                0
            }
        } else {
            0
        };
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Bool(n1 <= n2)),
            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 <= s2)),
            _ => {
                let error = self.runtime_error(
                    "Operands must be numbers or strings".to_string(),
                    line,
                );
                match self.handle_exception(error) {
                    Ok(()) => Ok(Value::Null),
                    Err(e) => Err(e),
                }
            }
        }
    }

    /// Получить доступ к глобальным переменным (для экспорта)
    pub fn get_globals(&self) -> &Vec<Value> {
        &self.globals
    }

    /// Получить доступ к именам глобальных переменных
    pub fn get_global_names(&self) -> &std::collections::HashMap<usize, String> {
        &self.global_names
    }

    /// Получить доступ к именам переменных, явно объявленных с ключевым словом 'global'
    pub fn get_explicit_global_names(&self) -> &std::collections::HashMap<usize, String> {
        &self.explicit_global_names
    }

    /// Добавить явную связь между колонками таблиц
    pub fn add_explicit_relation(&mut self, relation: ExplicitRelation) {
        self.explicit_relations.push(relation);
    }

    /// Получить все явные связи
    pub fn get_explicit_relations(&self) -> &Vec<ExplicitRelation> {
        &self.explicit_relations
    }

    /// Добавить явный первичный ключ таблицы
    pub fn add_explicit_primary_key(&mut self, primary_key: ExplicitPrimaryKey) {
        self.explicit_primary_keys.push(primary_key);
    }

    /// Получить явные первичные ключи таблиц
    pub fn get_explicit_primary_keys(&self) -> &Vec<ExplicitPrimaryKey> {
        &self.explicit_primary_keys
    }
}

