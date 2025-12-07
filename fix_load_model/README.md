# Тестирование загрузки данных model_data

Эта папка содержит скрипты для тестирования загрузки данных из `model_data` и создания SQLite базы данных.

## Файлы

- `load_model_data.dc` - основной скрипт загрузки всех данных
- `BUG_REPORT.md` - отчет об обнаруженных ошибках и их исправлении
- `test_basic.dc` - базовый тест загрузки справочников
- `test_file_listing.dc` - тест функции `list_files()`
- `test_file_exists.dc` - тест функции `file_exists()`
- `test_merge.dc` - тест объединения таблиц

## Использование

### Установка и тестирование

```bash
make install && cd fix_load_model && datacode load_model_data.dc --build_model
```

### Отдельные тесты

```bash
# Тест базовой загрузки
datacode test_basic.dc

# Тест функции list_files
datacode test_file_listing.dc

# Тест функции file_exists
datacode test_file_exists.dc

# Тест объединения таблиц
datacode test_merge.dc
```

## Результаты

После выполнения `load_model_data.dc --build_model` создается файл `load_model_data.db` с SQLite базой данных.

## Известные проблемы

См. `BUG_REPORT.md` для детального списка обнаруженных проблем и их статуса.

