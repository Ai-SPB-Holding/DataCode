use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::builtins::call_builtin_function_with_named_args;
use std::collections::HashMap;
use std::time::Instant;

// Импортируем FunctionCache из модуля cache
// Используем прямой путь через модуль cache
// В main.rs модули объявлены локально, но в lib.rs они объявлены через pub mod
// Поэтому используем crate::cache
use crate::cache::FunctionCache;

// Подмодули
pub mod user_functions;
pub mod variables;
pub mod expressions;
pub mod execution;
pub mod control_flow;
pub mod call_frame;
mod evaluate_signal;
mod execute_instruction;

// Реэкспорт основных типов
pub use user_functions::{UserFunction, TryBlock, UserFunctionManager};
pub use variables::VariableManager;
pub use call_frame::{CallFrame, CallStack, ExecResult, ExecSignal};

/// Результат выполнения функции в контексте trampoline pattern
/// Используется для преобразования рекурсивных вызовов в итеративные
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FunctionResult {
    /// Функция завершилась, вернуть значение
    Done(Value),
    /// Функция требует еще один вызов (рекурсия или вложенный вызов)
    /// Хранит имя функции, аргументы и контекст выполнения
    Continue {
        function_name: String,
        args: Vec<Value>,
    },
}

/// Контекст выполнения функции для trampoline
/// Хранит информацию о текущем состоянии выполнения функции
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct FunctionFrame {
    /// Имя функции
    function_name: String,
    /// Аргументы функции
    args: Vec<Value>,
    /// Сохраненное возвращаемое значение (для восстановления после вложенных вызовов)
    saved_return_value: Option<Value>,
    /// Глубина рекурсии на момент создания фрейма
    recursion_depth: usize,
}

/// Основная структура интерпретатора DataCode
pub struct Interpreter {
    /// Менеджер переменных с поддержкой областей видимости
    pub variable_manager: VariableManager,
    /// Менеджер пользовательских функций
    pub function_manager: UserFunctionManager,

    /// Возвращаемое значение функции
    pub return_value: Option<Value>,
    /// Флаг запроса прерывания цикла (break)
    pub break_requested: bool,
    /// Флаг запроса пропуска текущей итерации цикла (next/continue)
    pub continue_requested: bool,
    /// Счетчик активных циклов (для проверки правильности использования break)
    pub active_loop_count: usize,
    /// Текущая строка для отслеживания ошибок
    pub current_line: usize,
    /// Стек блоков try/catch
    pub exception_stack: Vec<TryBlock>,
    /// Глубина рекурсии для предотвращения переполнения стека (legacy, используется для совместимости)
    pub recursion_depth: usize,
    /// Стек вызовов функций (Call Frame Engine)
    pub call_stack: CallStack,
    /// Флаг использования Call Frame Engine (по умолчанию true)
    pub use_call_frame_engine: bool,
    /// Флаг использования trampoline pattern для вызовов функций (legacy, для обратной совместимости)
    #[allow(dead_code)]
    pub use_trampoline: bool,
    /// Очередь вызовов функций для trampoline (имя функции, аргументы) (legacy)
    pub trampoline_queue: Vec<(String, Vec<Value>)>,
    /// Стек результатов вызовов функций для trampoline (legacy)
    pub trampoline_result_stack: Vec<Value>,
    /// Флаг, указывающий что мы находимся внутри trampoline loop (legacy)
    pub in_trampoline_loop: bool,
    /// Кэш результатов функций для мемоизации
    pub function_cache: FunctionCache,
}

impl Interpreter {
    /// Создать новый интерпретатор
    pub fn new() -> Self {
        Self {
            variable_manager: VariableManager::new(),
            function_manager: UserFunctionManager::new(),
            return_value: None,
            break_requested: false,
            continue_requested: false,
            active_loop_count: 0,
            current_line: 1,
            exception_stack: Vec::new(),
            recursion_depth: 0,
            call_stack: CallStack::new(1_000_000), // Максимальная глубина стека: 1 миллион
            use_call_frame_engine: true, // По умолчанию используем Call Frame Engine
            use_trampoline: false, // Отключаем старый trampoline
            trampoline_queue: Vec::new(),
            trampoline_result_stack: Vec::new(),
            in_trampoline_loop: false,
            function_cache: FunctionCache::default(),
        }
    }

    /// Получить переменную
    /// Сначала проверяет локальные переменные текущего фрейма Call Frame Engine,
    /// затем использует стандартный variable_manager
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        // Если используется Call Frame Engine и есть активный фрейм,
        // сначала проверяем локальные переменные фрейма
        if self.use_call_frame_engine {
            if let Some(frame) = self.call_stack.last() {
                if let Some(value) = frame.get_local(name) {
                    return Some(value);
                }
            }
        }
        // Затем используем стандартный механизм variable_manager
        self.variable_manager.get_variable(name)
    }

    /// Установить переменную
    pub fn set_variable(&mut self, name: String, value: Value, is_global: bool) {
        self.variable_manager.set_variable(name, value, is_global);
    }

    /// Умно установить переменную - обновляет существующую переменную в её текущей области видимости
    pub fn set_variable_smart(&mut self, name: String, value: Value) {
        self.variable_manager.set_variable_smart(name, value);
    }

    /// Получить все глобальные переменные
    pub fn get_all_variables(&self) -> &HashMap<String, Value> {
        self.variable_manager.get_all_global_variables()
    }

    /// Специальный метод для установки переменной цикла
    pub fn set_loop_variable(&mut self, name: String, value: Value) {
        self.variable_manager.set_loop_variable(name, value);
    }

    /// Вычислить выражение
    pub fn eval_expr(&mut self, expr: &str) -> Result<Value> {
        // Проверяем, что выражение не пустое (может быть пустым после удаления комментариев лексером)
        let trimmed_expr = expr.trim();
        
        // ВАЖНО: Проверяем на function САМЫМ ПЕРВЫМ делом, ДО всех остальных проверок, включая отладочный вывод
        // Это нужно делать ДО парсинга, чтобы избежать ошибок парсера
        if trimmed_expr.starts_with("function ") || trimmed_expr.starts_with("global function ") || trimmed_expr.starts_with("local function ") {
            return Err(DataCodeError::syntax_error(
                &format!("Function definition '{}' cannot be used as expression. Use 'function name(params) do ... endfunction' as a statement.", trimmed_expr),
                self.current_line, 0
            ));
        }
        
        // Отладка: выводим, что мы получили
        if std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
            eprintln!("🔍 DEBUG eval_expr ENTRY: '{}' (trimmed: '{}')", expr, trimmed_expr);
        }
        
        if trimmed_expr.is_empty() {
            return Ok(Value::Null);
        }

        // Обработка print - должна быть до проверки ключевых слов, так как print может быть в выражениях
        // Но print как оператор обрабатывается в execute_line_simple, здесь обрабатываем только как функцию
        if trimmed_expr.starts_with("print(") {
            // Это вызов функции print в выражении - обрабатываем отдельно
            // Но на самом деле print не должен возвращать значение, поэтому это ошибка
            // Однако для совместимости обработаем это
            return Err(DataCodeError::syntax_error(
                "print() cannot be used as expression. Use 'print(...)' as a statement.",
                self.current_line, 0
            ));
        }

        // ВАЖНО: Проверяем на ключевые слова global/local в начале выражения
        // Это должно быть обработано как оператор присваивания, а не как выражение
        if trimmed_expr.starts_with("global ") || trimmed_expr.starts_with("local ") {
            if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                eprintln!("⚠️  DEBUG eval_expr: Rejecting expression starting with 'global' or 'local': '{}'", trimmed_expr);
            }
            return Err(DataCodeError::syntax_error(
                &format!("Unexpected keyword '{}' in expression context. Assignment statements like 'global x = ...' or 'local x = ...' cannot be used as expressions. Use 'x = ...' for assignment in expression context.", 
                    if trimmed_expr.starts_with("global ") { "global" } else { "local" }),
                self.current_line, 0
            ));
        }

        // Проверяем на блочные конструкции, которые не должны обрабатываться как выражения
        // Это нужно делать ДО парсинга, чтобы избежать ошибок парсера
        if trimmed_expr == "try" || trimmed_expr == "catch" || trimmed_expr == "finally" ||
           trimmed_expr == "endtry" || trimmed_expr == "else" || trimmed_expr == "endif" || trimmed_expr == "endeif" ||
           trimmed_expr == "endfunction" || trimmed_expr.starts_with("next ") ||
           (trimmed_expr.starts_with("for ") && trimmed_expr.ends_with(" do")) ||
           trimmed_expr.starts_with("function ") || trimmed_expr.starts_with("global function ") || trimmed_expr.starts_with("local function ") ||
           trimmed_expr == "print" || (trimmed_expr.starts_with("print ") && !trimmed_expr.starts_with("print(")) {
            return Err(DataCodeError::syntax_error(
                &format!("Unexpected keyword '{}' in expression context. Keywords like 'try', 'catch', 'function', 'for', 'print', etc. cannot be used as expressions.", trimmed_expr),
                self.current_line, 0
            ));
        }

        // КРИТИЧЕСКАЯ ПРОВЕРКА: выражение не должно начинаться с 'local' или 'global'
        // Это должно быть обработано ДО создания парсера
        if trimmed_expr.starts_with("local ") || trimmed_expr.starts_with("global ") {
            if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
                eprintln!("❌ DEBUG eval_expr: CRITICAL ERROR - Expression starts with 'local' or 'global'!");
                eprintln!("   This should have been handled as a statement, not an expression!");
                eprintln!("   Expression: '{}'", trimmed_expr);
            }
            return Err(DataCodeError::syntax_error(
                &format!("Cannot parse '{}' as expression. Statements starting with 'local' or 'global' must be handled as statements, not expressions.", trimmed_expr),
                self.current_line, 0
            ));
        }

        // Парсим выражение (пока без оптимизатора)
        // Отладка: выводим выражение, которое парсится
        if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
            eprintln!("🔍 DEBUG eval_expr: Parsing expression at line {}: '{}'", self.current_line, trimmed_expr);
            // Проверяем, что выражение не содержит многострочный текст
            if trimmed_expr.contains('\n') {
                eprintln!("⚠️  DEBUG eval_expr: WARNING - Expression contains newlines! This might cause parser issues.");
            }
        }
        let mut parser = crate::parser::Parser::new(trimmed_expr);
        let parsed_expr = parser.parse_expression().map_err(|e| {
            eprintln!("❌ DEBUG eval_expr: Parse error for expression '{}' at line {}: {}", trimmed_expr, self.current_line, e);
            e
        })?;
        
        // Проверяем, что после парсинга парсер видит EOF
        if std::env::var("DATACODE_DEBUG").is_ok() || std::env::var("DATACODE_DEBUG_PARSE").is_ok() {
            let final_token = format!("{:?}", parser.current_token());
            if !matches!(parser.current_token(), crate::parser::Token::EOF) {
                eprintln!("⚠️  DEBUG eval_expr: WARNING - Parser did not reach EOF after parsing! Current token: {}", final_token);
            }
        }

        // Вычисляем с поддержкой пользовательских функций
        self.evaluate_expression(&parsed_expr)
    }

    /// Выполнить строку кода (будет реализовано в execution.rs)
    pub fn exec(&mut self, line: &str) -> Result<()> {
        let start_time = Instant::now();
        let result = execution::execute_line(self, line);
        let _duration = start_time.elapsed();

        // Профилирование выполнения (временно отключено для совместимости)
        // TODO: Добавить профилирование после исправления импортов

        result
    }

    /// Выполнить многострочный код
    #[allow(dead_code)]
    pub fn exec_multiline(&mut self, code: &str) -> Result<()> {
        execution::execute_multiline(self, code)
    }

    /// Определить пользовательскую функцию
    #[allow(dead_code)]
    pub fn define_function(&mut self, name: String, parameters: Vec<String>, body: Vec<String>, is_global: bool) -> Result<()> {
        let function = UserFunction::new(name.clone(), parameters, body, is_global);
        self.function_manager.add_function(function);
        Ok(())
    }

    /// Вычислить выражение с поддержкой пользовательских функций
    fn evaluate_expression(&mut self, expr: &crate::parser::Expr) -> Result<Value> {
        use crate::parser::Expr;

        match expr {
            Expr::Literal(value) => Ok(value.clone()),

            Expr::Variable(name) => {
                self.get_variable(name)
                    .cloned()
                    .ok_or_else(|| DataCodeError::variable_not_found(name, self.current_line))
            }

            Expr::FunctionCall { name, args, named_args } => {
                // Вычисляем позиционные аргументы с поддержкой spread оператора
                let mut arg_values = Vec::new();
                for arg in args {
                    match arg {
                        Expr::Spread { expression } => {
                            // Обрабатываем spread оператор
                            let spread_value = self.evaluate_expression(expression)?;
                            self.expand_spread_argument(spread_value, &mut arg_values)?;
                        }
                        _ => {
                            arg_values.push(self.evaluate_expression(arg)?);
                        }
                    }
                }

                // Вычисляем именованные аргументы
                let mut named_arg_values = std::collections::HashMap::new();
                for (arg_name, arg_expr) in named_args {
                    let value = self.evaluate_expression(&arg_expr)?;
                    named_arg_values.insert(arg_name.clone(), value);
                }

                // Проверяем, является ли это пользовательской функцией
                if self.function_manager.contains_function(name) {
                    // Пока пользовательские функции не поддерживают именованные аргументы
                    if !named_arg_values.is_empty() {
                        return Err(DataCodeError::runtime_error(
                            &format!("User functions do not support named arguments yet"),
                            self.current_line
                        ));
                    }
                    
                    // ВАЖНО: Если мы уже внутри Call Frame Engine (есть активный фрейм),
                    // мы НЕ вызываем call_user_function рекурсивно. Вместо этого,
                    // мы возвращаем ошибку, которая будет обработана на уровне выше.
                    // Это временное решение - нужно переписать evaluate_expression чтобы
                    // использовать evaluate_expression_signal
                    if self.use_call_frame_engine && !self.call_stack.is_empty() {
                        // Мы внутри Call Frame Engine - возвращаем ошибку для обработки через сигналы
                        // Эта ошибка будет обработана в execute_instruction_signal, который использует
                        // evaluate_expression_signal для правильной обработки вызовов функций
                        return Err(DataCodeError::runtime_error(
                            &format!("USER_FUNCTION_CALL_IN_EXPRESSION:{}:{}", name, arg_values.len()),
                            self.current_line
                        ));
                    } else {
                        // Первый вызов функции (стек пуст) - используем обычный механизм
                        self.call_user_function(name, arg_values)
                    }
                } else {
                    // Встроенная функция
                    call_builtin_function_with_named_args(name, arg_values, named_arg_values, self.current_line)
                }
            }

            _ => {
                // Для остальных типов выражений используем ExpressionEvaluator из interpreter модуля
                // который имеет доступ к function_manager для поддержки рекурсивных вызовов
                let evaluator = crate::interpreter::expressions::ExpressionEvaluator::new(
                    &self.variable_manager,
                    &self.function_manager,
                    self.current_line,
                );

                match evaluator.evaluate(expr) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        // Обрабатываем вызов пользовательской функции из выражения
                        // Ищем вызов пользовательской функции в выражении рекурсивно
                        self.handle_user_function_in_expression(expr)
                    }
                    result => result
                }
            }
        }
    }

    /// Выполнить один фрейм функции (для trampoline pattern)
    /// Возвращает FunctionResult::Done если функция завершена, или FunctionResult::Continue
    /// если требуется вызвать другую функцию (рекурсия или вложенный вызов)
    /// 
    /// ВАЖНО: Этот метод НЕ должен вызывать call_user_function рекурсивно.
    /// Вместо этого, если во время выполнения встречается вызов функции,
    /// он должен быть добавлен в trampoline_queue, и метод должен вернуть FunctionResult::Continue
    #[allow(dead_code)]
    fn execute_function_frame(&mut self, name: &str, args: Vec<Value>) -> Result<FunctionResult> {
        let function = self.function_manager.get_function(name)
            .ok_or_else(|| DataCodeError::function_not_found(name, self.current_line))?
            .clone();

        // Проверяем количество аргументов
        if function.parameters.len() != args.len() {
            return Err(DataCodeError::wrong_argument_count(
                name,
                function.parameters.len(),
                args.len(),
                self.current_line,
            ));
        }

        // Проверяем лимит рекурсии ПЕРЕД входом в область видимости
        // Используем тот же лимит, что и в enter_function_scope для консистентности
        const MAX_RECURSION_DEPTH: usize = 1000;
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            return Err(DataCodeError::runtime_error(
                &format!("Превышена максимальная глубина рекурсии ({}) в функции '{}'", MAX_RECURSION_DEPTH, name),
                self.current_line
            ));
        }

        // Входим в область видимости функции
        self.recursion_depth += 1;
        self.variable_manager.enter_function_scope();

        // Устанавливаем параметры функции
        if let Err(e) = self.variable_manager
            .set_function_parameters(&function.parameters, args) {
            self.variable_manager.exit_function_scope();
            self.recursion_depth -= 1;
            return Err(DataCodeError::runtime_error(&e, self.current_line));
        }

        // Сохраняем текущее возвращаемое значение
        let old_return_value = self.return_value.take();

        // Выполняем тело функции
        // Если во время выполнения встречается вызов функции, он будет добавлен в trampoline_queue
        // и мы вернем FunctionResult::Continue для обработки через trampoline loop
        let execution_result = execution::execute_block_directly(
            self,
            &function.body.iter().map(|s| s.as_str()).collect::<Vec<_>>()
        );
        
        // Проверяем, были ли добавлены вызовы функций в очередь
        let pending_calls = self.trampoline_queue.clone();

        // Получаем результат выполнения функции
        let final_result = if let Some(return_val) = self.return_value.take() {
            return_val
        } else {
            Value::Null
        };

        // Восстанавливаем предыдущее возвращаемое значение
        self.return_value = old_return_value;

        // Выходим из области видимости функции
        self.variable_manager.exit_function_scope();
        self.recursion_depth -= 1;

        // Проверяем результат выполнения
        execution_result?;

        // Если есть ожидающие вызовы функций, возвращаем Continue
        // Это означает, что функция приостановила выполнение для вызова другой функции
        // ВАЖНО: Но сначала нужно обработать вызовы и получить результаты
        // Проблема: мы не можем продолжить выполнение функции, пока не получим результаты вызовов
        // Поэтому мы возвращаем Continue, и trampoline loop обработает вызовы
        if !pending_calls.is_empty() {
            // Берем первый вызов из очереди
            let (next_func, next_args) = pending_calls[0].clone();
            // Остальные вызовы оставляем в очереди для обработки
            // (они уже в trampoline_queue, не нужно их добавлять обратно)
            return Ok(FunctionResult::Continue {
                function_name: next_func,
                args: next_args,
            });
        }
        
        // Если во время выполнения были вызовы функций, результаты должны быть в trampoline_result_stack
        // Но проблема: мы не знаем, сколько результатов там должно быть, и как их использовать
        // Это требует более сложной архитектуры с continuation passing style

        Ok(FunctionResult::Done(final_result))
    }

    /// Вызвать пользовательскую функцию с использованием Call Frame Engine
    /// ЕДИНЫЙ СОБЫТИЙНЫЙ ЦИКЛ: обрабатывает все вызовы функций без рекурсии Rust
    /// Использует ExecSignal для координации между evaluate_expression и execute_instruction
    pub fn call_user_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Проверяем кэш перед вызовом функции
        if let Some(cached_result) = self.function_cache.get(name, &args) {
            if std::env::var("DATACODE_DEBUG").is_ok() {
                eprintln!("🔍 DEBUG call_user_function: Cache HIT for {}({:?})", name, args);
            }
            return Ok(cached_result);
        }
        
        if std::env::var("DATACODE_DEBUG").is_ok() {
            eprintln!("🔍 DEBUG call_user_function: Cache MISS for {}({:?})", name, args);
        }
        
        // Отмечаем, что функция начала выполняться (для предотвращения рекурсивных проблем с кэшем)
        self.function_cache.mark_in_progress(name, &args);
        
        if !self.use_call_frame_engine {
            // Если Call Frame Engine отключен, используем старую реализацию
            let result = self.call_user_function_direct(name, args.clone())?;
            // Отмечаем функцию как завершенную перед сохранением в кэш
            self.function_cache.mark_completed(name, &args);
            // Сохраняем результат в кэш
            self.function_cache.put(name, &args, result.clone());
            return Ok(result);
        }

        // Получаем функцию
        let function = self.function_manager.get_function(name)
            .ok_or_else(|| DataCodeError::function_not_found(name, self.current_line))?
            .clone();

        // Проверяем количество аргументов
        if function.parameters.len() != args.len() {
            return Err(DataCodeError::wrong_argument_count(
                name,
                function.parameters.len(),
                args.len(),
                self.current_line,
            ));
        }

        // Сохраняем аргументы для кэша (нужны для сохранения результата)
        let args_for_cache = args.clone();
        
        // Создаем начальный фрейм
        let initial_frame = CallFrame::new(
            name.to_string(),
            args,
            None, // return_slot будет установлен при необходимости
            self.call_stack.len(),
        );

        // Добавляем фрейм в стек
        self.call_stack.push(initial_frame)?;

        // Инициализируем локальные переменные функции
        self.variable_manager.enter_function_scope();
        if let Some(frame) = self.call_stack.last_mut() {
            let args = frame.args.clone();
            for (param, arg_value) in function.parameters.iter().zip(args.iter()) {
                frame.set_local(param.clone(), arg_value.clone());
                if let Some(local_vars) = self.variable_manager.call_stack.last_mut() {
                    local_vars.insert(param.clone(), arg_value.clone());
                }
            }
        }

        // ЕДИНЫЙ СОБЫТИЙНЫЙ ЦИКЛ: обрабатываем все сигналы без рекурсии
        loop {
            // Получаем текущий фрейм и его тело
            let (current_function_id, current_function_body, current_ip) = {
                let frame = self.call_stack.last()
                    .ok_or_else(|| DataCodeError::runtime_error(
                        "Internal error: no frame in stack",
                        self.current_line
                    ))?;
                
                let func_id = frame.function_id.clone();
                let func = self.function_manager.get_function(&func_id)
                    .ok_or_else(|| DataCodeError::function_not_found(&func_id, self.current_line))?;
                (func_id, func.body.clone(), frame.ip)
            };

            // Проверяем, завершена ли функция
            if current_ip >= current_function_body.len() {
                // Функция завершена без явного return - возвращаем Null
                let return_slot = {
                    if let Some(frame) = self.call_stack.last() {
                        frame.return_slot.clone()
                    } else {
                        None
                    }
                };
                
                let return_value = Value::Null;
                // Сохраняем информацию о функции перед удалением фрейма
                let (returning_function_id, returning_args) = {
                    if let Some(frame) = self.call_stack.last() {
                        (frame.function_id.clone(), frame.args.clone())
                    } else {
                        (current_function_id.clone(), Vec::new())
                    }
                };
                
                self.call_stack.pop();
                self.variable_manager.exit_function_scope();
                
                if self.call_stack.is_empty() {
                    // Отмечаем функцию как завершенную перед сохранением в кэш
                    self.function_cache.mark_completed(&returning_function_id, &returning_args);
                    // Сохраняем результат в кэш перед возвратом
                    self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                    return Ok(return_value);
                }
                
                // Отмечаем функцию как завершенную перед сохранением в кэш
                self.function_cache.mark_completed(&returning_function_id, &returning_args);
                // Сохраняем результат в кэш для вложенного вызова
                self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                
                if let Some(return_slot) = return_slot {
                    if let Some(caller_frame) = self.call_stack.last_mut() {
                        caller_frame.set_local(return_slot, return_value.clone());
                    }
                }
                
                continue;
            }

            // Синхронизируем локальные переменные фрейма с variable_manager
            if let Some(frame) = self.call_stack.last() {
                if let Some(local_vars) = self.variable_manager.call_stack.last_mut() {
                    local_vars.clear();
                    for (name, value) in &frame.locals {
                        local_vars.insert(name.clone(), value.clone());
                    }
                }
            }

            // Получаем текущую инструкцию
            let instruction = &current_function_body[current_ip];
            let trimmed_instruction = instruction.trim();
            
            if std::env::var("DATACODE_DEBUG").is_ok() {
                eprintln!("🔍 DEBUG call_user_function: Executing instruction {}: '{}'", current_ip, instruction);
            }
            
            // Проверяем, является ли это блочной конструкцией (for, if, try)
            // Если да, обрабатываем через execute_block_directly
            if trimmed_instruction.starts_with("for ") && trimmed_instruction.ends_with(" do") {
                // Это цикл for - обрабатываем через execute_block_directly
                // Собираем строки от текущей позиции до конца функции
                let remaining_lines: Vec<&str> = current_function_body[current_ip..].iter().map(|s| s.as_str()).collect();
                use crate::interpreter::execution::execute_block_directly;
                
                // Выполняем блок, который обработает цикл for и все последующие инструкции
                execute_block_directly(self, &remaining_lines)?;
                
                // Проверяем return
                if self.return_value.is_some() {
                    // Return был выполнен - завершаем функцию
                    let return_value = self.return_value.take().unwrap();
                    let return_slot = {
                        if let Some(frame) = self.call_stack.last() {
                            frame.return_slot.clone()
                        } else {
                            None
                        }
                    };
                    
                    // Сохраняем информацию о функции перед удалением фрейма
                    let (returning_function_id, returning_args) = {
                        if let Some(frame) = self.call_stack.last() {
                            (frame.function_id.clone(), frame.args.clone())
                        } else {
                            (current_function_id.clone(), Vec::new())
                        }
                    };
                    
                    self.call_stack.pop();
                    self.variable_manager.exit_function_scope();
                    
                    if self.call_stack.is_empty() {
                        // Отмечаем функцию как завершенную перед сохранением в кэш
                        self.function_cache.mark_completed(&returning_function_id, &returning_args);
                        // Сохраняем результат в кэш перед возвратом
                        self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                        return Ok(return_value);
                    }
                    
                    // Отмечаем функцию как завершенную перед сохранением в кэш
                    self.function_cache.mark_completed(&returning_function_id, &returning_args);
                    // Сохраняем результат в кэш для вложенного вызова
                    self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                    
                    if let Some(return_slot) = return_slot {
                        if let Some(caller_frame) = self.call_stack.last_mut() {
                            caller_frame.set_local(return_slot, return_value.clone());
                        }
                    }
                    
                    continue;
                }
                
                // Блочная конструкция выполнена - переходим к концу функции
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.ip = current_function_body.len();
                }
                continue;
            } else if (trimmed_instruction.starts_with("if ") && (trimmed_instruction.contains(" do") || trimmed_instruction.contains(" then"))) ||
                      trimmed_instruction == "try" {
                // Это if или try - обрабатываем через execute_block_directly
                // Вычисляем количество строк в блоке if/try для правильного увеличения IP
                let mut block_size = 1; // Начальная строка (if/try)
                if trimmed_instruction.starts_with("if ") {
                    let mut if_depth = 1;
                    let mut j = current_ip + 1;
                    while j < current_function_body.len() && if_depth > 0 {
                        let line = current_function_body[j].trim();
                        if line.starts_with("if ") && (line.contains(" do") || line.contains(" then")) {
                            if_depth += 1;
                        } else if line == "endif" || line == "endeif" {
                            if_depth -= 1;
                        }
                        block_size += 1;
                        if if_depth == 0 {
                            break;
                        }
                        j += 1;
                    }
                } else if trimmed_instruction == "try" {
                    let mut try_depth = 1;
                    let mut j = current_ip + 1;
                    while j < current_function_body.len() && try_depth > 0 {
                        let line = current_function_body[j].trim();
                        if line == "try" {
                            try_depth += 1;
                        } else if line == "endtry" {
                            try_depth -= 1;
                        }
                        block_size += 1;
                        if try_depth == 0 {
                            break;
                        }
                        j += 1;
                    }
                }
                
                let remaining_lines: Vec<&str> = current_function_body[current_ip..].iter().map(|s| s.as_str()).collect();
                use crate::interpreter::execution::execute_block_directly;
                execute_block_directly(self, &remaining_lines)?;
                
                // Проверяем return
                if self.return_value.is_some() {
                    let return_value = self.return_value.take().unwrap();
                    let return_slot = {
                        if let Some(frame) = self.call_stack.last() {
                            frame.return_slot.clone()
                        } else {
                            None
                        }
                    };
                    
                    // Сохраняем информацию о функции перед удалением фрейма
                    let (returning_function_id, returning_args) = {
                        if let Some(frame) = self.call_stack.last() {
                            (frame.function_id.clone(), frame.args.clone())
                        } else {
                            (current_function_id.clone(), Vec::new())
                        }
                    };
                    
                    self.call_stack.pop();
                    self.variable_manager.exit_function_scope();
                    
                    if self.call_stack.is_empty() {
                        // Отмечаем функцию как завершенную перед сохранением в кэш
                        self.function_cache.mark_completed(&returning_function_id, &returning_args);
                        // Сохраняем результат в кэш перед возвратом
                        self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                        return Ok(return_value);
                    }
                    
                    // Отмечаем функцию как завершенную перед сохранением в кэш
                    self.function_cache.mark_completed(&returning_function_id, &returning_args);
                    // Сохраняем результат в кэш для вложенного вызова
                    self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                    
                    if let Some(return_slot) = return_slot {
                        if let Some(caller_frame) = self.call_stack.last_mut() {
                            caller_frame.set_local(return_slot, return_value.clone());
                        }
                    }
                    
                    continue;
                }
                
                // Блочная конструкция выполнена - увеличиваем IP на количество обработанных строк
                if let Some(frame) = self.call_stack.last_mut() {
                    if frame.function_id == current_function_id && frame.ip == current_ip {
                        frame.ip = current_ip + block_size;
                    }
                }
                continue;
            }
            
            // Выполняем инструкцию и получаем сигнал
            let mut signal = self.execute_instruction_signal(instruction)?;
            
            // Если сигнал - это ExecSignal::Call с результатом в кэше, заменяем на ExecSignal::Value
            // Это нужно для продолжения вычисления бинарных операций
            if let ExecSignal::Call { function_id, args, return_slot } = &signal {
                if let Some(cached_result) = self.function_cache.get(function_id, args) {
                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG call_user_function: Cache HIT in instruction for {}({:?}), converting to Value", function_id, args);
                    }
                    // Заменяем ExecSignal::Call на ExecSignal::Value с результатом из кэша
                    signal = ExecSignal::Value(cached_result);
                }
            }
            
            // Обрабатываем сигнал
            match signal {
                ExecSignal::Value(_) => {
                    // Инструкция выполнена, переходим к следующей
                    if let Some(frame) = self.call_stack.last_mut() {
                        if frame.function_id == current_function_id && frame.ip == current_ip {
                            frame.advance();
                        }
                    }
                    
                    // Синхронизируем локальные переменные обратно в фрейм
                    if let Some(frame) = self.call_stack.last_mut() {
                        if let Some(local_vars) = self.variable_manager.call_stack.last() {
                            for (name, value) in local_vars {
                                frame.set_local(name.clone(), value.clone());
                            }
                        }
                    }
                }
                
                ExecSignal::Call { function_id, args, return_slot } => {
                    // Проверяем кэш перед созданием нового фрейма
                    if let Some(cached_result) = self.function_cache.get(&function_id, &args) {
                        if std::env::var("DATACODE_DEBUG").is_ok() {
                            eprintln!("🔍 DEBUG call_user_function: Cache HIT in ExecSignal::Call for {}({:?})", function_id, args);
                        }
                        
                        // Сохраняем результат в return_slot (если есть)
                        if let Some(return_slot) = &return_slot {
                            if let Some(caller_frame) = self.call_stack.last_mut() {
                                if return_slot.starts_with("__assign_") {
                                    // Формат: __assign_{var_name}_{depth}
                                    if let Some(underscore_pos) = return_slot[9..].find('_') {
                                        let var_name = &return_slot[9..9+underscore_pos];
                                        caller_frame.set_local(var_name.to_string(), cached_result.clone());
                                        self.set_variable_smart(var_name.to_string(), cached_result.clone());
                                    } else {
                                        caller_frame.set_local(return_slot.clone(), cached_result.clone());
                                    }
                                } else if return_slot.starts_with("__binary_") {
                                    // Это бинарная операция - нужно продолжить вычисление
                                    // Пока просто сохраняем в return_slot
                                    caller_frame.set_local(return_slot.clone(), cached_result.clone());
                                } else {
                                    caller_frame.set_local(return_slot.clone(), cached_result.clone());
                                }
                            }
                        }
                        
                        // Увеличиваем IP текущего фрейма
                        if let Some(frame) = self.call_stack.last_mut() {
                            if frame.function_id == current_function_id && frame.ip == current_ip {
                                frame.advance();
                            }
                        }
                        
                        // Продолжаем выполнение - результат уже в return_slot
                        continue;
                    }
                    
                    if std::env::var("DATACODE_DEBUG").is_ok() {
                        eprintln!("🔍 DEBUG call_user_function: Cache MISS in ExecSignal::Call for {}({:?})", function_id, args);
                    }
                    
                    // Вызов функции - создаем новый фрейм
                    // ВАЖНО: НЕ увеличиваем IP здесь, увеличим после возврата из функции
                    let called_function = self.function_manager.get_function(&function_id)
                        .ok_or_else(|| DataCodeError::function_not_found(&function_id, self.current_line))?;
                    
                    if called_function.parameters.len() != args.len() {
                        return Err(DataCodeError::wrong_argument_count(
                            &function_id,
                            called_function.parameters.len(),
                            args.len(),
                            self.current_line,
                        ));
                    }
                    
                    // Сохраняем аргументы для кэша
                    let args_for_cache = args.clone();
                    
                    let new_frame = CallFrame::new(
                        function_id.clone(),
                        args,
                        return_slot,
                        self.call_stack.len(),
                    );
                    
                    self.call_stack.push(new_frame)?;
                    self.variable_manager.enter_function_scope();
                    
                    if let Some(frame) = self.call_stack.last_mut() {
                        let args = frame.args.clone();
                        for (param, arg_value) in called_function.parameters.iter().zip(args.iter()) {
                            frame.set_local(param.clone(), arg_value.clone());
                            if let Some(local_vars) = self.variable_manager.call_stack.last_mut() {
                                local_vars.insert(param.clone(), arg_value.clone());
                            }
                        }
                    }
                    
                    // Продолжаем выполнение нового фрейма
                    continue;
                }
                
                ExecSignal::Return(return_value) => {
                    // Return - завершаем текущую функцию
                    // Сохраняем информацию о функции перед удалением фрейма
                    let (returning_function_id, returning_args) = {
                        if let Some(frame) = self.call_stack.last() {
                            (frame.function_id.clone(), frame.args.clone())
                        } else {
                            (current_function_id.clone(), Vec::new())
                        }
                    };
                    
                    let return_slot = {
                        if let Some(frame) = self.call_stack.last() {
                            frame.return_slot.clone()
                        } else {
                            None
                        }
                    };
                    
                    self.call_stack.pop();
                    self.variable_manager.exit_function_scope();
                    
                    if self.call_stack.is_empty() {
                        // Отмечаем функцию как завершенную перед сохранением в кэш
                        self.function_cache.mark_completed(&returning_function_id, &returning_args);
                        // Сохраняем результат в кэш перед возвратом
                        // Используем аргументы возвращающей функции
                        self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                        return Ok(return_value);
                    }
                    
                    // Отмечаем функцию как завершенную перед сохранением в кэш
                    self.function_cache.mark_completed(&returning_function_id, &returning_args);
                    // Сохраняем результат в кэш для вложенного вызова
                    self.function_cache.put(&returning_function_id, &returning_args, return_value.clone());
                    
                    // Записываем возвращаемое значение в return_slot (если есть)
                    let should_advance_ip = if let Some(return_slot) = &return_slot {
                        if let Some(caller_frame) = self.call_stack.last_mut() {
                            // Если return_slot начинается с __assign_, это присваивание
                            // Извлекаем имя переменной и записываем значение в нее
                            if return_slot.starts_with("__assign_") {
                                // Формат: __assign_{var_name}_{depth}
                                if let Some(underscore_pos) = return_slot[9..].find('_') {
                                    let var_name = &return_slot[9..9+underscore_pos];
                                    // Записываем значение в переменную
                                    caller_frame.set_local(var_name.to_string(), return_value.clone());
                                    
                                    // Увеличиваем IP, так как присваивание завершено
                                    let should_advance = caller_frame.function_id == current_function_id && caller_frame.ip == current_ip;
                                    if should_advance {
                                        caller_frame.advance();
                                    }
                                    
                                    // Также синхронизируем с variable_manager (после освобождения заимствования)
                                    // Определяем, является ли это global присваиванием
                                    // Для этого нужно проверить текущую инструкцию
                                    // Но так как мы уже увеличили IP (или собираемся), нужно использовать другой подход
                                    // Пока просто используем умное определение области видимости
                                    self.set_variable_smart(var_name.to_string(), return_value.clone());
                                    
                                    // IP уже увеличен, не нужно увеличивать еще раз
                                    false
                                } else {
                                    // Обычный return_slot - просто записываем значение
                                    caller_frame.set_local(return_slot.clone(), return_value.clone());
                                    true
                                }
                            } else {
                                // Обычный return_slot - просто записываем значение
                                caller_frame.set_local(return_slot.clone(), return_value.clone());
                                true
                            }
                        } else {
                            true
                        }
                    } else {
                        true
                    };
                    
                    // Увеличиваем IP текущего фрейма, так как вызов функции завершен
                    // (если IP еще не был увеличен выше для присваивания)
                    if should_advance_ip {
                        if let Some(frame) = self.call_stack.last_mut() {
                            if frame.function_id == current_function_id && frame.ip == current_ip {
                                frame.advance();
                            }
                        }
                    }
                    
                    // Продолжаем выполнение вызывающей функции
                    continue;
                }
            }
        }
    }

    /// Выполнить одну инструкцию в контексте фрейма
    /// Возвращает ExecResult, который указывает, что делать дальше
    /// ВАЖНО: Этот метод временно использует стандартный механизм выполнения
    /// TODO: Реализовать полную поддержку локальных переменных фрейма
    #[allow(dead_code)]
    fn execute_instruction_in_frame(&mut self, instruction: &str, _frame: &mut CallFrame) -> Result<ExecResult> {
        let trimmed = instruction.trim();

        // Пропускаем пустые строки и комментарии
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(ExecResult::Continue);
        }

        // Обработка return
        if trimmed.starts_with("return") {
            let after_return = trimmed.strip_prefix("return").unwrap().trim();
            let value = if after_return.is_empty() {
                Value::Null
            } else {
                // Временно используем стандартный eval_expr
                // TODO: Использовать локальные переменные фрейма
                self.eval_expr(after_return)?
            };
            return Ok(ExecResult::Return(value));
        }

        // Обработка присваивания переменных
        if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") 
            && !trimmed.contains("<=") && !trimmed.contains(">=") {
            let parts: Vec<_> = trimmed.splitn(2, '=').map(|s| s.trim()).collect();
            
            if parts.len() == 2 {
                let var_name = parts[0];
                
                // Проверяем, что левая часть - это простой идентификатор
                if var_name.chars().all(|c| c.is_alphanumeric() || c == '_') && !var_name.is_empty() {
                    let val = self.eval_expr(parts[1])?;
                    
                    // Сохраняем переменную в текущем фрейме
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.set_local(var_name.to_string(), val);
                    }
                    return Ok(ExecResult::Continue);
                }
            }
        }

        // Обработка global/local переменных
        if trimmed.starts_with("global ") || trimmed.starts_with("local ") {
            let is_global = trimmed.starts_with("global ");
            let code = &trimmed[if is_global { 7 } else { 6 }..];
            let parts: Vec<_> = code.splitn(2, '=').map(|s| s.trim()).collect();

            if parts.len() == 2 {
                let var_name = parts[0].to_string();
                let val = self.eval_expr(parts[1])?;
                
                // Если global, сохраняем в глобальных переменных
                if is_global {
                    self.set_variable(var_name.clone(), val.clone(), true);
                }
                // Сохраняем в локальных переменных фрейма
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.set_local(var_name, val);
                }
                return Ok(ExecResult::Continue);
            }
        }

        // Все остальное - выражения (которые могут содержать вызовы функций)
        // Вычисляем выражение, но не сохраняем результат
        self.eval_expr(trimmed)?;
        Ok(ExecResult::Continue)
    }

    /// Вычислить выражение в контексте фрейма
    /// Использует локальные переменные фрейма вместо глобальных
    #[allow(dead_code)]
    fn eval_expr_in_frame(&mut self, expr: &str, frame: &CallFrame) -> Result<Value> {
        let trimmed_expr = expr.trim();
        if trimmed_expr.is_empty() {
            return Ok(Value::Null);
        }

        // Парсим выражение
        let mut parser = crate::parser::Parser::new(trimmed_expr);
        let parsed_expr = parser.parse_expression().map_err(|e| {
            DataCodeError::syntax_error(
                &format!("Parse error: {}", e),
                self.current_line,
                0
            )
        })?;

        // Вычисляем выражение с учетом локальных переменных фрейма
        self.evaluate_expression_in_frame(&parsed_expr, frame)
    }

    /// Вычислить выражение AST в контексте фрейма
    #[allow(dead_code)]
    fn evaluate_expression_in_frame(&mut self, expr: &crate::parser::Expr, frame: &CallFrame) -> Result<Value> {
        use crate::parser::Expr;

        match expr {
            Expr::Literal(value) => Ok(value.clone()),

            Expr::Variable(name) => {
                // Сначала проверяем локальные переменные фрейма
                if let Some(value) = frame.get_local(name) {
                    return Ok(value.clone());
                }
                // Затем проверяем глобальные переменные
                self.get_variable(name)
                    .cloned()
                    .ok_or_else(|| DataCodeError::variable_not_found(name, self.current_line))
            }

            Expr::FunctionCall { name, args, named_args } => {
                // Вычисляем позиционные аргументы
                let mut arg_values = Vec::new();
                for arg in args {
                    match arg {
                        Expr::Spread { expression } => {
                            let spread_value = self.evaluate_expression_in_frame(expression, frame)?;
                            self.expand_spread_argument(spread_value, &mut arg_values)?;
                        }
                        _ => {
                            arg_values.push(self.evaluate_expression_in_frame(arg, frame)?);
                        }
                    }
                }

                // Вычисляем именованные аргументы
                let mut named_arg_values = std::collections::HashMap::new();
                for (arg_name, arg_expr) in named_args {
                    let value = self.evaluate_expression_in_frame(&arg_expr, frame)?;
                    named_arg_values.insert(arg_name.clone(), value);
                }

                // Проверяем, является ли это пользовательской функцией
                if self.function_manager.contains_function(name) {
                    if !named_arg_values.is_empty() {
                        return Err(DataCodeError::runtime_error(
                            "User functions do not support named arguments yet",
                            self.current_line
                        ));
                    }
                    
                    // Вызываем функцию через Call Frame Engine
                    // Создаем временный слот для результата
                    let temp_slot = format!("__temp_result_{}", self.call_stack.len());
                    let new_frame = CallFrame::new(
                        name.clone(),
                        arg_values.clone(),
                        Some(temp_slot.clone()),
                        frame.depth + 1,
                    );
                    
                    // Добавляем фрейм в стек
                    self.call_stack.push(new_frame)?;
                    
                    // Инициализируем локальные переменные новой функции
                    if let Some(new_frame) = self.call_stack.last_mut() {
                        let called_function = self.function_manager.get_function(name)
                            .ok_or_else(|| DataCodeError::function_not_found(name, self.current_line))?;
                        
                        let args = new_frame.args.clone();
                        for (param, arg_value) in called_function.parameters.iter().zip(args.iter()) {
                            new_frame.set_local(param.clone(), arg_value.clone());
                        }
                    }
                    
                    // Выполняем функцию через call_user_function
                    // ВАЖНО: call_user_function теперь использует единый цикл без рекурсии Rust
                    let result = self.call_user_function(name, arg_values)?;
                    
                    // Возвращаем результат
                    Ok(result)
                } else {
                    // Встроенная функция
                    call_builtin_function_with_named_args(name, arg_values, named_arg_values, self.current_line)
                }
            }

            _ => {
                // Для остальных типов выражений используем ExpressionEvaluator
                // но с учетом локальных переменных фрейма
                // Временно используем стандартный evaluator
                let evaluator = crate::interpreter::expressions::ExpressionEvaluator::new(
                    &self.variable_manager,
                    &self.function_manager,
                    self.current_line,
                );

                match evaluator.evaluate(expr) {
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        // Обрабатываем вызов пользовательской функции из выражения
                        self.handle_user_function_in_expression(expr)
                    }
                    result => result
                }
            }
        }
    }

    /// УДАЛЕНО: execute_function_until_return больше не используется
    /// Вместо этого используется единый цикл в call_user_function
    #[allow(dead_code)]
    fn _execute_function_until_return_removed(&mut self) -> Result<Value> {
        // Получаем функцию из текущего фрейма
        let (function_id, function_body) = {
            let frame = self.call_stack.last()
                .ok_or_else(|| DataCodeError::runtime_error(
                    "Internal error: no frame in stack",
                    self.current_line
                ))?;
            let func_id = frame.function_id.clone();
            let func = self.function_manager.get_function(&func_id)
                .ok_or_else(|| DataCodeError::function_not_found(&func_id, self.current_line))?;
            (func_id, func.body.clone())
        };

        // Выполняем функцию до завершения
        loop {
            // Получаем текущую инструкцию и IP без mutable borrow
            let (instruction, current_ip, current_depth) = {
                let frame = self.call_stack.last()
                    .ok_or_else(|| DataCodeError::runtime_error(
                        "Internal error: no frame in stack",
                        self.current_line
                    ))?;
                
                if frame.function_id != function_id {
                    // Фрейм изменился - функция завершена
                    break;
                }
                
                let ip = frame.ip;
                let depth = frame.depth;
                
                match frame.current_instruction(&function_body) {
                    Some(inst) => (inst.clone(), ip, depth),
                    None => {
                        // Нет инструкций - функция завершена
                        // Получаем return_slot перед удалением фрейма
                        let return_slot = {
                            if let Some(frame) = self.call_stack.last() {
                                frame.return_slot.clone()
                            } else {
                                None
                            }
                        };
                        
                        let return_value = self.return_value.take().unwrap_or(Value::Null);
                        self.call_stack.pop();
                        self.variable_manager.exit_function_scope();
                        
                        // Если есть return_slot, сохраняем результат в вызывающем фрейме
                        if let Some(return_slot) = &return_slot {
                            if let Some(caller_frame) = self.call_stack.last_mut() {
                                caller_frame.set_local(return_slot.clone(), return_value.clone());
                            }
                            // Возвращаем результат из return_slot вызывающего фрейма
                            if let Some(caller_frame) = self.call_stack.last() {
                                if let Some(result) = caller_frame.get_local(return_slot) {
                                    return Ok(result.clone());
                                }
                            }
                        }
                        
                        return Ok(return_value);
                    }
                }
            };

            // Проверяем, завершена ли функция
            if current_ip >= function_body.len() {
                // Получаем return_slot перед удалением фрейма
                let return_slot = {
                    if let Some(frame) = self.call_stack.last() {
                        frame.return_slot.clone()
                    } else {
                        None
                    }
                };
                
                let return_value = self.return_value.take().unwrap_or(Value::Null);
                self.call_stack.pop();
                self.variable_manager.exit_function_scope();
                
                // Если есть return_slot, сохраняем результат в вызывающем фрейме
                if let Some(return_slot) = &return_slot {
                    if let Some(caller_frame) = self.call_stack.last_mut() {
                        caller_frame.set_local(return_slot.clone(), return_value.clone());
                    }
                    // Возвращаем результат из return_slot вызывающего фрейма
                    if let Some(caller_frame) = self.call_stack.last() {
                        if let Some(result) = caller_frame.get_local(return_slot) {
                            return Ok(result.clone());
                        }
                    }
                }
                
                return Ok(return_value);
            }

            // Синхронизируем локальные переменные фрейма с variable_manager перед выполнением
            if let Some(frame) = self.call_stack.last() {
                // Обновляем variable_manager с локальными переменными фрейма
                if let Some(local_vars) = self.variable_manager.call_stack.last_mut() {
                    local_vars.clear();
                    for (name, value) in &frame.locals {
                        local_vars.insert(name.clone(), value.clone());
                    }
                }
            }
            
            // Выполняем инструкцию
            // ВАЖНО: Не используем self.exec(trimmed), так как он может вызвать evaluate_expression,
            // который может добавить новый фрейм в стек и вызвать execute_function_until_return снова,
            // создавая рекурсию на Rust стеке. Вместо этого обрабатываем инструкции напрямую.
            let trimmed = instruction.trim();
            let exec_result = if trimmed.is_empty() || trimmed.starts_with('#') {
                ExecResult::Continue
            } else if trimmed.starts_with("return") {
                let after_return = trimmed.strip_prefix("return").unwrap().trim();
                let value = if after_return.is_empty() {
                    Value::Null
                } else {
                    // Вычисляем значение return через eval_expr
                    // ВАЖНО: eval_expr может вызвать evaluate_expression, который может добавить
                    // новый фрейм в стек для вложенных вызовов функций. Но evaluate_expression
                    // проверяет, есть ли активный фрейм в call_stack, и если есть, то добавляет
                    // новый фрейм и выполняет его до завершения через execute_function_until_return.
                    // Это создает рекурсию! Нужно использовать eval_expr_in_frame вместо eval_expr.
                    // Но eval_expr_in_frame не существует. Временно используем eval_expr, но с пониманием,
                    // что это может вызвать рекурсию для глубоких вызовов.
                    // TODO: Реализовать eval_expr_in_frame, который не вызывает evaluate_expression рекурсивно
                    self.eval_expr(after_return)?
                };
                ExecResult::Return(value)
            } else if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") 
                && !trimmed.contains("<=") && !trimmed.contains(">=") {
                // Обработка присваивания переменных
                let parts: Vec<_> = trimmed.splitn(2, '=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let var_name = parts[0];
                    if var_name.chars().all(|c| c.is_alphanumeric() || c == '_') && !var_name.is_empty() {
                        let val = self.eval_expr(parts[1])?;
                        // Сохраняем переменную в текущем фрейме
                        if let Some(frame) = self.call_stack.last_mut() {
                            frame.set_local(var_name.to_string(), val);
                        }
                        // Также синхронизируем с variable_manager
                        if let Some(local_vars) = self.variable_manager.call_stack.last_mut() {
                            if let Some(frame) = self.call_stack.last() {
                                for (name, value) in &frame.locals {
                                    local_vars.insert(name.clone(), value.clone());
                                }
                            }
                        }
                        ExecResult::Continue
                    } else {
                        // Сложное присваивание - используем exec
                        self.exec(trimmed)?;
                        ExecResult::Continue
                    }
                } else {
                    ExecResult::Continue
                }
            } else {
                // Для остальных инструкций используем exec
                // ВАЖНО: Это может вызвать рекурсию для глубоких вызовов
                // TODO: Реализовать прямую обработку всех типов инструкций
                self.exec(trimmed)?;
                ExecResult::Continue
            };
            
            // После выполнения синхронизируем обратно локальные переменные из variable_manager в фрейм
            if let Some(frame) = self.call_stack.last_mut() {
                if let Some(local_vars) = self.variable_manager.call_stack.last() {
                    for (name, value) in local_vars {
                        frame.set_local(name.clone(), value.clone());
                    }
                }
            }

            match exec_result {
                ExecResult::Continue => {
                    // Переходим к следующей инструкции
                    if let Some(frame) = self.call_stack.last_mut() {
                        if frame.function_id == function_id && frame.ip == current_ip {
                            frame.advance();
                        }
                    }
                }
                ExecResult::Return(value) => {
                    // Получаем return_slot перед удалением фрейма
                    let return_slot = {
                        if let Some(frame) = self.call_stack.last() {
                            frame.return_slot.clone()
                        } else {
                            None
                        }
                    };
                    
                    // Удаляем фрейм
                    self.call_stack.pop();
                    self.variable_manager.exit_function_scope();
                    
                    // Если есть return_slot, сохраняем результат в вызывающем фрейме
                    if let Some(return_slot) = &return_slot {
                        if let Some(caller_frame) = self.call_stack.last_mut() {
                            caller_frame.set_local(return_slot.clone(), value.clone());
                        }
                        // Возвращаем результат из return_slot вызывающего фрейма
                        if let Some(caller_frame) = self.call_stack.last() {
                            if let Some(result) = caller_frame.get_local(return_slot) {
                                return Ok(result.clone());
                            }
                        }
                    }
                    
                    // Если нет return_slot, возвращаем значение напрямую
                    return Ok(value);
                }
                ExecResult::Call { function_id: called_func_id, args } => {
                    // Вложенный вызов функции
                    let new_frame = CallFrame::new(
                        called_func_id.clone(),
                        args,
                        None,
                        current_depth + 1,
                    );
                    self.call_stack.push(new_frame)?;
                    
                    // Инициализируем локальные переменные новой функции
                    if let Some(new_frame) = self.call_stack.last_mut() {
                        let called_function = self.function_manager.get_function(&called_func_id)
                            .ok_or_else(|| DataCodeError::function_not_found(&called_func_id, self.current_line))?;
                        
                        let args = new_frame.args.clone();
                        for (param, arg_value) in called_function.parameters.iter().zip(args.iter()) {
                            new_frame.set_local(param.clone(), arg_value.clone());
                        }
                    }
                    // Продолжаем выполнение с нового фрейма
                    continue;
                }
                ExecResult::TailCall { function_id: tail_func_id, args } => {
                    // Хвостовой вызов - заменяем текущий фрейм
                    let (return_slot, depth) = {
                        let frame = self.call_stack.last()
                            .ok_or_else(|| DataCodeError::runtime_error(
                                "Internal error: no frame in stack",
                                self.current_line
                            ))?;
                        (frame.return_slot.clone(), frame.depth)
                    };
                    let new_frame = CallFrame::new(
                        tail_func_id.clone(),
                        args,
                        return_slot,
                        depth,
                    );
                    self.call_stack.replace_top(new_frame)?;
                    
                    // Инициализируем локальные переменные новой функции
                    if let Some(new_frame) = self.call_stack.last_mut() {
                        let called_function = self.function_manager.get_function(&tail_func_id)
                            .ok_or_else(|| DataCodeError::function_not_found(&tail_func_id, self.current_line))?;
                        
                        let args = new_frame.args.clone();
                        for (param, arg_value) in called_function.parameters.iter().zip(args.iter()) {
                            new_frame.set_local(param.clone(), arg_value.clone());
                        }
                    }
                    // Продолжаем выполнение с замененного фрейма
                    continue;
                }
            }
        }

        Err(DataCodeError::runtime_error(
            "Internal error: function execution ended unexpectedly",
            self.current_line
        ))
    }


    /// Выполнить вложенный вызов функции через trampoline
    /// Используется когда функция вызывает другую функцию из своего тела
    /// Это предотвращает рекурсию на уровне Rust стека
    /// 
    /// ВАЖНО: Этот метод выполняет вызов синхронно через тот же trampoline loop
    /// но без создания новой рекурсии на уровне Rust стека
    #[allow(dead_code)]
    fn execute_nested_function_call(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Сохраняем текущее состояние стека результатов
        let old_result_stack_len = self.trampoline_result_stack.len();
        
        // Добавляем вызов в очередь
        self.trampoline_queue.push((name.to_string(), args));
        
        // Обрабатываем вызовы из очереди до тех пор, пока не получим результат
        // ВАЖНО: Мы НЕ создаем новый trampoline loop, а используем тот же механизм
        // что и основной loop, но обрабатываем вызовы синхронно
        while !self.trampoline_queue.is_empty() {
            let (func_name, func_args) = self.trampoline_queue.remove(0); // Берем первый элемент
            
            // Выполняем один фрейм функции
            match self.execute_function_frame(&func_name, func_args)? {
                FunctionResult::Done(value) => {
                    // Функция завершилась, сохраняем результат
                    self.trampoline_result_stack.push(value);
                }
                FunctionResult::Continue { function_name, args } => {
                    // Требуется вызвать другую функцию, добавляем в очередь
                    self.trampoline_queue.push((function_name, args));
                }
            }
        }
        
        // Извлекаем результат из стека результатов
        if self.trampoline_result_stack.len() > old_result_stack_len {
            // Результат должен быть последним добавленным элементом
            let result = self.trampoline_result_stack.pop().unwrap();
            Ok(result)
        } else {
            Err(DataCodeError::runtime_error(
                "Internal error: nested function call did not produce a result",
                self.current_line
            ))
        }
    }

    /// Прямой вызов пользовательской функции (без trampoline)
    /// Используется когда trampoline отключен или для обратной совместимости
    fn call_user_function_direct(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        let function = self.function_manager.get_function(name)
            .ok_or_else(|| DataCodeError::function_not_found(name, self.current_line))?
            .clone();

        // Проверяем количество аргументов
        if function.parameters.len() != args.len() {
            return Err(DataCodeError::wrong_argument_count(
                name,
                function.parameters.len(),
                args.len(),
                self.current_line,
            ));
        }

        // Входим в область видимости функции (с проверкой рекурсии)
        self.enter_function_scope()?;

        // Устанавливаем параметры функции
        if let Err(e) = self.variable_manager
            .set_function_parameters(&function.parameters, args) {
            self.exit_function_scope();
            return Err(DataCodeError::runtime_error(&e, self.current_line));
        }

        // Сохраняем текущее возвращаемое значение
        let old_return_value = self.return_value.take();

        // Выполняем тело функции
        let body_lines: Vec<&str> = function.body.iter().map(|s| s.as_str()).collect();
        if std::env::var("DATACODE_DEBUG").is_ok() {
            eprintln!("🔍 DEBUG call_user_function_direct: Executing function '{}' with body:", name);
            for (i, line) in body_lines.iter().enumerate() {
                eprintln!("  [{}] '{}'", i, line);
            }
            eprintln!("🔍 DEBUG call_user_function_direct: Function execution is isolated from parser context");
        }
        let execution_result = execution::execute_block_directly(self, &body_lines);

        // Получаем результат выполнения функции
        let final_result = if let Some(return_val) = self.return_value.take() {
            return_val
        } else {
            Value::Null
        };

        // Восстанавливаем предыдущее возвращаемое значение
        self.return_value = old_return_value;

        // Выходим из области видимости функции
        self.exit_function_scope();

        // Проверяем результат выполнения
        execution_result?;

        Ok(final_result)
    }

    /// Добавить пользовательскую функцию
    #[allow(dead_code)]
    pub fn add_user_function(&mut self, function: UserFunction) {
        self.function_manager.add_function(function);
    }

    /// Проверить, существует ли пользовательская функция
    pub fn has_user_function(&self, name: &str) -> bool {
        self.function_manager.contains_function(name)
    }

    /// Сложение значений
    fn add_values(&self, left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => Ok(Number(a + b)),
            (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            (String(a), Number(b)) => Ok(String(format!("{}{}", a, b))),
            (Number(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot add {:?} and {:?}", left, right),
                self.current_line,
            )),
        }
    }

    /// Вычитание значений
    fn subtract_values(&self, left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => Ok(Number(a - b)),
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot subtract {:?} and {:?}", left, right),
                self.current_line,
            )),
        }
    }

    /// Умножение значений
    fn multiply_values(&self, left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => Ok(Number(a * b)),
            (String(s), Number(n)) => {
                if *n >= 0.0 && n.fract() == 0.0 {
                    let count = *n as usize;
                    Ok(String(s.repeat(count)))
                } else {
                    Err(DataCodeError::runtime_error(
                        "String multiplication requires non-negative integer",
                        self.current_line,
                    ))
                }
            }
            (Number(n), String(s)) => {
                if *n >= 0.0 && n.fract() == 0.0 {
                    let count = *n as usize;
                    Ok(String(s.repeat(count)))
                } else {
                    Err(DataCodeError::runtime_error(
                        "String multiplication requires non-negative integer",
                        self.current_line,
                    ))
                }
            }
            (Bool(b), Number(n)) => {
                // Bool(true) = 1.0, Bool(false) = 0.0
                Ok(Number(if *b { *n } else { 0.0 }))
            }
            (Number(n), Bool(b)) => {
                // Bool(true) = 1.0, Bool(false) = 0.0
                Ok(Number(if *b { *n } else { 0.0 }))
            }
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot multiply {:?} and {:?}", left, right),
                self.current_line,
            )),
        }
    }

    /// Деление значений
    fn divide_values(&self, left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => {
                if *b == 0.0 {
                    Err(DataCodeError::runtime_error("Division by zero", self.current_line))
                } else {
                    Ok(Number(a / b))
                }
            }
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot divide {:?} and {:?}", left, right),
                self.current_line,
            )),
        }
    }

    /// Остаток от деления значений
    fn modulo_values(&self, left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => {
                if *b == 0.0 {
                    Err(DataCodeError::runtime_error("Modulo by zero", self.current_line))
                } else {
                    Ok(Number(a % b))
                }
            }
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot modulo {:?} and {:?}", left, right),
                self.current_line,
            )),
        }
    }

    /// Сравнение значений на равенство
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Null, Null) => true,
            _ => false,
        }
    }

    /// Сравнение значений "меньше чем"
    fn less_than_values(&self, left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => Ok(Bool(a < b)),
            (String(a), String(b)) => Ok(Bool(a < b)),
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot compare {:?} and {:?}", left, right),
                self.current_line,
            )),
        }
    }

    /// Сравнение значений "больше чем"
    fn greater_than_values(&self, left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            (Number(a), Number(b)) => Ok(Bool(a > b)),
            (String(a), String(b)) => Ok(Bool(a > b)),
            _ => Err(DataCodeError::runtime_error(
                &format!("Cannot compare {:?} and {:?}", left, right),
                self.current_line,
            )),
        }
    }

    /// Преобразование значения в булево
    fn to_bool(&self, value: &Value) -> bool {
        use Value::*;
        match value {
            Bool(b) => *b,
            Number(n) => *n != 0.0,
            String(s) => !s.is_empty(),
            Null => false,
            _ => true,
        }
    }

    /// Обработать пользовательскую функцию в выражении рекурсивно
    fn handle_user_function_in_expression(&mut self, expr: &crate::parser::tokens::Expr) -> Result<Value> {
        use crate::parser::tokens::Expr;

        match expr {
            Expr::FunctionCall { name, args, named_args } => {
                if self.function_manager.contains_function(name) {
                    // Вычисляем позиционные аргументы в контексте интерпретатора
                    let mut arg_values = Vec::new();
                    for arg in args {
                        let arg_value = self.evaluate_expression(arg)?;
                        arg_values.push(arg_value);
                    }
                    
                    // Вычисляем именованные аргументы
                    let mut named_arg_values = std::collections::HashMap::new();
                    for (arg_name, arg_expr) in named_args {
                        let value = self.evaluate_expression(&arg_expr)?;
                        named_arg_values.insert(arg_name.clone(), value);
                    }
                    
                    // Пока пользовательские функции не поддерживают именованные аргументы
                    if !named_arg_values.is_empty() {
                        return Err(DataCodeError::runtime_error(
                            &format!("User functions do not support named arguments yet"),
                            self.current_line
                        ));
                    }

                    self.call_user_function(name, arg_values)
                } else {
                    Err(DataCodeError::function_not_found(name, self.current_line))
                }
            }

            Expr::Binary { left, operator, right } => {
                // Сначала пытаемся вычислить левую часть
                let left_val = match self.evaluate_expression(left) {
                    Ok(val) => val,
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        self.handle_user_function_in_expression(left)?
                    }
                    Err(e) => return Err(e)
                };

                // Затем пытаемся вычислить правую часть
                let right_val = match self.evaluate_expression(right) {
                    Ok(val) => val,
                    Err(e) if e.to_string().contains("USER_FUNCTION_CALL_EXPR:") => {
                        self.handle_user_function_in_expression(right)?
                    }
                    Err(e) => return Err(e)
                };

                // Выполняем бинарную операцию
                use crate::parser::tokens::BinaryOp;
                match operator {
                    BinaryOp::Add => self.add_values(&left_val, &right_val),
                    BinaryOp::Subtract => self.subtract_values(&left_val, &right_val),
                    BinaryOp::Multiply => self.multiply_values(&left_val, &right_val),
                    BinaryOp::Divide => self.divide_values(&left_val, &right_val),
                    BinaryOp::Modulo => self.modulo_values(&left_val, &right_val),
                    BinaryOp::Equal => Ok(Value::Bool(self.values_equal(&left_val, &right_val))),
                    BinaryOp::NotEqual => Ok(Value::Bool(!self.values_equal(&left_val, &right_val))),
                    BinaryOp::Less => self.less_than_values(&left_val, &right_val),
                    BinaryOp::Greater => self.greater_than_values(&left_val, &right_val),
                    BinaryOp::LessEqual => {
                        let less = self.less_than_values(&left_val, &right_val)?;
                        let equal = self.values_equal(&left_val, &right_val);
                        Ok(Value::Bool(less.as_bool().unwrap_or(false) || equal))
                    }
                    BinaryOp::GreaterEqual => {
                        let greater = self.greater_than_values(&left_val, &right_val)?;
                        let equal = self.values_equal(&left_val, &right_val);
                        Ok(Value::Bool(greater.as_bool().unwrap_or(false) || equal))
                    }
                    BinaryOp::And => {
                        let left_bool = self.to_bool(&left_val);
                        if !left_bool {
                            Ok(Value::Bool(false))
                        } else {
                            Ok(Value::Bool(self.to_bool(&right_val)))
                        }
                    }
                    BinaryOp::Or => {
                        let left_bool = self.to_bool(&left_val);
                        if left_bool {
                            Ok(Value::Bool(true))
                        } else {
                            Ok(Value::Bool(self.to_bool(&right_val)))
                        }
                    }
                    _ => Err(DataCodeError::runtime_error(
                        &format!("Unsupported binary operator: {:?}", operator),
                        self.current_line,
                    )),
                }
            }

            _ => {
                // Для других типов выражений просто пытаемся их вычислить
                self.evaluate_expression(expr)
            }
        }
    }



    /// Войти в область видимости цикла
    pub fn enter_loop_scope(&mut self) {
        self.variable_manager.enter_loop_scope();
    }

    /// Выйти из области видимости цикла
    pub fn exit_loop_scope(&mut self) {
        self.variable_manager.exit_loop_scope();
    }

    /// Очистить все данные интерпретатора
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.variable_manager.clear();
        self.function_manager.clear();
        self.return_value = None;
        self.break_requested = false;
        self.active_loop_count = 0;
        self.current_line = 1;
        self.exception_stack.clear();
        self.recursion_depth = 0;
        self.call_stack = CallStack::new(1_000_000);
        self.trampoline_queue.clear();
        self.trampoline_result_stack.clear();
        self.in_trampoline_loop = false;
    }

    // === Методы для управления стеком исключений ===

    /// Добавить блок try/catch в стек исключений
    pub fn push_try_block(&mut self, try_block: TryBlock) {
        self.exception_stack.push(try_block);
    }

    /// Удалить блок try/catch из стека исключений
    pub fn pop_try_block(&mut self) -> Option<TryBlock> {
        self.exception_stack.pop()
    }

    /// Найти подходящий обработчик исключений в стеке
    /// Возвращает индекс блока в стеке, который может обработать исключение
    #[allow(dead_code)]
    pub fn find_exception_handler(&self) -> Option<usize> {
        // Ищем с конца стека (самый вложенный блок)
        for (index, try_block) in self.exception_stack.iter().enumerate().rev() {
            if try_block.can_handle_exception() {
                return Some(index);
            }
        }
        None
    }

    /// Получить текущий уровень вложенности try/catch блоков
    pub fn get_try_nesting_level(&self) -> usize {
        self.exception_stack.len()
    }

    /// Деактивировать все блоки try/catch до указанного уровня (включительно)
    /// Используется при обработке исключений для предотвращения повторной обработки
    #[allow(dead_code)]
    pub fn deactivate_try_blocks_until(&mut self, until_index: usize) {
        for i in until_index..self.exception_stack.len() {
            if let Some(try_block) = self.exception_stack.get_mut(i) {
                try_block.deactivate();
            }
        }
    }

    /// Получить следующий уникальный ID для блока try/catch
    pub fn get_next_try_block_id(&self) -> usize {
        // Простая реализация - используем текущий размер стека + 1
        // В более сложной реализации можно использовать глобальный счетчик
        self.exception_stack.len() + 1
    }

    /// Войти в область видимости функции (увеличить глубину рекурсии)
    pub fn enter_function_scope(&mut self) -> Result<()> {
        // Проверяем лимит рекурсии ПЕРЕД увеличением глубины
        // Это критически важно для предотвращения переполнения Rust стека
        const MAX_RECURSION_DEPTH: usize = 1000; // Уменьшено для раннего обнаружения проблемы
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            return Err(DataCodeError::runtime_error(
                &format!("Превышена максимальная глубина рекурсии ({})", MAX_RECURSION_DEPTH),
                self.current_line
            ));
        }

        self.recursion_depth += 1;
        self.variable_manager.enter_function_scope();
        Ok(())
    }

    /// Выйти из области видимости функции (уменьшить глубину рекурсии)
    pub fn exit_function_scope(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
        self.variable_manager.exit_function_scope();
    }

    /// Проверить, находимся ли мы в функции
    #[allow(dead_code)]
    pub fn is_in_function(&self) -> bool {
        self.recursion_depth > 0
    }

    /// Получить текущую глубину рекурсии
    #[allow(dead_code)]
    pub fn get_recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    /// Развернуть spread аргумент в список значений
    fn expand_spread_argument(&self, spread_value: Value, arg_values: &mut Vec<Value>) -> Result<()> {
        match spread_value {
            Value::Object(obj) => {
                // Для объектов добавляем значения в порядке ключей
                // Сначала собираем ключи и сортируем их для предсказуемого порядка
                let mut keys: Vec<_> = obj.keys().collect();
                keys.sort();

                for key in keys {
                    if let Some(value) = obj.get(key) {
                        arg_values.push(value.clone());
                    }
                }
                Ok(())
            }
            Value::Array(arr) => {
                // Для массивов добавляем все элементы
                for item in arr {
                    arg_values.push(item);
                }
                Ok(())
            }
            _ => Err(DataCodeError::runtime_error(
                "Spread operator can only be used with objects or arrays",
                self.current_line
            ))
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter_creation() {
        let interp = Interpreter::new();
        assert_eq!(interp.current_line, 1);
        assert!(interp.return_value.is_none());
        assert!(interp.exception_stack.is_empty());
    }

    #[test]
    fn test_variable_operations() {
        let mut interp = Interpreter::new();
        
        // Установить переменную
        interp.set_variable("test".to_string(), Value::Number(42.0), true);
        
        // Получить переменную
        assert_eq!(interp.get_variable("test"), Some(&Value::Number(42.0)));
        
        // Проверить несуществующую переменную
        assert_eq!(interp.get_variable("nonexistent"), None);
    }

    #[test]
    fn test_user_function_management() {
        let mut interp = Interpreter::new();
        
        let func = UserFunction::new(
            "test_func".to_string(),
            vec!["x".to_string()],
            vec!["return x * 2".to_string()],
            true,
        );
        
        interp.add_user_function(func);
        assert!(interp.has_user_function("test_func"));
        assert!(!interp.has_user_function("nonexistent"));
    }
}
