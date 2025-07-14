use crate::value::Value;
use crate::error::{DataCodeError, Result};
use crate::builtins::call_builtin_function;
use std::collections::HashMap;

// Подмодули
pub mod user_functions;
pub mod variables;
pub mod expressions;
pub mod execution;
pub mod control_flow;

// Реэкспорт основных типов
pub use user_functions::{UserFunction, TryBlock, UserFunctionManager};
pub use variables::VariableManager;
pub use expressions::ExpressionEvaluator;

/// Основная структура интерпретатора DataCode
pub struct Interpreter {
    /// Менеджер переменных с поддержкой областей видимости
    pub variable_manager: VariableManager,
    /// Менеджер пользовательских функций
    pub function_manager: UserFunctionManager,
    /// Возвращаемое значение функции
    pub return_value: Option<Value>,
    /// Текущая строка для отслеживания ошибок
    pub current_line: usize,
    /// Стек блоков try/catch
    pub exception_stack: Vec<TryBlock>,
    /// Глубина рекурсии для предотвращения переполнения стека
    pub recursion_depth: usize,
}

impl Interpreter {
    /// Создать новый интерпретатор
    pub fn new() -> Self {
        Self {
            variable_manager: VariableManager::new(),
            function_manager: UserFunctionManager::new(),
            return_value: None,
            current_line: 1,
            exception_stack: Vec::new(),
            recursion_depth: 0,
        }
    }

    /// Получить переменную
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variable_manager.get_variable(name)
    }

    /// Установить переменную
    pub fn set_variable(&mut self, name: String, value: Value, is_global: bool) {
        self.variable_manager.set_variable(name, value, is_global);
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
        if trimmed_expr.is_empty() {
            return Ok(Value::Null);
        }

        // Парсим выражение
        let mut parser = crate::parser::Parser::new(trimmed_expr);
        let parsed_expr = parser.parse_expression()?;

        // Вычисляем с поддержкой пользовательских функций
        self.evaluate_expression(&parsed_expr)
    }

    /// Выполнить строку кода (будет реализовано в execution.rs)
    pub fn exec(&mut self, line: &str) -> Result<()> {
        execution::execute_line(self, line)
    }

    /// Выполнить многострочный код
    pub fn exec_multiline(&mut self, code: &str) -> Result<()> {
        execution::execute_multiline(self, code)
    }

    /// Определить пользовательскую функцию
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

            Expr::FunctionCall { name, args } => {
                // Сначала вычисляем аргументы
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate_expression(arg)?);
                }

                // Проверяем, является ли это пользовательской функцией
                if self.function_manager.contains_function(name) {
                    self.call_user_function(name, arg_values)
                } else {
                    // Встроенная функция
                    call_builtin_function(name, arg_values, self.current_line)
                }
            }

            _ => {
                // Для остальных типов выражений используем ExpressionEvaluator
                let evaluator = ExpressionEvaluator::new(
                    &self.variable_manager,
                    &self.function_manager,
                    self.current_line,
                );
                evaluator.evaluate(expr)
            }
        }
    }

    /// Вызвать пользовательскую функцию
    fn call_user_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Проверяем глубину рекурсии
        if self.recursion_depth > 5 {
            return Err(DataCodeError::runtime_error(
                &format!("Maximum recursion depth exceeded when calling function '{}' (depth: {})", name, self.recursion_depth),
                self.current_line
            ));
        }

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

        // Увеличиваем глубину рекурсии
        self.recursion_depth += 1;

        // Входим в новую область видимости
        self.variable_manager.enter_function_scope();

        // Устанавливаем параметры функции
        self.variable_manager
            .set_function_parameters(&function.parameters, args)
            .map_err(|e| DataCodeError::runtime_error(&e, self.current_line))?;

        // Сохраняем текущее возвращаемое значение
        let old_return_value = self.return_value.take();

        // Выполняем тело функции напрямую без рекурсии
        if let Err(e) = execution::execute_block_directly(self, &function.body.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
            // Восстанавливаем состояние при ошибке
            self.variable_manager.exit_function_scope();
            self.return_value = old_return_value;
            self.recursion_depth -= 1;
            return Err(e);
        }

        // Получаем результат выполнения функции
        let result = if let Some(return_val) = &self.return_value {
            return_val.clone()
        } else {
            Value::Null
        };

        // Обновляем результат если был вызван return
        let final_result = if self.return_value.is_some() {
            self.return_value.take().unwrap()
        } else {
            result
        };

        // Выходим из области видимости
        self.variable_manager.exit_function_scope();

        // Восстанавливаем предыдущее возвращаемое значение
        self.return_value = old_return_value;

        // Уменьшаем глубину рекурсии
        self.recursion_depth -= 1;

        Ok(final_result)
    }

    /// Добавить пользовательскую функцию
    pub fn add_user_function(&mut self, function: UserFunction) {
        self.function_manager.add_function(function);
    }

    /// Проверить, существует ли пользовательская функция
    pub fn has_user_function(&self, name: &str) -> bool {
        self.function_manager.contains_function(name)
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
    pub fn clear(&mut self) {
        self.variable_manager.clear();
        self.function_manager.clear();
        self.return_value = None;
        self.current_line = 1;
        self.exception_stack.clear();
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
