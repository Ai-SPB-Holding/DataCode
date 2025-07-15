// Логика работы с таблицами в DataCode
// Включает TableColumn и Table структуры с их методами

use std::collections::HashMap;
use super::types::{Value, DataType};

/// Колонка таблицы с информацией о типах данных
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    pub name: String,
    pub inferred_type: DataType,
    pub type_counts: HashMap<DataType, usize>,
    pub total_values: usize,
    pub warnings: Vec<String>,
}

impl TableColumn {
    /// Создать новую колонку с заданным именем
    pub fn new(name: String) -> Self {
        Self {
            name,
            inferred_type: DataType::Null,
            type_counts: HashMap::new(),
            total_values: 0,
            warnings: Vec::new(),
        }
    }

    /// Добавить значение в колонку и обновить статистику типов
    /// Phase 1 Optimization: Reduced type inference overhead
    pub fn add_value(&mut self, value: &Value) {
        let data_type = DataType::from_value(value);
        *self.type_counts.entry(data_type.clone()).or_insert(0) += 1;
        self.total_values += 1;

        // Phase 1 Optimization: Only infer type every 100 values or at the end
        // This reduces computational overhead during bulk operations
        if self.total_values % 100 == 0 || self.should_infer_immediately(&data_type) {
            self.infer_primary_type();
        }
    }

    /// Phase 1 Optimization: Determine if immediate type inference is needed
    fn should_infer_immediately(&self, new_type: &DataType) -> bool {
        // Infer immediately if this is the first value or if type changes significantly
        self.total_values == 1 ||
        (self.inferred_type != *new_type && !self.inferred_type.is_compatible_with(new_type))
    }

    /// Определить основной тип колонки на основе статистики
    fn infer_primary_type(&mut self) {
        if self.total_values == 0 {
            return;
        }

        // Специальная логика для числовых типов - объединяем Integer и Float
        let integer_count = self.type_counts.get(&DataType::Integer).unwrap_or(&0);
        let float_count = self.type_counts.get(&DataType::Float).unwrap_or(&0);
        let numeric_count = integer_count + float_count;

        // Находим тип с наибольшим количеством значений
        let mut max_count = 0;
        let mut primary_type = DataType::Null;

        // Если есть числовые данные, рассматриваем их как единый тип
        if numeric_count > 0 {
            max_count = numeric_count;
            // Если есть хотя бы одно float значение, считаем всю колонку Float
            // Это позволяет избежать предупреждений о смешанных числовых типах
            primary_type = if *float_count > 0 {
                DataType::Float
            } else {
                DataType::Integer
            };
        }

        // Проверяем остальные типы только если числовые типы не доминируют
        if numeric_count == 0 {
            for (data_type, count) in &self.type_counts {
                // Пропускаем числовые типы, так как мы их уже обработали
                if matches!(data_type, DataType::Integer | DataType::Float) {
                    continue;
                }

                if *count > max_count {
                    max_count = *count;
                    primary_type = data_type.clone();
                }
            }
        }

        self.inferred_type = primary_type;

        // Генерируем предупреждения о неоднородности
        self.warnings.clear();

        // Для числовых типов (Integer и Float) НЕ генерируем предупреждения
        // так как они считаются совместимыми числовыми типами
        if matches!(self.inferred_type, DataType::Integer | DataType::Float) {
            let non_numeric_count = self.total_values - numeric_count;

            // Предупреждение только если есть НЕ-числовые значения (String, Bool, etc.)
            if non_numeric_count > 0 {
                let percentage = (non_numeric_count as f64 / self.total_values as f64) * 100.0;
                self.warnings.push(format!(
                    "Колонка '{}' содержит неоднородные данные: {:.1}% значений не являются числовыми",
                    self.name, percentage
                ));
            }
            // НЕ генерируем предупреждения для смешанных Integer/Float - это нормально!
        } else {
            // Для не-числовых типов используем старую логику
            // НО если основной тип Integer или Float, то учитываем числовую совместимость
            if matches!(self.inferred_type, DataType::Integer | DataType::Float) {
                // Даже если основной тип Integer или Float, считаем их совместимыми
                let non_numeric_count = self.total_values - numeric_count;
                if non_numeric_count > 0 {
                    let percentage = (non_numeric_count as f64 / self.total_values as f64) * 100.0;
                    self.warnings.push(format!(
                        "Колонка '{}' содержит неоднородные данные: {:.1}% значений не являются числовыми",
                        self.name, percentage
                    ));
                }
            } else {
                // Для остальных типов проверяем общую неоднородность
                let primary_count = self.type_counts.get(&self.inferred_type).unwrap_or(&0);
                let other_count = self.total_values - primary_count;

                if other_count > 0 {
                    let percentage = (other_count as f64 / self.total_values as f64) * 100.0;
                    self.warnings.push(format!(
                        "Колонка '{}' содержит неоднородные данные: {:.1}% значений не соответствуют основному типу {}",
                        self.name, percentage, self.inferred_type.to_string()
                    ));
                }
            }
        }
    }

    /// Получить процент значений основного типа
    pub fn get_type_purity(&self) -> f64 {
        if self.total_values == 0 {
            return 100.0;
        }

        let primary_count = if matches!(self.inferred_type, DataType::Integer | DataType::Float) {
            // Для числовых типов считаем оба типа как один
            let integer_count = self.type_counts.get(&DataType::Integer).unwrap_or(&0);
            let float_count = self.type_counts.get(&DataType::Float).unwrap_or(&0);
            integer_count + float_count
        } else {
            *self.type_counts.get(&self.inferred_type).unwrap_or(&0)
        };

        (primary_count as f64 / self.total_values as f64) * 100.0
    }

    /// Проверить, является ли колонка однородной
    pub fn is_homogeneous(&self) -> bool {
        self.warnings.is_empty()
    }

    /// Phase 1 Optimization: Finalize type inference after bulk operations
    pub fn finalize_type_inference(&mut self) {
        self.infer_primary_type();
    }
}

/// Таблица данных
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<Value>>,
    pub column_names: Vec<String>,
}

impl Table {
    /// Создать новую таблицу с заданными именами колонок
    pub fn new(column_names: Vec<String>) -> Self {
        let columns = column_names.iter()
            .map(|name| TableColumn::new(name.clone()))
            .collect();

        Self {
            columns,
            rows: Vec::new(),
            column_names,
        }
    }

    /// Добавить строку в таблицу
    pub fn add_row(&mut self, row: Vec<Value>) -> Result<(), String> {
        if row.len() != self.column_names.len() {
            return Err(format!(
                "Количество значений в строке ({}) не соответствует количеству колонок ({})",
                row.len(), self.column_names.len()
            ));
        }

        // Обновляем типизацию колонок
        for (i, value) in row.iter().enumerate() {
            if let Some(column) = self.columns.get_mut(i) {
                column.add_value(value);
            }
        }

        self.rows.push(row);
        Ok(())
    }

    /// Phase 1 Optimization: Add multiple rows efficiently
    pub fn add_rows(&mut self, rows: Vec<Vec<Value>>) -> Result<(), String> {
        // Pre-allocate capacity for better performance
        self.rows.reserve(rows.len());

        for row in rows {
            if row.len() != self.column_names.len() {
                return Err(format!(
                    "Количество значений в строке ({}) не соответствует количеству колонок ({})",
                    row.len(), self.column_names.len()
                ));
            }

            // Update column type information
            for (i, value) in row.iter().enumerate() {
                if let Some(column) = self.columns.get_mut(i) {
                    column.add_value(value);
                }
            }

            self.rows.push(row);
        }

        // Finalize type inference after bulk operation
        self.finalize_type_inference();
        Ok(())
    }

    /// Phase 1 Optimization: Finalize type inference for all columns
    pub fn finalize_type_inference(&mut self) {
        for column in &mut self.columns {
            column.finalize_type_inference();
        }
    }

    /// Получить все предупреждения от всех колонок
    pub fn get_warnings(&self) -> Vec<String> {
        self.columns.iter()
            .flat_map(|col| col.warnings.iter())
            .cloned()
            .collect()
    }

    /// Получить количество строк
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Получить количество колонок
    pub fn column_count(&self) -> usize {
        self.column_names.len()
    }

    /// Получить значение по индексам строки и колонки
    pub fn get_value(&self, row: usize, col: usize) -> Option<&Value> {
        self.rows.get(row)?.get(col)
    }

    /// Получить всю строку по индексу
    pub fn get_row(&self, row: usize) -> Option<&Vec<Value>> {
        self.rows.get(row)
    }

    /// Получить колонку по имени
    pub fn get_column_by_name(&self, name: &str) -> Option<&TableColumn> {
        self.columns.iter().find(|col| col.name == name)
    }

    /// Получить индекс колонки по имени
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.column_names.iter().position(|col_name| col_name == name)
    }

    /// Получить все значения колонки по имени
    pub fn get_column_values(&self, name: &str) -> Option<Vec<&Value>> {
        let col_index = self.get_column_index(name)?;
        Some(self.rows.iter().map(|row| &row[col_index]).collect())
    }

    /// Проверить, является ли таблица пустой
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Очистить все данные таблицы
    pub fn clear(&mut self) {
        self.rows.clear();
        for column in &mut self.columns {
            column.type_counts.clear();
            column.total_values = 0;
            column.warnings.clear();
            column.inferred_type = DataType::Null;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_column_creation() {
        let column = TableColumn::new("test_col".to_string());
        assert_eq!(column.name, "test_col");
        assert_eq!(column.inferred_type, DataType::Null);
        assert_eq!(column.total_values, 0);
        assert!(column.warnings.is_empty());
    }

    #[test]
    fn test_table_column_add_value() {
        let mut column = TableColumn::new("numbers".to_string());

        column.add_value(&Value::Number(42.0));
        column.finalize_type_inference(); // Phase 1 Optimization: Finalize after adding values
        assert_eq!(column.total_values, 1);
        assert_eq!(column.inferred_type, DataType::Integer);

        column.add_value(&Value::Number(3.14));
        column.finalize_type_inference(); // Phase 1 Optimization: Finalize after adding values
        assert_eq!(column.total_values, 2);
        assert_eq!(column.inferred_type, DataType::Float);
    }

    #[test]
    fn test_table_column_numeric_compatibility() {
        let mut column = TableColumn::new("mixed_numbers".to_string());

        column.add_value(&Value::Number(42.0));  // Integer
        column.add_value(&Value::Number(3.14));  // Float
        column.add_value(&Value::Number(10.0));  // Integer
        column.finalize_type_inference(); // Phase 1 Optimization: Finalize after adding values

        // Должен быть Float (так как есть хотя бы одно float значение)
        assert_eq!(column.inferred_type, DataType::Float);
        // Не должно быть предупреждений для смешанных числовых типов
        assert!(column.warnings.is_empty());
    }

    #[test]
    fn test_table_creation() {
        let table = Table::new(vec!["col1".to_string(), "col2".to_string()]);
        assert_eq!(table.column_count(), 2);
        assert_eq!(table.row_count(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_table_add_row() {
        let mut table = Table::new(vec!["name".to_string(), "age".to_string()]);

        let row = vec![Value::String("Alice".to_string()), Value::Number(30.0)];
        assert!(table.add_row(row).is_ok());
        assert_eq!(table.row_count(), 1);
        assert!(!table.is_empty());
    }

    #[test]
    fn test_table_add_row_wrong_size() {
        let mut table = Table::new(vec!["col1".to_string(), "col2".to_string()]);

        let row = vec![Value::Number(42.0)]; // Только одно значение для двух колонок
        assert!(table.add_row(row).is_err());
    }

    #[test]
    fn test_table_get_value() {
        let mut table = Table::new(vec!["name".to_string(), "age".to_string()]);
        let row = vec![Value::String("Bob".to_string()), Value::Number(25.0)];
        table.add_row(row).unwrap();

        assert_eq!(table.get_value(0, 0), Some(&Value::String("Bob".to_string())));
        assert_eq!(table.get_value(0, 1), Some(&Value::Number(25.0)));
        assert_eq!(table.get_value(1, 0), None); // Нет второй строки
    }

    #[test]
    fn test_table_get_column_by_name() {
        let mut table = Table::new(vec!["name".to_string(), "age".to_string()]);
        let row = vec![Value::String("Charlie".to_string()), Value::Number(35.0)];
        table.add_row(row).unwrap();

        let name_column = table.get_column_by_name("name");
        assert!(name_column.is_some());
        assert_eq!(name_column.unwrap().name, "name");

        let nonexistent = table.get_column_by_name("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_table_get_column_values() {
        let mut table = Table::new(vec!["numbers".to_string()]);
        table.add_row(vec![Value::Number(1.0)]).unwrap();
        table.add_row(vec![Value::Number(2.0)]).unwrap();
        table.add_row(vec![Value::Number(3.0)]).unwrap();

        let values = table.get_column_values("numbers").unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], &Value::Number(1.0));
        assert_eq!(values[1], &Value::Number(2.0));
        assert_eq!(values[2], &Value::Number(3.0));
    }

    #[test]
    fn test_table_clear() {
        let mut table = Table::new(vec!["col1".to_string()]);
        table.add_row(vec![Value::Number(42.0)]).unwrap();
        assert!(!table.is_empty());

        table.clear();
        assert!(table.is_empty());
        assert_eq!(table.row_count(), 0);
    }

    #[test]
    fn test_column_type_purity() {
        let mut column = TableColumn::new("test".to_string());

        column.add_value(&Value::Number(1.0));
        column.add_value(&Value::Number(2.0));
        column.add_value(&Value::String("text".to_string()));

        // 2 из 3 значений числовые, должно быть ~66.7%
        let purity = column.get_type_purity();
        assert!((purity - 66.67).abs() < 0.1);
    }
}
