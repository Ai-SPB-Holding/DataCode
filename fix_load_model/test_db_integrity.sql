-- Тест для проверки корректности и полноты данных в SQLite БД
-- Использование: sqlite3 load_model_data.db < test_db_integrity.sql

.mode column
.headers on

.print "============================================================"
.print "🧪 Тест корректности и полноты данных в БД"
.print "============================================================"
.print ""

-- ============================================================
-- ТЕСТ 1: Проверка наличия всех таблиц
-- ============================================================
.print "📋 ТЕСТ 1: Проверка наличия таблиц"
.print ""

-- Проверяем наличие каждой таблицы отдельно
SELECT 
    'product_catalog' AS table_name,
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='product_catalog') THEN '✅'
        ELSE '❌'
    END AS status
UNION ALL
SELECT 'regions',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='regions') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'employees',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='employees') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'sales_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='sales_all') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'inventory_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='inventory_all') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'refunds_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='refunds_all') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'marketing_spend_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='marketing_spend_all') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'financial_summary_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='financial_summary_all') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'regional_summary_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='regional_summary_all') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'product_summary_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='product_summary_all') THEN '✅'
        ELSE '❌'
    END
UNION ALL
SELECT 'employee_performance_all',
    CASE 
        WHEN EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='employee_performance_all') THEN '✅'
        ELSE '❌'
    END;

.print ""
.print "Список всех таблиц:"
SELECT name AS table_name 
FROM sqlite_master 
WHERE type = 'table' 
  AND name NOT LIKE '_%'
ORDER BY name;

.print ""

-- ============================================================
-- ТЕСТ 2: Проверка количества строк в таблицах
-- ============================================================
.print "📊 ТЕСТ 2: Проверка количества строк"
.print ""

SELECT 
    'product_catalog' AS table_name,
    COUNT(*) AS row_count,
    CASE 
        WHEN COUNT(*) = 50 THEN '✅'
        ELSE '⚠️'
    END AS status
FROM product_catalog
UNION ALL
SELECT 
    'regions',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 10 THEN '✅'
        ELSE '⚠️'
    END
FROM regions
UNION ALL
SELECT 
    'employees',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 30 THEN '✅'
        ELSE '⚠️'
    END
FROM employees
UNION ALL
SELECT 
    'sales_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) >= 10000 THEN '✅'
        ELSE '⚠️'
    END
FROM sales_all
UNION ALL
SELECT 
    'inventory_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) >= 10000 THEN '✅'
        ELSE '⚠️'
    END
FROM inventory_all
UNION ALL
SELECT 
    'refunds_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) > 0 THEN '✅'
        ELSE '⚠️'
    END
FROM refunds_all
UNION ALL
SELECT 
    'marketing_spend_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) > 0 THEN '✅'
        ELSE '⚠️'
    END
FROM marketing_spend_all
UNION ALL
SELECT 
    'financial_summary_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) > 0 THEN '✅'
        ELSE '⚠️'
    END
FROM financial_summary_all
UNION ALL
SELECT 
    'regional_summary_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) > 0 THEN '✅'
        ELSE '⚠️'
    END
FROM regional_summary_all
UNION ALL
SELECT 
    'product_summary_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) > 0 THEN '✅'
        ELSE '⚠️'
    END
FROM product_summary_all
UNION ALL
SELECT 
    'employee_performance_all',
    COUNT(*),
    CASE 
        WHEN COUNT(*) > 0 THEN '✅'
        ELSE '⚠️'
    END
FROM employee_performance_all;

.print ""

-- ============================================================
-- ТЕСТ 3: Проверка целостности данных (внешние ключи)
-- ============================================================
.print "🔗 ТЕСТ 3: Проверка целостности данных"
.print ""

-- Проверка product_id в sales_all
.print "Проверка product_id в sales_all:"
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN '✅ Все product_id существуют в product_catalog'
        ELSE '❌ Найдены несуществующие product_id: ' || COUNT(*)
    END AS status,
    COUNT(*) AS invalid_count
FROM (
    SELECT DISTINCT s.product_id
    FROM sales_all s
    LEFT JOIN product_catalog p ON s.product_id = p.product_id
    WHERE p.product_id IS NULL
    LIMIT 100
);

-- Проверка region в sales_all
.print ""
.print "Проверка region в sales_all:"
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN '✅ Все region существуют в regions'
        ELSE '❌ Найдены несуществующие region: ' || COUNT(*)
    END AS status,
    COUNT(*) AS invalid_count
FROM (
    SELECT DISTINCT s.region
    FROM sales_all s
    LEFT JOIN regions r ON s.region = r.region_code
    WHERE r.region_code IS NULL
    LIMIT 100
);

-- Проверка employee_id в sales_all
.print ""
.print "Проверка employee_id в sales_all:"
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN '✅ Все employee_id существуют в employees'
        ELSE '❌ Найдены несуществующие employee_id: ' || COUNT(*)
    END AS status,
    COUNT(*) AS invalid_count
FROM (
    SELECT DISTINCT s.employee_id
    FROM sales_all s
    LEFT JOIN employees e ON s.employee_id = e.employee_id
    WHERE e.employee_id IS NULL
    LIMIT 100
);

-- Проверка transaction_id в refunds_all
.print ""
.print "Проверка transaction_id в refunds_all:"
SELECT 
    CASE 
        WHEN COUNT(*) = 0 THEN '✅ Все transaction_id существуют в sales_all'
        ELSE '⚠️  Найдены transaction_id без соответствующих продаж: ' || COUNT(*)
    END AS status,
    COUNT(*) AS orphaned_count
FROM (
    SELECT DISTINCT rf.transaction_id
    FROM refunds_all rf
    LEFT JOIN sales_all s ON rf.transaction_id = s.transaction_id
    WHERE s.transaction_id IS NULL
    LIMIT 100
);

.print ""

-- ============================================================
-- ТЕСТ 4: Проверка отсутствия NULL в ключевых полях
-- ============================================================
.print "🔍 ТЕСТ 4: Проверка NULL значений в ключевых полях"
.print ""

SELECT 
    'sales_all.product_id' AS field,
    COUNT(*) AS null_count,
    CASE 
        WHEN COUNT(*) = 0 THEN '✅'
        ELSE '❌'
    END AS status
FROM sales_all 
WHERE product_id IS NULL
UNION ALL
SELECT 
    'sales_all.region',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 0 THEN '✅'
        ELSE '❌'
    END
FROM sales_all 
WHERE region IS NULL
UNION ALL
SELECT 
    'sales_all.employee_id',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 0 THEN '✅'
        ELSE '❌'
    END
FROM sales_all 
WHERE employee_id IS NULL
UNION ALL
SELECT 
    'sales_all.transaction_id',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 0 THEN '✅'
        ELSE '❌'
    END
FROM sales_all 
WHERE transaction_id IS NULL
UNION ALL
SELECT 
    'product_catalog.product_id',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 0 THEN '✅'
        ELSE '❌'
    END
FROM product_catalog 
WHERE product_id IS NULL
UNION ALL
SELECT 
    'regions.region_code',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 0 THEN '✅'
        ELSE '❌'
    END
FROM regions 
WHERE region_code IS NULL
UNION ALL
SELECT 
    'employees.employee_id',
    COUNT(*),
    CASE 
        WHEN COUNT(*) = 0 THEN '✅'
        ELSE '❌'
    END
FROM employees 
WHERE employee_id IS NULL;

.print ""

-- ============================================================
-- ТЕСТ 5: Проверка связей между таблицами
-- ============================================================
.print "🔗 ТЕСТ 5: Проверка связей между таблицами"
.print ""

SELECT 
    CASE 
        WHEN COUNT(*) >= 10 THEN '✅ Связи созданы (' || COUNT(*) || ')'
        ELSE '⚠️  Мало связей: ' || COUNT(*)
    END AS status,
    COUNT(*) AS relation_count
FROM _datacode_relations;

.print ""
.print "Детали связей:"
SELECT 
    from_table AS 'Таблица с FK',
    from_column AS 'Колонка FK',
    to_table AS 'Ссылается на',
    to_column AS 'Колонка PK'
FROM _datacode_relations
ORDER BY from_table, from_column;

.print ""

-- ============================================================
-- ТЕСТ 6: Проверка диапазонов данных
-- ============================================================
.print "📈 ТЕСТ 6: Проверка диапазонов данных"
.print ""

-- Проверка дат в sales_all (если есть колонка date)
.print "Проверка диапазона дат в sales_all:"
SELECT 
    CASE 
        WHEN MIN(date) >= '2023-01-01' AND MAX(date) <= '2025-12-31' THEN '✅'
        ELSE '⚠️'
    END AS status,
    MIN(date) AS min_date,
    MAX(date) AS max_date,
    COUNT(*) AS total_records
FROM sales_all
WHERE date IS NOT NULL;

.print ""

-- Проверка количества уникальных значений
.print "Количество уникальных значений:"
SELECT 
    'Уникальных product_id в sales_all' AS metric,
    COUNT(DISTINCT product_id) AS count
FROM sales_all
UNION ALL
SELECT 
    'Уникальных region в sales_all',
    COUNT(DISTINCT region)
FROM sales_all
UNION ALL
SELECT 
    'Уникальных employee_id в sales_all',
    COUNT(DISTINCT employee_id)
FROM sales_all
UNION ALL
SELECT 
    'Уникальных transaction_id в sales_all',
    COUNT(DISTINCT transaction_id)
FROM sales_all;

.print ""

-- ============================================================
-- ИТОГОВАЯ СТАТИСТИКА
-- ============================================================
.print "============================================================"
.print "📊 Итоговая статистика"
.print "============================================================"
.print ""

SELECT 
    'Всего таблиц' AS metric,
    COUNT(*) AS value
FROM sqlite_master 
WHERE type = 'table' 
  AND name NOT LIKE '_%'
  AND name NOT LIKE 'sqlite_%'
UNION ALL
SELECT 
    'Всего связей',
    COUNT(*)
FROM _datacode_relations
UNION ALL
SELECT 
    'Всего строк в sales_all',
    COUNT(*)
FROM sales_all
UNION ALL
SELECT 
    'Всего строк в inventory_all',
    COUNT(*)
FROM inventory_all
UNION ALL
SELECT 
    'Всего строк в refunds_all',
    COUNT(*)
FROM refunds_all
UNION ALL
SELECT 
    'Всего строк в marketing_spend_all',
    COUNT(*)
FROM marketing_spend_all;

.print ""
.print "✅ Тестирование завершено!"
.print ""

