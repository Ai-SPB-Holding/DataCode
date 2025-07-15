// Статический анализатор для DataCode
// Проверяет существование переменных, типы и другие ошибки до выполнения

use crate::parser::{Expr, BinaryOp, UnaryOp};
use crate::value::{Value, DataType};
use crate::error::Result;
use std::collections::{HashMap, HashSet};

/// Статический анализатор
pub struct StaticAnalyzer {
    error_count: usize,
    warnings: Vec<String>,
}

impl StaticAnalyzer {
    /// Создать новый статический анализатор
    pub fn new() -> Self {
        Self {
            error_count: 0,
            warnings: Vec::new(),
        }
    }
    
    /// Анализировать выражение
    pub fn analyze(&mut self, expr: &Expr) -> Result<AnalysisResult> {
        let mut context = AnalysisContext::new();
        self.analyze_expression(expr, &mut context)?;
        
        Ok(AnalysisResult {
            variables_used: context.variables_used,
            functions_called: context.functions_called,
            potential_errors: context.potential_errors,
            type_info: context.type_info,
            warnings: self.warnings.clone(),
        })
    }
    
    /// Анализировать выражение с контекстом переменных
    pub fn analyze_with_context(&mut self, expr: &Expr, variables: &HashMap<String, Value>) -> Result<AnalysisResult> {
        let mut context = AnalysisContext::with_variables(variables);
        self.analyze_expression(expr, &mut context)?;
        
        Ok(AnalysisResult {
            variables_used: context.variables_used,
            functions_called: context.functions_called,
            potential_errors: context.potential_errors,
            type_info: context.type_info,
            warnings: self.warnings.clone(),
        })
    }
    
    /// Получить количество ошибок
    pub fn get_error_count(&self) -> usize {
        self.error_count
    }
    
    /// Получить предупреждения
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }
    
    /// Очистить состояние анализатора
    pub fn clear(&mut self) {
        self.error_count = 0;
        self.warnings.clear();
    }
    
    /// Анализировать выражение рекурсивно
    fn analyze_expression(&mut self, expr: &Expr, context: &mut AnalysisContext) -> Result<DataType> {
        match expr {
            Expr::Literal(value) => {
                Ok(DataType::from_value(value))
            }
            Expr::Variable(name) => {
                context.variables_used.insert(name.clone());
                
                // Проверяем, существует ли переменная
                if let Some(value) = context.known_variables.get(name) {
                    Ok(DataType::from_value(value))
                } else {
                    // Переменная не найдена - это потенциальная ошибка
                    context.potential_errors.push(format!("Variable '{}' may not be defined", name));
                    Ok(DataType::Mixed) // Предполагаем смешанный тип
                }
            }
            Expr::Binary { left, operator, right } => {
                let left_type = self.analyze_expression(left, context)?;
                let right_type = self.analyze_expression(right, context)?;
                
                self.analyze_binary_operation(&left_type, operator, &right_type, context)
            }
            Expr::Unary { operator, operand } => {
                let operand_type = self.analyze_expression(operand, context)?;
                self.analyze_unary_operation(operator, &operand_type, context)
            }
            Expr::FunctionCall { name, args } => {
                context.functions_called.insert(name.clone());
                
                // Анализируем аргументы
                let mut arg_types = Vec::new();
                for arg in args {
                    let arg_type = self.analyze_expression(arg, context)?;
                    arg_types.push(arg_type);
                }
                
                self.analyze_function_call(name, &arg_types, context)
            }
            Expr::Index { object, index } => {
                let object_type = self.analyze_expression(object, context)?;
                let index_type = self.analyze_expression(index, context)?;
                
                self.analyze_indexing(&object_type, &index_type, context)
            }
            Expr::Member { object, member } => {
                let object_type = self.analyze_expression(object, context)?;
                self.analyze_member_access(&object_type, member, context)
            }
            Expr::ArrayLiteral { elements } => {
                let mut element_types = Vec::new();
                for element in elements {
                    let element_type = self.analyze_expression(element, context)?;
                    element_types.push(element_type);
                }
                
                // Проверяем однородность типов в массиве
                if !element_types.is_empty() {
                    let first_type = &element_types[0];
                    if !element_types.iter().all(|t| t == first_type) {
                        self.warnings.push("Array contains mixed types".to_string());
                    }
                }
                
                Ok(DataType::Mixed) // Массивы представляем как смешанный тип
            }
            Expr::ObjectLiteral { pairs } => {
                for (key, value) in pairs {
                    let value_type = self.analyze_expression(value, context)?;
                    context.type_info.insert(format!("object.{}", key), value_type);
                }
                Ok(DataType::Mixed) // Объекты представляем как смешанный тип
            }
            Expr::Spread { expression } => {
                // Анализируем выражение внутри spread оператора
                self.analyze_expression(expression, context)?;
                Ok(DataType::Mixed) // Spread может возвращать различные типы
            }
            Expr::TryBlock { .. } => {
                // Try блоки не возвращают значения в выражениях
                Ok(DataType::Null)
            }
            Expr::ThrowStatement { message } => {
                self.analyze_expression(message, context)?;
                Ok(DataType::Null) // throw не возвращает значение
            }
        }
    }
    
    /// Анализировать бинарную операцию
    fn analyze_binary_operation(&mut self, left_type: &DataType, op: &BinaryOp, right_type: &DataType, context: &mut AnalysisContext) -> Result<DataType> {
        match op {
            BinaryOp::Add => {
                match (left_type, right_type) {
                    (DataType::Integer, DataType::Integer) => Ok(DataType::Integer),
                    (DataType::Float, DataType::Float) => Ok(DataType::Float),
                    (DataType::Integer, DataType::Float) | (DataType::Float, DataType::Integer) => Ok(DataType::Float),
                    (DataType::String, DataType::String) => Ok(DataType::String),
                    _ => {
                        context.potential_errors.push(format!("Type mismatch in addition: {:?} + {:?}", left_type, right_type));
                        Ok(DataType::Mixed)
                    }
                }
            }
            BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
                match (left_type, right_type) {
                    (DataType::Integer, DataType::Integer) => Ok(DataType::Integer),
                    (DataType::Float, DataType::Float) => Ok(DataType::Float),
                    (DataType::Integer, DataType::Float) | (DataType::Float, DataType::Integer) => Ok(DataType::Float),
                    _ => {
                        context.potential_errors.push(format!("Arithmetic operation requires numbers: {:?} {:?} {:?}", left_type, op, right_type));
                        Ok(DataType::Mixed)
                    }
                }
            }
            BinaryOp::Equal | BinaryOp::NotEqual => {
                // Сравнение возможно между любыми типами
                Ok(DataType::Bool)
            }
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual => {
                match (left_type, right_type) {
                    (DataType::Integer, DataType::Integer) | (DataType::Float, DataType::Float) => Ok(DataType::Bool),
                    (DataType::Integer, DataType::Float) | (DataType::Float, DataType::Integer) => Ok(DataType::Bool),
                    (DataType::String, DataType::String) => Ok(DataType::Bool),
                    _ => {
                        context.potential_errors.push(format!("Comparison requires compatible types: {:?} {:?} {:?}", left_type, op, right_type));
                        Ok(DataType::Bool) // Все равно возвращаем bool
                    }
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                // Логические операции работают с любыми типами (приводятся к bool)
                Ok(DataType::Bool)
            }
            BinaryOp::PathJoin => {
                // Упрощенная обработка - возвращаем строку
                Ok(DataType::String)
            }
        }
    }
    
    /// Анализировать унарную операцию
    fn analyze_unary_operation(&mut self, op: &UnaryOp, operand_type: &DataType, context: &mut AnalysisContext) -> Result<DataType> {
        match op {
            UnaryOp::Not => Ok(DataType::Bool),
            UnaryOp::Minus => {
                match operand_type {
                    DataType::Integer => Ok(DataType::Integer),
                    DataType::Float => Ok(DataType::Float),
                    _ => {
                        context.potential_errors.push(format!("Unary minus requires number, got {:?}", operand_type));
                        Ok(DataType::Mixed)
                    }
                }
            }
        }
    }
    
    /// Анализировать вызов функции
    fn analyze_function_call(&mut self, name: &str, arg_types: &[DataType], context: &mut AnalysisContext) -> Result<DataType> {
        match name {
            "table" | "table_create" => {
                if arg_types.len() >= 1 {
                    Ok(DataType::Mixed) // Таблицы представляем как смешанный тип
                } else {
                    context.potential_errors.push("table() requires at least 1 argument".to_string());
                    Ok(DataType::Mixed)
                }
            }
            "table_filter" | "table_where" | "table_select" | "table_head" | "table_tail" => {
                if arg_types.len() >= 1 {
                    Ok(DataType::Mixed) // Все табличные функции возвращают таблицы
                } else {
                    context.potential_errors.push(format!("{}() requires at least 1 argument", name));
                    Ok(DataType::Mixed)
                }
            }
            "len" | "length" => {
                if arg_types.len() == 1 {
                    Ok(DataType::Integer) // len возвращает целое число
                } else {
                    context.potential_errors.push("len() requires exactly 1 argument".to_string());
                    Ok(DataType::Mixed)
                }
            }
            "print" | "println" => {
                // print принимает любые аргументы и ничего не возвращает
                Ok(DataType::Null)
            }
            _ => {
                // Неизвестная функция - предупреждение, но не ошибка
                self.warnings.push(format!("Unknown function: {}", name));
                Ok(DataType::Mixed)
            }
        }
    }
    
    /// Анализировать индексацию
    fn analyze_indexing(&mut self, _object_type: &DataType, index_type: &DataType, context: &mut AnalysisContext) -> Result<DataType> {
        // Упрощенная проверка - индекс должен быть числом или строкой
        match index_type {
            DataType::Integer | DataType::Float | DataType::String => {
                Ok(DataType::Mixed) // Результат индексации может быть любого типа
            }
            _ => {
                context.potential_errors.push(format!("Invalid index type: {:?}", index_type));
                Ok(DataType::Mixed)
            }
        }
    }

    /// Анализировать доступ к члену
    fn analyze_member_access(&mut self, _object_type: &DataType, _member: &str, _context: &mut AnalysisContext) -> Result<DataType> {
        // Упрощенная проверка - доступ к члену всегда возможен
        Ok(DataType::Mixed) // Результат доступа к члену может быть любого типа
    }
}

impl Default for StaticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Контекст анализа
struct AnalysisContext {
    known_variables: HashMap<String, Value>,
    variables_used: HashSet<String>,
    functions_called: HashSet<String>,
    potential_errors: Vec<String>,
    type_info: HashMap<String, DataType>,
}

impl AnalysisContext {
    fn new() -> Self {
        Self {
            known_variables: HashMap::new(),
            variables_used: HashSet::new(),
            functions_called: HashSet::new(),
            potential_errors: Vec::new(),
            type_info: HashMap::new(),
        }
    }
    
    fn with_variables(variables: &HashMap<String, Value>) -> Self {
        Self {
            known_variables: variables.clone(),
            variables_used: HashSet::new(),
            functions_called: HashSet::new(),
            potential_errors: Vec::new(),
            type_info: HashMap::new(),
        }
    }
}

/// Результат статического анализа
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub variables_used: HashSet<String>,
    pub functions_called: HashSet<String>,
    pub potential_errors: Vec<String>,
    pub type_info: HashMap<String, DataType>,
    pub warnings: Vec<String>,
}

impl AnalysisResult {
    /// Проверить, есть ли ошибки
    pub fn has_errors(&self) -> bool {
        !self.potential_errors.is_empty()
    }
    
    /// Проверить, есть ли предупреждения
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
    
    /// Получить все проблемы (ошибки + предупреждения)
    pub fn get_all_issues(&self) -> Vec<String> {
        let mut issues = self.potential_errors.clone();
        issues.extend(self.warnings.clone());
        issues
    }
}
