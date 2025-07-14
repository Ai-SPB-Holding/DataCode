use std::collections::HashMap;

/// Структура для представления пользовательской функции
#[derive(Debug, Clone)]
pub struct UserFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<String>,
    pub is_global: bool,
}

impl UserFunction {
    pub fn new(name: String, parameters: Vec<String>, body: Vec<String>, is_global: bool) -> Self {
        Self {
            name,
            parameters,
            body,
            is_global,
        }
    }
}

/// Структура для представления блока try/catch с полной поддержкой стека исключений
#[derive(Debug, Clone)]
pub struct TryBlock {
    /// Переменная для хранения сообщения об ошибке в блоке catch
    pub catch_var: Option<String>,
    /// Тело блока catch
    pub catch_body: Vec<String>,
    /// Тело блока finally (выполняется всегда)
    pub finally_body: Option<Vec<String>>,
    /// Уникальный идентификатор блока для отслеживания в стеке
    pub block_id: usize,
    /// Уровень вложенности блока (для отладки)
    pub nesting_level: usize,
    /// Флаг, указывающий, что блок активен (находится в процессе выполнения)
    pub is_active: bool,
}

impl TryBlock {
    /// Создать новый блок try/catch
    pub fn new(
        catch_var: Option<String>,
        catch_body: Vec<String>,
        finally_body: Option<Vec<String>>,
        block_id: usize,
        nesting_level: usize,
    ) -> Self {
        Self {
            catch_var,
            catch_body,
            finally_body,
            block_id,
            nesting_level,
            is_active: true,
        }
    }

    /// Проверить, может ли этот блок обработать исключение
    pub fn can_handle_exception(&self) -> bool {
        self.is_active && !self.catch_body.is_empty()
    }

    /// Деактивировать блок (когда он завершает выполнение)
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

/// Менеджер пользовательских функций
pub struct UserFunctionManager {
    functions: HashMap<String, UserFunction>,
}

impl UserFunctionManager {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, function: UserFunction) {
        self.functions.insert(function.name.clone(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&UserFunction> {
        self.functions.get(name)
    }

    pub fn contains_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn get_all_functions(&self) -> &HashMap<String, UserFunction> {
        &self.functions
    }

    pub fn remove_function(&mut self, name: &str) -> Option<UserFunction> {
        self.functions.remove(name)
    }

    pub fn clear(&mut self) {
        self.functions.clear();
    }
}

impl Default for UserFunctionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_function_creation() {
        let func = UserFunction::new(
            "test_func".to_string(),
            vec!["param1".to_string(), "param2".to_string()],
            vec!["return param1 + param2".to_string()],
            true,
        );

        assert_eq!(func.name, "test_func");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.body.len(), 1);
        assert!(func.is_global);
    }

    #[test]
    fn test_user_function_manager() {
        let mut manager = UserFunctionManager::new();
        
        let func = UserFunction::new(
            "add".to_string(),
            vec!["a".to_string(), "b".to_string()],
            vec!["return a + b".to_string()],
            true,
        );

        // Добавляем функцию
        manager.add_function(func);

        // Проверяем, что функция добавлена
        assert!(manager.contains_function("add"));
        assert!(manager.get_function("add").is_some());

        // Проверяем, что несуществующая функция не найдена
        assert!(!manager.contains_function("subtract"));
        assert!(manager.get_function("subtract").is_none());

        // Удаляем функцию
        let removed = manager.remove_function("add");
        assert!(removed.is_some());
        assert!(!manager.contains_function("add"));
    }

    #[test]
    fn test_try_block_creation() {
        let try_block = TryBlock::new(
            Some("error".to_string()),
            vec!["print(error)".to_string()],
            Some(vec!["cleanup()".to_string()]),
            1,
            0,
        );

        assert_eq!(try_block.catch_var, Some("error".to_string()));
        assert_eq!(try_block.catch_body.len(), 1);
        assert!(try_block.finally_body.is_some());
        assert_eq!(try_block.block_id, 1);
        assert_eq!(try_block.nesting_level, 0);
        assert!(try_block.is_active);
        assert!(try_block.can_handle_exception());
    }
}
