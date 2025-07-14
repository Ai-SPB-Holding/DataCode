use crate::value::Value;
use std::collections::HashMap;

/// Менеджер переменных с поддержкой областей видимости
pub struct VariableManager {
    /// Глобальные переменные
    pub global_variables: HashMap<String, Value>,
    /// Стек локальных переменных для функций
    pub call_stack: Vec<HashMap<String, Value>>,
    /// Стек локальных переменных для циклов
    pub loop_stack: Vec<HashMap<String, Value>>,
}

impl VariableManager {
    pub fn new() -> Self {
        Self {
            global_variables: HashMap::new(),
            call_stack: Vec::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Получить переменную с учетом областей видимости
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        // Сначала ищем в локальных переменных циклов (проверяем все уровни, начиная с последнего)
        for loop_vars in self.loop_stack.iter().rev() {
            if let Some(value) = loop_vars.get(name) {
                return Some(value);
            }
        }

        // Затем ищем в локальных переменных функций (стек вызовов)
        if let Some(local_vars) = self.call_stack.last() {
            if let Some(value) = local_vars.get(name) {
                return Some(value);
            }
        }

        // Затем в глобальных переменных
        self.global_variables.get(name)
    }

    /// Установить переменную
    pub fn set_variable(&mut self, name: String, value: Value, is_global: bool) {
        if is_global || self.call_stack.is_empty() {
            // Глобальная переменная или мы не в функции
            self.global_variables.insert(name, value);
        } else {
            // Локальная переменная в функции
            if let Some(local_vars) = self.call_stack.last_mut() {
                local_vars.insert(name, value);
            }
        }
    }

    /// Специальный метод для установки переменной цикла
    pub fn set_loop_variable(&mut self, name: String, value: Value) {
        if let Some(loop_vars) = self.loop_stack.last_mut() {
            loop_vars.insert(name, value);
        }
    }

    /// Получить все глобальные переменные
    pub fn get_all_global_variables(&self) -> &HashMap<String, Value> {
        &self.global_variables
    }

    /// Войти в новую область видимости функции
    pub fn enter_function_scope(&mut self) {
        self.call_stack.push(HashMap::new());
    }

    /// Выйти из области видимости функции
    pub fn exit_function_scope(&mut self) {
        self.call_stack.pop();
    }

    /// Войти в новую область видимости цикла
    pub fn enter_loop_scope(&mut self) {
        self.loop_stack.push(HashMap::new());
    }

    /// Выйти из области видимости цикла
    pub fn exit_loop_scope(&mut self) {
        self.loop_stack.pop();
    }

    /// Установить параметры функции в локальной области видимости
    pub fn set_function_parameters(&mut self, parameters: &[String], arguments: Vec<Value>) -> Result<(), String> {
        if parameters.len() != arguments.len() {
            return Err(format!(
                "Неверное количество аргументов: ожидалось {}, получено {}",
                parameters.len(),
                arguments.len()
            ));
        }

        if let Some(local_vars) = self.call_stack.last_mut() {
            for (param, arg) in parameters.iter().zip(arguments.into_iter()) {
                local_vars.insert(param.clone(), arg);
            }
        }

        Ok(())
    }

    /// Очистить все переменные
    pub fn clear(&mut self) {
        self.global_variables.clear();
        self.call_stack.clear();
        self.loop_stack.clear();
    }

    /// Получить количество уровней вложенности функций
    pub fn function_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Получить количество уровней вложенности циклов
    pub fn loop_depth(&self) -> usize {
        self.loop_stack.len()
    }

    /// Проверить, находимся ли мы в функции
    pub fn is_in_function(&self) -> bool {
        !self.call_stack.is_empty()
    }

    /// Проверить, находимся ли мы в цикле
    pub fn is_in_loop(&self) -> bool {
        !self.loop_stack.is_empty()
    }
}

impl Default for VariableManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_variables() {
        let mut vm = VariableManager::new();
        
        vm.set_variable("x".to_string(), Value::Number(42.0), true);
        assert_eq!(vm.get_variable("x"), Some(&Value::Number(42.0)));
        
        vm.set_variable("y".to_string(), Value::String("hello".to_string()), true);
        assert_eq!(vm.get_variable("y"), Some(&Value::String("hello".to_string())));
    }

    #[test]
    fn test_function_scope() {
        let mut vm = VariableManager::new();
        
        // Глобальная переменная
        vm.set_variable("global_var".to_string(), Value::Number(1.0), true);
        
        // Входим в функцию
        vm.enter_function_scope();
        vm.set_variable("local_var".to_string(), Value::Number(2.0), false);
        
        // Локальная переменная должна быть видна
        assert_eq!(vm.get_variable("local_var"), Some(&Value::Number(2.0)));
        // Глобальная переменная тоже должна быть видна
        assert_eq!(vm.get_variable("global_var"), Some(&Value::Number(1.0)));
        
        // Выходим из функции
        vm.exit_function_scope();
        
        // Локальная переменная больше не видна
        assert_eq!(vm.get_variable("local_var"), None);
        // Глобальная переменная все еще видна
        assert_eq!(vm.get_variable("global_var"), Some(&Value::Number(1.0)));
    }

    #[test]
    fn test_loop_scope() {
        let mut vm = VariableManager::new();
        
        vm.enter_loop_scope();
        vm.set_loop_variable("i".to_string(), Value::Number(0.0));
        
        assert_eq!(vm.get_variable("i"), Some(&Value::Number(0.0)));
        
        vm.exit_loop_scope();
        assert_eq!(vm.get_variable("i"), None);
    }

    #[test]
    fn test_function_parameters() {
        let mut vm = VariableManager::new();
        
        vm.enter_function_scope();
        
        let params = vec!["a".to_string(), "b".to_string()];
        let args = vec![Value::Number(1.0), Value::Number(2.0)];
        
        vm.set_function_parameters(&params, args).unwrap();
        
        assert_eq!(vm.get_variable("a"), Some(&Value::Number(1.0)));
        assert_eq!(vm.get_variable("b"), Some(&Value::Number(2.0)));
        
        vm.exit_function_scope();
    }
}
