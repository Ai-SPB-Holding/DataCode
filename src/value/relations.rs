// Реестр связей между колонками таблиц для модели данных

use std::rc::Rc;
use std::cell::RefCell;
use super::table::Table;
use super::types::DataType;

/// Информация о связи между двумя колонками таблиц
#[derive(Clone, Debug)]
pub struct Relation {
    pub table1: Rc<RefCell<Table>>,
    pub column1: String,
    pub table2: Rc<RefCell<Table>>,
    pub column2: String,
    pub relation_type: DataType, // Тип связи (String, Integer, Float, etc.)
}

impl Relation {
    pub fn new(
        table1: Rc<RefCell<Table>>,
        column1: String,
        table2: Rc<RefCell<Table>>,
        column2: String,
        relation_type: DataType,
    ) -> Self {
        Self {
            table1,
            column1,
            table2,
            column2,
            relation_type,
        }
    }
}

/// Глобальный реестр связей между колонками таблиц
pub struct RelationRegistry {
    relations: Vec<Relation>,
}

impl RelationRegistry {
    pub fn new() -> Self {
        Self {
            relations: Vec::new(),
        }
    }

    /// Добавить связь между двумя колонками
    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    /// Получить все связи
    pub fn get_relations(&self) -> &[Relation] {
        &self.relations
    }

    /// Получить связи для конкретной таблицы и колонки
    pub fn get_relations_for_column(
        &self,
        table: &Rc<RefCell<Table>>,
        column: &str,
    ) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|rel| {
                // Проверяем, является ли эта связь связанной с указанной таблицей и колонкой
                let table1_ptr = rel.table1.as_ptr();
                let table2_ptr = rel.table2.as_ptr();
                let table_ptr = table.as_ptr();
                
                (table1_ptr == table_ptr && rel.column1 == column) ||
                (table2_ptr == table_ptr && rel.column2 == column)
            })
            .collect()
    }

    /// Очистить все связи
    pub fn clear(&mut self) {
        self.relations.clear();
    }
}

// Глобальный реестр связей (используем RefCell вместо Mutex, так как Rc<RefCell<Table>> не является Send)
// Используем RefCell, так как интерпретатор работает в одном потоке
// Используем unsafe статическую переменную, так как RefCell не является Sync
static mut GLOBAL_RELATION_REGISTRY: Option<RefCell<RelationRegistry>> = None;

fn get_registry() -> &'static RefCell<RelationRegistry> {
    unsafe {
        if GLOBAL_RELATION_REGISTRY.is_none() {
            GLOBAL_RELATION_REGISTRY = Some(RefCell::new(RelationRegistry::new()));
        }
        GLOBAL_RELATION_REGISTRY.as_ref().unwrap()
    }
}

/// Добавить связь в глобальный реестр
pub fn add_relation(relation: Relation) {
    get_registry().borrow_mut().add_relation(relation);
}

/// Получить все связи
pub fn get_all_relations() -> Vec<Relation> {
    get_registry().borrow().get_relations().to_vec()
}

/// Получить связи для конкретной таблицы и колонки
pub fn get_relations_for_column(
    table: &Rc<RefCell<Table>>,
    column: &str,
) -> Vec<Relation> {
    get_registry().borrow().get_relations_for_column(table, column)
        .into_iter()
        .cloned()
        .collect()
}

/// Очистить все связи
pub fn clear_relations() {
    get_registry().borrow_mut().clear();
}

