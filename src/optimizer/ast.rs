// AST оптимизатор для DataCode
// Реализует различные оптимизации AST перед выполнением

use crate::parser::{Expr, BinaryOp, UnaryOp};
use crate::value::Value;
use crate::error::{DataCodeError, Result};


/// AST оптимизатор
pub struct ASTOptimizer {
    optimization_count: usize,
}

impl ASTOptimizer {
    /// Создать новый AST оптимизатор
    pub fn new() -> Self {
        Self {
            optimization_count: 0,
        }
    }
    
    /// Оптимизировать выражение
    pub fn optimize(&mut self, expr: Expr) -> Result<Expr> {
        let mut optimized = expr;
        
        // Применяем оптимизации в порядке эффективности
        optimized = self.fold_constants(optimized)?;
        optimized = self.combine_filters(optimized)?;
        optimized = self.remove_dead_code(optimized)?;
        optimized = self.simplify_boolean_expressions(optimized)?;
        optimized = self.optimize_function_calls(optimized)?;
        
        Ok(optimized)
    }
    
    /// Получить количество выполненных оптимизаций
    pub fn get_optimization_count(&self) -> usize {
        self.optimization_count
    }
    
    /// Свертка констант: 10 + 5 → 15
    fn fold_constants(&mut self, expr: Expr) -> Result<Expr> {
        match expr {
            Expr::Binary { left, operator, right } => {
                let left_opt = self.fold_constants(*left)?;
                let right_opt = self.fold_constants(*right)?;
                
                // Если оба операнда - литералы, вычисляем результат
                if let (Expr::Literal(left_val), Expr::Literal(right_val)) = (&left_opt, &right_opt) {
                    if let Ok(result) = self.evaluate_constant_binary(left_val, &operator, right_val) {
                        self.optimization_count += 1;
                        return Ok(Expr::Literal(result));
                    }
                }
                
                Ok(Expr::Binary {
                    left: Box::new(left_opt),
                    operator,
                    right: Box::new(right_opt),
                })
            }
            Expr::Unary { operator, operand } => {
                let operand_opt = self.fold_constants(*operand)?;
                
                // Если операнд - литерал, вычисляем результат
                if let Expr::Literal(operand_val) = &operand_opt {
                    if let Ok(result) = self.evaluate_constant_unary(&operator, operand_val) {
                        self.optimization_count += 1;
                        return Ok(Expr::Literal(result));
                    }
                }
                
                Ok(Expr::Unary {
                    operator,
                    operand: Box::new(operand_opt),
                })
            }
            Expr::FunctionCall { name, args } => {
                let optimized_args: Result<Vec<Expr>> = args.into_iter()
                    .map(|arg| self.fold_constants(arg))
                    .collect();
                
                Ok(Expr::FunctionCall {
                    name,
                    args: optimized_args?,
                })
            }
            Expr::ArrayLiteral { elements } => {
                let optimized_elements: Result<Vec<Expr>> = elements.into_iter()
                    .map(|elem| self.fold_constants(elem))
                    .collect();
                
                Ok(Expr::ArrayLiteral {
                    elements: optimized_elements?,
                })
            }
            Expr::Index { object, index } => {
                Ok(Expr::Index {
                    object: Box::new(self.fold_constants(*object)?),
                    index: Box::new(self.fold_constants(*index)?),
                })
            }
            Expr::Member { object, member } => {
                Ok(Expr::Member {
                    object: Box::new(self.fold_constants(*object)?),
                    member,
                })
            }
            Expr::ObjectLiteral { pairs } => {
                let optimized_pairs: Result<Vec<(String, Expr)>> = pairs.into_iter()
                    .map(|(key, value)| Ok((key, self.fold_constants(value)?)))
                    .collect();
                
                Ok(Expr::ObjectLiteral {
                    pairs: optimized_pairs?,
                })
            }
            // Литералы и переменные не требуют оптимизации
            other => Ok(other),
        }
    }
    
    /// Объединение фильтров: filter(filter(data, x > 5), x < 10) → combined_filter(data, x > 5 && x < 10)
    fn combine_filters(&mut self, expr: Expr) -> Result<Expr> {
        match expr {
            Expr::FunctionCall { name, args } => {
                if name == "table_filter" && args.len() == 2 {
                    // Проверяем, является ли первый аргумент тоже table_filter
                    if let Expr::FunctionCall { name: inner_name, args: inner_args } = &args[0] {
                        if inner_name == "table_filter" && inner_args.len() == 2 {
                            // Объединяем условия фильтрации
                            let combined_condition = Expr::Binary {
                                left: Box::new(inner_args[1].clone()),
                                operator: BinaryOp::And,
                                right: Box::new(args[1].clone()),
                            };
                            
                            self.optimization_count += 1;
                            return Ok(Expr::FunctionCall {
                                name: "table_filter".to_string(),
                                args: vec![inner_args[0].clone(), combined_condition],
                            });
                        }
                    }
                }
                
                // Рекурсивно оптимизируем аргументы
                let optimized_args: Result<Vec<Expr>> = args.into_iter()
                    .map(|arg| self.combine_filters(arg))
                    .collect();
                
                Ok(Expr::FunctionCall {
                    name,
                    args: optimized_args?,
                })
            }
            Expr::Binary { left, operator, right } => {
                Ok(Expr::Binary {
                    left: Box::new(self.combine_filters(*left)?),
                    operator,
                    right: Box::new(self.combine_filters(*right)?),
                })
            }
            Expr::Unary { operator, operand } => {
                Ok(Expr::Unary {
                    operator,
                    operand: Box::new(self.combine_filters(*operand)?),
                })
            }
            Expr::Index { object, index } => {
                Ok(Expr::Index {
                    object: Box::new(self.combine_filters(*object)?),
                    index: Box::new(self.combine_filters(*index)?),
                })
            }
            Expr::Member { object, member } => {
                Ok(Expr::Member {
                    object: Box::new(self.combine_filters(*object)?),
                    member,
                })
            }
            Expr::ArrayLiteral { elements } => {
                let optimized_elements: Result<Vec<Expr>> = elements.into_iter()
                    .map(|elem| self.combine_filters(elem))
                    .collect();
                
                Ok(Expr::ArrayLiteral {
                    elements: optimized_elements?,
                })
            }
            Expr::ObjectLiteral { pairs } => {
                let optimized_pairs: Result<Vec<(String, Expr)>> = pairs.into_iter()
                    .map(|(key, value)| Ok((key, self.combine_filters(value)?)))
                    .collect();
                
                Ok(Expr::ObjectLiteral {
                    pairs: optimized_pairs?,
                })
            }
            // Остальные выражения не требуют оптимизации
            other => Ok(other),
        }
    }
    
    /// Удаление мертвого кода
    fn remove_dead_code(&mut self, expr: Expr) -> Result<Expr> {
        match expr {
            // Удаляем недостижимый код после return/throw
            Expr::FunctionCall { name, args } => {
                // Если это select с неиспользуемыми колонками, можем оптимизировать
                if name == "table_select" && args.len() == 2 {
                    // Здесь можно добавить логику для определения используемых колонок
                    // Пока просто рекурсивно обрабатываем аргументы
                }
                
                let optimized_args: Result<Vec<Expr>> = args.into_iter()
                    .map(|arg| self.remove_dead_code(arg))
                    .collect();
                
                Ok(Expr::FunctionCall {
                    name,
                    args: optimized_args?,
                })
            }
            Expr::Binary { left, operator, right } => {
                Ok(Expr::Binary {
                    left: Box::new(self.remove_dead_code(*left)?),
                    operator,
                    right: Box::new(self.remove_dead_code(*right)?),
                })
            }
            Expr::Unary { operator, operand } => {
                Ok(Expr::Unary {
                    operator,
                    operand: Box::new(self.remove_dead_code(*operand)?),
                })
            }
            // Остальные выражения обрабатываем рекурсивно
            other => Ok(other),
        }
    }
    
    /// Упрощение булевых выражений
    fn simplify_boolean_expressions(&mut self, expr: Expr) -> Result<Expr> {
        match expr {
            Expr::Binary { left, operator, right } => {
                let left_opt = self.simplify_boolean_expressions(*left)?;
                let right_opt = self.simplify_boolean_expressions(*right)?;
                
                // Оптимизации для булевых операторов
                match operator {
                    BinaryOp::And => {
                        // true && x → x
                        if let Expr::Literal(Value::Bool(true)) = left_opt {
                            self.optimization_count += 1;
                            return Ok(right_opt);
                        }
                        // x && true → x
                        if let Expr::Literal(Value::Bool(true)) = right_opt {
                            self.optimization_count += 1;
                            return Ok(left_opt);
                        }
                        // false && x → false
                        if let Expr::Literal(Value::Bool(false)) = left_opt {
                            self.optimization_count += 1;
                            return Ok(Expr::Literal(Value::Bool(false)));
                        }
                        // x && false → false
                        if let Expr::Literal(Value::Bool(false)) = right_opt {
                            self.optimization_count += 1;
                            return Ok(Expr::Literal(Value::Bool(false)));
                        }
                    }
                    BinaryOp::Or => {
                        // false || x → x
                        if let Expr::Literal(Value::Bool(false)) = left_opt {
                            self.optimization_count += 1;
                            return Ok(right_opt);
                        }
                        // x || false → x
                        if let Expr::Literal(Value::Bool(false)) = right_opt {
                            self.optimization_count += 1;
                            return Ok(left_opt);
                        }
                        // true || x → true
                        if let Expr::Literal(Value::Bool(true)) = left_opt {
                            self.optimization_count += 1;
                            return Ok(Expr::Literal(Value::Bool(true)));
                        }
                        // x || true → true
                        if let Expr::Literal(Value::Bool(true)) = right_opt {
                            self.optimization_count += 1;
                            return Ok(Expr::Literal(Value::Bool(true)));
                        }
                    }
                    _ => {}
                }
                
                Ok(Expr::Binary {
                    left: Box::new(left_opt),
                    operator,
                    right: Box::new(right_opt),
                })
            }
            Expr::Unary { operator, operand } => {
                let operand_opt = self.simplify_boolean_expressions(*operand)?;
                
                // Двойное отрицание: !!x → x
                if let UnaryOp::Not = operator {
                    if let Expr::Unary { operator: UnaryOp::Not, operand: inner_operand } = operand_opt {
                        self.optimization_count += 1;
                        return Ok(*inner_operand);
                    }
                }
                
                Ok(Expr::Unary {
                    operator,
                    operand: Box::new(operand_opt),
                })
            }
            // Рекурсивно обрабатываем остальные выражения
            other => Ok(other),
        }
    }
    
    /// Оптимизация вызовов функций
    fn optimize_function_calls(&mut self, expr: Expr) -> Result<Expr> {
        match expr {
            Expr::FunctionCall { name, args } => {
                // Оптимизация для table_head(table_head(data, n), m) → table_head(data, min(n, m))
                if name == "table_head" && args.len() == 2 {
                    if let Expr::FunctionCall { name: inner_name, args: inner_args } = &args[0] {
                        if inner_name == "table_head" && inner_args.len() == 2 {
                            // Берем минимальное значение из двух head операций
                            if let (Expr::Literal(Value::Number(n)), Expr::Literal(Value::Number(m))) = (&inner_args[1], &args[1]) {
                                let min_val = n.min(*m);
                                self.optimization_count += 1;
                                return Ok(Expr::FunctionCall {
                                    name: "table_head".to_string(),
                                    args: vec![inner_args[0].clone(), Expr::Literal(Value::Number(min_val))],
                                });
                            }
                        }
                    }
                }
                
                // Рекурсивно оптимизируем аргументы
                let optimized_args: Result<Vec<Expr>> = args.into_iter()
                    .map(|arg| self.optimize_function_calls(arg))
                    .collect();
                
                Ok(Expr::FunctionCall {
                    name,
                    args: optimized_args?,
                })
            }
            // Рекурсивно обрабатываем остальные выражения
            other => Ok(other),
        }
    }
    
    /// Вычислить константное бинарное выражение
    fn evaluate_constant_binary(&self, left: &Value, op: &BinaryOp, right: &Value) -> Result<Value> {
        use Value::*;
        
        match (left, op, right) {
            (Number(a), BinaryOp::Add, Number(b)) => Ok(Number(a + b)),
            (Number(a), BinaryOp::Subtract, Number(b)) => Ok(Number(a - b)),
            (Number(a), BinaryOp::Multiply, Number(b)) => Ok(Number(a * b)),
            (Number(a), BinaryOp::Divide, Number(b)) => {
                if *b == 0.0 {
                    Err(DataCodeError::runtime_error("Division by zero in constant folding", 0))
                } else {
                    Ok(Number(a / b))
                }
            }
            (Number(a), BinaryOp::Less, Number(b)) => Ok(Bool(a < b)),
            (Number(a), BinaryOp::Greater, Number(b)) => Ok(Bool(a > b)),
            (Number(a), BinaryOp::LessEqual, Number(b)) => Ok(Bool(a <= b)),
            (Number(a), BinaryOp::GreaterEqual, Number(b)) => Ok(Bool(a >= b)),
            (Number(a), BinaryOp::Equal, Number(b)) => Ok(Bool((a - b).abs() < f64::EPSILON)),
            (Number(a), BinaryOp::NotEqual, Number(b)) => Ok(Bool((a - b).abs() >= f64::EPSILON)),
            (Bool(a), BinaryOp::And, Bool(b)) => Ok(Bool(*a && *b)),
            (Bool(a), BinaryOp::Or, Bool(b)) => Ok(Bool(*a || *b)),
            (Bool(a), BinaryOp::Equal, Bool(b)) => Ok(Bool(a == b)),
            (Bool(a), BinaryOp::NotEqual, Bool(b)) => Ok(Bool(a != b)),
            (String(a), BinaryOp::Add, String(b)) => Ok(String(format!("{}{}", a, b))),
            (String(a), BinaryOp::Equal, String(b)) => Ok(Bool(a == b)),
            (String(a), BinaryOp::NotEqual, String(b)) => Ok(Bool(a != b)),
            _ => Err(DataCodeError::runtime_error("Cannot evaluate constant expression", 0)),
        }
    }
    
    /// Вычислить константное унарное выражение
    fn evaluate_constant_unary(&self, op: &UnaryOp, operand: &Value) -> Result<Value> {
        use Value::*;
        
        match (op, operand) {
            (UnaryOp::Minus, Number(n)) => Ok(Number(-n)),
            (UnaryOp::Not, Bool(b)) => Ok(Bool(!b)),
            _ => Err(DataCodeError::runtime_error("Cannot evaluate constant unary expression", 0)),
        }
    }
}

impl Default for ASTOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
