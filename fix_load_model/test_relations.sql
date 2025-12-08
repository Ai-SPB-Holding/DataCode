-- Тест для проверки создания связей между таблицами в БД
-- Использование: sqlite3 load_model_data.db < test_relations.sql

.mode column
.headers on

.print "============================================================"
.print "🔗 Тест проверки связей между таблицами"
.print "============================================================"
.print ""

-- ============================================================
-- ТЕСТ 1: Проверка наличия таблицы метаданных о связях
-- ============================================================
.print "📋 ТЕСТ 1: Проверка наличия таблицы метаданных"
.print ""

SELECT 
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='_datacode_relations') 
        THEN '✅ Таблица _datacode_relations существует'
        ELSE '❌ Таблица _datacode_relations не найдена'
    END AS status;

.print ""

-- ============================================================
-- ТЕСТ 2: Проверка количества связей
-- ============================================================
.print "📊 ТЕСТ 2: Проверка количества связей"
.print ""

SELECT 
    COUNT(*) AS total_relations,
    CASE 
        WHEN COUNT(*) >= 10 THEN '✅ Достаточно связей'
        WHEN COUNT(*) > 0 THEN '⚠️  Мало связей'
        ELSE '❌ Связи отсутствуют'
    END AS status
FROM _datacode_relations;

.print ""
.print "Ожидаемое количество связей: 12"
.print ""

-- ============================================================
-- ТЕСТ 3: Проверка конкретных связей
-- ============================================================
.print "🔗 ТЕСТ 3: Проверка конкретных связей"
.print ""

-- Проверка связей product_catalog с sales_all
.print "Связь product_catalog.product_id ↔ sales_all.product_id:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'sales_all' AND from_column = 'product_id' AND to_table = 'product_catalog' AND to_column = 'product_id')
            OR (from_table = 'product_catalog' AND from_column = 'product_id' AND to_table = 'sales_all' AND to_column = 'product_id')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей regions с sales_all
.print ""
.print "Связь regions.region_code ↔ sales_all.region:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'sales_all' AND from_column = 'region' AND to_table = 'regions' AND to_column = 'region_code')
            OR (from_table = 'regions' AND from_column = 'region_code' AND to_table = 'sales_all' AND to_column = 'region')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей employees с sales_all
.print ""
.print "Связь employees.employee_id ↔ sales_all.employee_id:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'sales_all' AND from_column = 'employee_id' AND to_table = 'employees' AND to_column = 'employee_id')
            OR (from_table = 'employees' AND from_column = 'employee_id' AND to_table = 'sales_all' AND to_column = 'employee_id')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей product_catalog с inventory_all
.print ""
.print "Связь product_catalog.product_id ↔ inventory_all.product_id:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'inventory_all' AND from_column = 'product_id' AND to_table = 'product_catalog' AND to_column = 'product_id')
            OR (from_table = 'product_catalog' AND from_column = 'product_id' AND to_table = 'inventory_all' AND to_column = 'product_id')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей regions с inventory_all
.print ""
.print "Связь regions.region_code ↔ inventory_all.region:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'inventory_all' AND from_column = 'region' AND to_table = 'regions' AND to_column = 'region_code')
            OR (from_table = 'regions' AND from_column = 'region_code' AND to_table = 'inventory_all' AND to_column = 'region')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей sales_all с refunds_all
.print ""
.print "Связь sales_all.transaction_id ↔ refunds_all.transaction_id:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'refunds_all' AND from_column = 'transaction_id' AND to_table = 'sales_all' AND to_column = 'transaction_id')
            OR (from_table = 'sales_all' AND from_column = 'transaction_id' AND to_table = 'refunds_all' AND to_column = 'transaction_id')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей product_catalog с refunds_all
.print ""
.print "Связь product_catalog.product_id ↔ refunds_all.product_id:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'refunds_all' AND from_column = 'product_id' AND to_table = 'product_catalog' AND to_column = 'product_id')
            OR (from_table = 'product_catalog' AND from_column = 'product_id' AND to_table = 'refunds_all' AND to_column = 'product_id')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей regions с refunds_all
.print ""
.print "Связь regions.region_code ↔ refunds_all.region:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'refunds_all' AND from_column = 'region' AND to_table = 'regions' AND to_column = 'region_code')
            OR (from_table = 'regions' AND from_column = 'region_code' AND to_table = 'refunds_all' AND to_column = 'region')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей regions с marketing_spend_all
.print ""
.print "Связь regions.region_code ↔ marketing_spend_all.region:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'marketing_spend_all' AND from_column = 'region' AND to_table = 'regions' AND to_column = 'region_code')
            OR (from_table = 'regions' AND from_column = 'region_code' AND to_table = 'marketing_spend_all' AND to_column = 'region')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей product_catalog с product_summary_all
.print ""
.print "Связь product_catalog.product_id ↔ product_summary_all.product_id:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'product_summary_all' AND from_column = 'product_id' AND to_table = 'product_catalog' AND to_column = 'product_id')
            OR (from_table = 'product_catalog' AND from_column = 'product_id' AND to_table = 'product_summary_all' AND to_column = 'product_id')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей regions с regional_summary_all
.print ""
.print "Связь regions.region_code ↔ regional_summary_all.region:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'regional_summary_all' AND from_column = 'region' AND to_table = 'regions' AND to_column = 'region_code')
            OR (from_table = 'regions' AND from_column = 'region_code' AND to_table = 'regional_summary_all' AND to_column = 'region')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

-- Проверка связей employees с employee_performance_all
.print ""
.print "Связь employees.employee_id ↔ employee_performance_all.employee_id:"
SELECT 
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM _datacode_relations 
            WHERE (from_table = 'employee_performance_all' AND from_column = 'employee_id' AND to_table = 'employees' AND to_column = 'employee_id')
            OR (from_table = 'employees' AND from_column = 'employee_id' AND to_table = 'employee_performance_all' AND to_column = 'employee_id')
        ) THEN '✅'
        ELSE '❌'
    END AS status;

.print ""

-- ============================================================
-- ТЕСТ 4: Детальный список всех связей
-- ============================================================
.print "📋 ТЕСТ 4: Детальный список всех связей"
.print ""

SELECT 
    from_table AS 'Таблица с FK',
    from_column AS 'Колонка FK',
    to_table AS 'Ссылается на',
    to_column AS 'Колонка PK',
    relation_type AS 'Тип связи',
    created_at AS 'Создано'
FROM _datacode_relations
ORDER BY from_table, from_column;

.print ""

-- ============================================================
-- ТЕСТ 5: Проверка индексов на внешних ключах
-- ============================================================
.print "📊 ТЕСТ 5: Проверка индексов на внешних ключах"
.print ""

SELECT 
    name AS 'Имя индекса',
    tbl_name AS 'Таблица',
    CASE 
        WHEN sql IS NOT NULL THEN '✅'
        ELSE '⚠️'
    END AS status
FROM sqlite_master
WHERE type = 'index' 
  AND name LIKE 'idx_%'
  AND name NOT LIKE 'sqlite_%'
ORDER BY tbl_name, name;

.print ""

-- Проверяем, что для каждой связи есть индекс
.print "Проверка наличия индексов для связей:"
SELECT 
    r.from_table || '.' || r.from_column AS relation,
    CASE 
        WHEN EXISTS (
            SELECT 1 FROM sqlite_master 
            WHERE type = 'index' 
            AND name = 'idx_' || r.from_table || '_' || r.from_column
        ) THEN '✅'
        ELSE '❌'
    END AS has_index
FROM _datacode_relations r
ORDER BY r.from_table, r.from_column;

.print ""

-- ============================================================
-- ТЕСТ 6: Проверка целостности связей (данные)
-- ============================================================
.print "🔍 ТЕСТ 6: Проверка целостности данных через связи"
.print ""

-- Проверка: все product_id в sales_all существуют в product_catalog
.print "Проверка product_id в sales_all через связь:"
SELECT 
    CASE 
        WHEN NOT EXISTS (
            SELECT DISTINCT s.product_id
            FROM sales_all s
            LEFT JOIN product_catalog p ON s.product_id = p.product_id
            WHERE p.product_id IS NULL
            LIMIT 10
        ) THEN '✅ Все product_id существуют'
        ELSE '❌ Найдены несуществующие product_id'
    END AS status,
    COUNT(DISTINCT s.product_id) AS unique_product_ids_in_sales,
    (SELECT COUNT(*) FROM product_catalog) AS total_products_in_catalog
FROM sales_all s;

-- Проверка: все region в sales_all существуют в regions
.print ""
.print "Проверка region в sales_all через связь:"
SELECT 
    CASE 
        WHEN NOT EXISTS (
            SELECT DISTINCT s.region
            FROM sales_all s
            LEFT JOIN regions r ON s.region = r.region_code
            WHERE r.region_code IS NULL
            LIMIT 10
        ) THEN '✅ Все region существуют'
        ELSE '❌ Найдены несуществующие region'
    END AS status,
    COUNT(DISTINCT s.region) AS unique_regions_in_sales,
    (SELECT COUNT(*) FROM regions) AS total_regions
FROM sales_all s;

-- Проверка: все employee_id в sales_all существуют в employees
.print ""
.print "Проверка employee_id в sales_all через связь:"
SELECT 
    CASE 
        WHEN NOT EXISTS (
            SELECT DISTINCT s.employee_id
            FROM sales_all s
            LEFT JOIN employees e ON s.employee_id = e.employee_id
            WHERE e.employee_id IS NULL AND s.employee_id IS NOT NULL
            LIMIT 10
        ) THEN '✅ Все employee_id существуют'
        ELSE '❌ Найдены несуществующие employee_id'
    END AS status,
    COUNT(DISTINCT s.employee_id) AS unique_employee_ids_in_sales,
    (SELECT COUNT(*) FROM employees) AS total_employees
FROM sales_all s
WHERE s.employee_id IS NOT NULL;

.print ""

-- ============================================================
-- ИТОГОВАЯ СТАТИСТИКА
-- ============================================================
.print "============================================================"
.print "📊 Итоговая статистика по связям"
.print "============================================================"
.print ""

SELECT 
    'Всего связей в БД' AS metric,
    COUNT(*) AS value
FROM _datacode_relations
UNION ALL
SELECT 
    'Связей со справочниками',
    COUNT(*)
FROM _datacode_relations
WHERE to_table IN ('product_catalog', 'regions', 'employees')
UNION ALL
SELECT 
    'Связей между таблицами данных',
    COUNT(*)
FROM _datacode_relations
WHERE to_table NOT IN ('product_catalog', 'regions', 'employees')
UNION ALL
SELECT 
    'Индексов на внешних ключах',
    COUNT(*)
FROM sqlite_master
WHERE type = 'index' 
  AND name LIKE 'idx_%'
  AND name NOT LIKE 'sqlite_%';

.print ""
.print "✅ Тестирование связей завершено!"
.print ""

