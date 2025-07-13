use crate::value::{Value, Table};
use crate::error::{DataCodeError, Result};
use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use glob::glob;

pub fn call_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
        "now" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("now", 0, args.len(), line));
            }
            Ok(String(Utc::now().to_rfc3339()))
        }
        "path" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("path", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(Path(PathBuf::from(s))),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        "list_files" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("list_files", 1, args.len(), line));
            }
            match &args[0] {
                Path(p) => {
                    // Обычное чтение директории без фильтрации
                    let entries = fs::read_dir(p).map_err(|e|
                        DataCodeError::runtime_error(&format!("Failed to read dir: {}", e), line))?;
                    let mut files = vec![];
                    for entry in entries {
                        let entry = entry.map_err(|e| DataCodeError::runtime_error(&e.to_string(), line))?;
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                if let Some(name) = entry.file_name().to_str() {
                                    files.push(String(name.to_string()));
                                }
                            }
                        }
                    }
                    Ok(Array(files))
                }
                Value::PathPattern(pattern) => {
                    // Используем glob для поиска файлов по паттерну
                    let pattern_str = pattern.to_str()
                        .ok_or_else(|| DataCodeError::runtime_error("Invalid path pattern", line))?;

                    let mut files = vec![];
                    match glob(pattern_str) {
                        Ok(paths) => {
                            for entry in paths {
                                match entry {
                                    Ok(path) => {
                                        if path.is_file() {
                                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                                files.push(String(name.to_string()));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        return Err(DataCodeError::runtime_error(
                                            &format!("Glob error: {}", e), line
                                        ));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            return Err(DataCodeError::runtime_error(
                                &format!("Invalid glob pattern: {}", e), line
                            ));
                        }
                    }
                    Ok(Array(files))
                }
                _ => Err(DataCodeError::type_error("Path or PathPattern", "other", line)),
            }
        }
        "getcwd" => {
            if !args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("getcwd", 0, args.len(), line));
            }
            let cwd = std::env::current_dir()
                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to get current dir: {}", e), line))?;
            Ok(Value::Path(cwd))
        }
        "read_file" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("read_file", 1, args.len(), line));
            }

            match &args[0] {
                Value::Path(p) => {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    match ext.as_str() {
                        "txt" => {
                            let contents = std::fs::read_to_string(p)
                                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read file: {}", e), line))?;
                            Ok(Value::String(contents))
                        }
                        "csv" => {
                            let mut rdr = csv::ReaderBuilder::new()
                                .has_headers(true)
                                .from_path(p)
                                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV: {}", e), line))?;

                            // Получаем заголовки
                            let headers = rdr.headers()
                                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV headers: {}", e), line))?;
                            let column_names: Vec<std::string::String> = headers.iter().map(|s| s.to_string()).collect();

                            // Создаем таблицу
                            let mut table = crate::value::Table::new(column_names);

                            // Читаем данные и добавляем в таблицу
                            let mut skipped_rows = 0;
                            let expected_columns = table.columns.len();

                            for (row_index, result) in rdr.records().enumerate() {
                                match result {
                                    Ok(record) => {
                                        // Проверяем количество полей
                                        if record.len() != expected_columns {
                                            eprintln!("⚠️  Строка {} пропущена: ожидалось {} полей, найдено {} полей",
                                                row_index + 2, expected_columns, record.len()); // +2 потому что строки начинаются с 1 и есть заголовок
                                            skipped_rows += 1;
                                            continue;
                                        }

                                        let row_data: Vec<Value> = record.iter()
                                            .map(|s| parse_csv_value(s))
                                            .collect();

                                        if let Err(e) = table.add_row(row_data) {
                                            eprintln!("⚠️  Ошибка добавления строки {}: {}", row_index + 2, e);
                                            skipped_rows += 1;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️  Ошибка чтения строки {}: {}", row_index + 2, e);
                                        skipped_rows += 1;
                                    }
                                }
                            }

                            if skipped_rows > 0 {
                                eprintln!("⚠️  Всего пропущено строк: {}", skipped_rows);
                            }

                            // Выводим предупреждения о типизации
                            for warning in table.get_warnings() {
                                eprintln!("⚠️  {}", warning);
                            }

                            Ok(Value::Table(table))
                        }
                        "xlsx" => {
                            use calamine::{Reader, open_workbook, Xlsx};
                            let mut workbook: Xlsx<_> = open_workbook(p)
                                .map_err(|e| DataCodeError::runtime_error(&format!("Failed to open xlsx: {}", e), line))?;
                            let range = workbook.worksheet_range_at(0)
                                .ok_or_else(|| DataCodeError::runtime_error("No sheets found", line))?
                                .map_err(|e| DataCodeError::runtime_error(&format!("Sheet error: {}", e), line))?;

                            let mut rows_iter = range.rows();

                            // Получаем заголовки из первой строки
                            let column_names: Vec<std::string::String> = if let Some(header_row) = rows_iter.next() {
                                header_row.iter().enumerate().map(|(i, cell)| {
                                    let cell_str = cell.to_string();
                                    if cell_str.trim().is_empty() {
                                        format!("col_{}", i)
                                    } else {
                                        cell_str
                                    }
                                }).collect()
                            } else {
                                return Err(DataCodeError::runtime_error("Excel file is empty", line));
                            };

                            // Создаем таблицу
                            let mut table = crate::value::Table::new(column_names);

                            // Читаем остальные строки
                            for row in rows_iter {
                                let row_data: Vec<Value> = row.iter()
                                    .map(|cell| parse_excel_value(cell))
                                    .collect();

                                if let Err(e) = table.add_row(row_data) {
                                    return Err(DataCodeError::runtime_error(&e, line));
                                }
                            }

                            // Выводим предупреждения о типизации
                            for warning in table.get_warnings() {
                                eprintln!("⚠️  {}", warning);
                            }

                            Ok(Value::Table(table))
                        }
                        _ => Err(DataCodeError::runtime_error("Unsupported file extension", line)),
                    }
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }
        "print" => {
            let parts: Vec<std::string::String> = args.into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    Value::Currency(c) => c,
                    Value::Path(p) => p.display().to_string(),
                    Value::PathPattern(p) => format!("Pattern({})", p.display()),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Table(table) => {
                        format!("Table({}x{} rows/cols)", table.rows.len(), table.columns.len())
                    }
                    Value::Array(_) | Value::Object(_) => format!("{:?}", v),
                })
                .collect();
            println!("{}", parts.join(" "));
            Ok(Value::Null)
        }

        // Математические функции
        "abs" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("abs", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(n.abs())),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sqrt", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => {
                    if *n < 0.0 {
                        Err(DataCodeError::runtime_error("Cannot take square root of negative number", line))
                    } else {
                        Ok(Number(n.sqrt()))
                    }
                }
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "pow" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("pow", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Number(base), Number(exp)) => Ok(Number(base.powf(*exp))),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "min" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("min", 1, 0, line));
            }
            let mut min_val = match &args[0] {
                Number(n) => *n,
                _ => return Err(DataCodeError::type_error("Number", "other", line)),
            };
            for arg in &args[1..] {
                match arg {
                    Number(n) => {
                        if *n < min_val {
                            min_val = *n;
                        }
                    }
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            }
            Ok(Number(min_val))
        }
        "max" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("max", 1, 0, line));
            }
            let mut max_val = match &args[0] {
                Number(n) => *n,
                _ => return Err(DataCodeError::type_error("Number", "other", line)),
            };
            for arg in &args[1..] {
                match arg {
                    Number(n) => {
                        if *n > max_val {
                            max_val = *n;
                        }
                    }
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            }
            Ok(Number(max_val))
        }
        "round" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("round", 1, args.len(), line));
            }
            match &args[0] {
                Number(n) => Ok(Number(n.round())),
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }

        // Функции для работы с массивами
        "length" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("length", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                Currency(c) => Ok(Number(c.len() as f64)),
                Table(table) => Ok(Number(table.rows.len() as f64)),
                _ => Err(DataCodeError::type_error("Array, String, Currency, or Table", "other", line)),
            }
        }
        "push" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("push", 2, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(args[1].clone());
                    Ok(Array(new_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        "pop" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("pop", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    if arr.is_empty() {
                        Ok(Value::Null)
                    } else {
                        Ok(arr[arr.len() - 1].clone())
                    }
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        "sort" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sort", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut sorted_arr = arr.clone();
                    sorted_arr.sort_by(|a, b| {
                        match (a, b) {
                            (Number(x), Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            (String(x), String(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        }
                    });
                    Ok(Array(sorted_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        // Функции для работы со строками
        "split" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("split", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (String(text), String(delimiter)) => {
                    let parts: Vec<Value> = text.split(delimiter)
                        .map(|s| String(s.to_string()))
                        .collect();
                    Ok(Array(parts))
                }
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        "join" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("join", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Array(arr), String(delimiter)) => {
                    let mut strings = Vec::new();
                    for v in arr {
                        match v {
                            String(s) => strings.push(s.clone()),
                            _ => return Err(DataCodeError::type_error("Array of Strings", "other", line)),
                        }
                    }
                    Ok(String(strings.join(delimiter)))
                }
                _ => Err(DataCodeError::type_error("Array and String", "other", line)),
            }
        }
        "trim" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("trim", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(String(s.trim().to_string())),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        "upper" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("upper", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(String(s.to_uppercase())),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        "lower" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("lower", 1, args.len(), line));
            }
            match &args[0] {
                String(s) => Ok(String(s.to_lowercase())),
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }
        "contains" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("contains", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (String(text), String(substring)) => {
                    Ok(Bool(text.contains(substring)))
                }
                _ => Err(DataCodeError::type_error("String", "other", line)),
            }
        }

        // Функции агрегации данных
        "sum" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("sum", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut total = 0.0;
                    for item in arr {
                        match item {
                            Number(n) => total += n,
                            _ => return Err(DataCodeError::type_error("Array of Numbers", "other", line)),
                        }
                    }
                    Ok(Number(total))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        "average" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("average", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    if arr.is_empty() {
                        return Ok(Number(0.0));
                    }
                    let mut total = 0.0;
                    for item in arr {
                        match item {
                            Number(n) => total += n,
                            _ => return Err(DataCodeError::type_error("Array of Numbers", "other", line)),
                        }
                    }
                    Ok(Number(total / arr.len() as f64))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        "count" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("count", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                Table(table) => Ok(Number(table.rows.len() as f64)),
                _ => Err(DataCodeError::type_error("Array, String, or Table", "other", line)),
            }
        }
        "unique" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("unique", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => {
                    let mut unique_items = Vec::new();
                    for item in arr {
                        if !unique_items.iter().any(|existing| {
                            match (existing, item) {
                                (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
                                (String(a), String(b)) => a == b,
                                (Bool(a), Bool(b)) => a == b,
                                _ => false,
                            }
                        }) {
                            unique_items.push(item.clone());
                        }
                    }
                    Ok(Array(unique_items))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }
        "len" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("len", 1, args.len(), line));
            }
            match &args[0] {
                Array(arr) => Ok(Number(arr.len() as f64)),
                String(s) => Ok(Number(s.len() as f64)),
                Currency(c) => Ok(Number(c.len() as f64)),
                Table(table) => Ok(Number(table.rows.len() as f64)),
                _ => Err(DataCodeError::type_error("Array, String, Currency, or Table", "other", line)),
            }
        }
        "div" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("div", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Number(a), Number(b)) => {
                    if *b == 0.0 {
                        Err(DataCodeError::runtime_error("Division by zero", line))
                    } else {
                        Ok(Number(a / b))
                    }
                }
                _ => Err(DataCodeError::type_error("Number", "other", line)),
            }
        }
        "array" => {
            if args.len() != 0 {
                return Err(DataCodeError::wrong_argument_count("array", 0, args.len(), line));
            }
            Ok(Array(vec![]))
        }
        "append" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("append", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Array(arr), value) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(value.clone());
                    Ok(Array(new_arr))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        // Функции для работы с таблицами
        "table" => {
            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("table", 1, args.len(), line));
            }

            match &args[0] {
                // Создание таблицы из массива массивов
                Array(rows) => {
                    if rows.is_empty() {
                        return Err(DataCodeError::runtime_error("Нельзя создать таблицу из пустого массива", line));
                    }

                    // Определяем заголовки колонок
                    let column_names = if args.len() > 1 {
                        // Заголовки переданы как второй аргумент
                        match &args[1] {
                            Array(headers) => {
                                headers.iter().map(|v| match v {
                                    String(s) => s.clone(),
                                    _ => format!("{:?}", v),
                                }).collect()
                            }
                            _ => return Err(DataCodeError::type_error("Array", "other", line)),
                        }
                    } else {
                        // Автоматически генерируем заголовки
                        match rows.first() {
                            Some(Array(first_row)) => {
                                (0..first_row.len()).map(|i| format!("col_{}", i)).collect()
                            }
                            Some(Object(obj)) => {
                                obj.keys().cloned().collect()
                            }
                            _ => return Err(DataCodeError::runtime_error("Первый элемент должен быть массивом или объектом", line)),
                        }
                    };

                    let mut table = crate::value::Table::new(column_names);

                    // Добавляем строки
                    for row_value in rows {
                        let row_data = match row_value {
                            Array(row) => row.clone(),
                            Object(obj) => {
                                // Преобразуем объект в массив значений в порядке колонок
                                table.column_names.iter()
                                    .map(|col_name| obj.get(col_name).cloned().unwrap_or(Value::Null))
                                    .collect()
                            }
                            _ => return Err(DataCodeError::runtime_error("Каждая строка должна быть массивом или объектом", line)),
                        };

                        if let Err(e) = table.add_row(row_data) {
                            return Err(DataCodeError::runtime_error(&e, line));
                        }
                    }

                    // Выводим предупреждения о неоднородности данных
                    for warning in table.get_warnings() {
                        eprintln!("⚠️  {}", warning);
                    }

                    Ok(Value::Table(table))
                }
                _ => Err(DataCodeError::type_error("Array", "other", line)),
            }
        }

        "show_table" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("show_table", 1, args.len(), line));
            }
            match &args[0] {
                Value::Table(table) => {
                    print_table_formatted(table, None);
                    Ok(Value::Null)
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_info" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("table_info", 1, args.len(), line));
            }
            match &args[0] {
                Value::Table(table) => {
                    println!("📊 Информация о таблице:");
                    println!("   Строк: {}", table.rows.len());
                    println!("   Колонок: {}", table.columns.len());
                    println!();
                    println!("📋 Колонки:");
                    for column in &table.columns {
                        println!("   • {} ({:?}) - {} значений",
                            column.name,
                            column.inferred_type,
                            column.total_values
                        );

                        // Показываем распределение типов если есть смешанные данные
                        if column.type_counts.len() > 1 {
                            println!("     Распределение типов:");
                            for (data_type, count) in &column.type_counts {
                                let percentage = (*count as f64 / column.total_values as f64) * 100.0;
                                println!("       {:?}: {} ({:.1}%)", data_type, count, percentage);
                            }
                        }
                    }

                    // Показываем предупреждения
                    let warnings = table.get_warnings();
                    if !warnings.is_empty() {
                        println!();
                        println!("⚠️  Предупреждения:");
                        for warning in warnings {
                            println!("   • {}", warning);
                        }
                    }

                    Ok(Value::Null)
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_head" => {
            let n = if args.len() > 1 {
                match &args[1] {
                    Number(num) => *num as usize,
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            } else {
                5 // По умолчанию показываем первые 5 строк
            };

            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("table_head", 1, args.len(), line));
            }

            match &args[0] {
                Value::Table(table) => {
                    print_table_formatted(table, Some(n));
                    Ok(Value::Null)
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_tail" => {
            let n = if args.len() > 1 {
                match &args[1] {
                    Number(num) => *num as usize,
                    _ => return Err(DataCodeError::type_error("Number", "other", line)),
                }
            } else {
                5 // По умолчанию показываем последние 5 строк
            };

            if args.is_empty() {
                return Err(DataCodeError::wrong_argument_count("table_tail", 1, args.len(), line));
            }

            match &args[0] {
                Value::Table(table) => {
                    let start_index = if table.rows.len() > n {
                        table.rows.len() - n
                    } else {
                        0
                    };

                    // Создаем временную таблицу с последними строками
                    let mut temp_table = table.clone();
                    temp_table.rows = table.rows[start_index..].to_vec();

                    print_table_formatted(&temp_table, None);
                    Ok(Value::Null)
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_headers" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("table_headers", 1, args.len(), line));
            }

            match &args[0] {
                Value::Table(table) => {
                    let headers: Vec<Value> = table.column_names.iter()
                        .map(|name| Value::String(name.clone()))
                        .collect();
                    Ok(Value::Array(headers))
                }
                _ => Err(DataCodeError::type_error("Table", "other", line)),
            }
        }

        "table_select" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_select", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Value::Table(table), Array(column_names)) => {
                    let selected_columns: Vec<std::string::String> = column_names.iter()
                        .map(|v| match v {
                            String(s) => s.clone(),
                            _ => format!("{:?}", v),
                        })
                        .collect();

                    // Проверяем, что все колонки существуют
                    for col_name in &selected_columns {
                        if !table.column_names.contains(col_name) {
                            return Err(DataCodeError::runtime_error(
                                &format!("Колонка '{}' не найдена в таблице", col_name),
                                line
                            ));
                        }
                    }

                    // Находим индексы выбранных колонок
                    let column_indices: Vec<usize> = selected_columns.iter()
                        .map(|name| table.column_names.iter().position(|n| n == name).unwrap())
                        .collect();

                    // Создаем новую таблицу с выбранными колонками
                    let mut new_table = crate::value::Table::new(selected_columns);

                    for row in &table.rows {
                        let new_row: Vec<Value> = column_indices.iter()
                            .map(|&i| row.get(i).cloned().unwrap_or(Value::Null))
                            .collect();

                        if let Err(e) = new_table.add_row(new_row) {
                            return Err(DataCodeError::runtime_error(&e, line));
                        }
                    }

                    Ok(Value::Table(new_table))
                }
                _ => Err(DataCodeError::type_error("Table and Array", "other", line)),
            }
        }

        "analyze_csv" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("analyze_csv", 1, args.len(), line));
            }
            match &args[0] {
                Value::Path(p) => {
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(true)
                        .flexible(true) // Позволяет строкам иметь разное количество полей
                        .from_path(p)
                        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV: {}", e), line))?;

                    // Получаем заголовки
                    let headers = rdr.headers()
                        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV headers: {}", e), line))?;
                    let expected_columns = headers.len();

                    let mut total_rows = 0;
                    let mut invalid_rows = 0;
                    let mut field_counts = std::collections::HashMap::new();

                    for (row_index, result) in rdr.records().enumerate() {
                        total_rows += 1;
                        match result {
                            Ok(record) => {
                                let field_count = record.len();
                                *field_counts.entry(field_count).or_insert(0) += 1;

                                if field_count != expected_columns {
                                    invalid_rows += 1;
                                }
                            }
                            Err(_) => {
                                invalid_rows += 1;
                            }
                        }
                    }

                    println!("📊 Анализ CSV файла: {}", p.display());
                    println!("   Ожидаемое количество колонок: {}", expected_columns);
                    println!("   Всего строк данных: {}", total_rows);
                    println!("   Строк с ошибками: {}", invalid_rows);
                    println!("   Распределение по количеству полей:");

                    let mut sorted_counts: Vec<_> = field_counts.iter().collect();
                    sorted_counts.sort_by_key(|&(k, _)| k);

                    for (field_count, count) in sorted_counts {
                        let percentage = (*count as f64 / total_rows as f64) * 100.0;
                        println!("     {} полей: {} строк ({:.1}%)", field_count, count, percentage);
                    }

                    Ok(Value::Null)
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }

        "read_csv_safe" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("read_csv_safe", 1, args.len(), line));
            }
            match &args[0] {
                Value::Path(p) => {
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(true)
                        .flexible(true) // Позволяет строкам иметь разное количество полей
                        .from_path(p)
                        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV: {}", e), line))?;

                    // Получаем заголовки
                    let headers = rdr.headers()
                        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV headers: {}", e), line))?;
                    let column_names: Vec<std::string::String> = headers.iter().map(|s| s.to_string()).collect();
                    let expected_columns = column_names.len();

                    // Создаем таблицу
                    let mut table = crate::value::Table::new(column_names);

                    // Читаем данные и добавляем в таблицу
                    let mut skipped_rows = 0;
                    let mut total_rows = 0;

                    for (row_index, result) in rdr.records().enumerate() {
                        total_rows += 1;
                        match result {
                            Ok(record) => {
                                // Если количество полей не совпадает, дополняем или обрезаем
                                let mut row_data: Vec<Value> = record.iter()
                                    .take(expected_columns) // Берем только нужное количество полей
                                    .map(|s| parse_csv_value(s))
                                    .collect();

                                // Дополняем недостающие поля значениями Null
                                while row_data.len() < expected_columns {
                                    row_data.push(Value::Null);
                                }

                                if let Err(e) = table.add_row(row_data) {
                                    eprintln!("⚠️  Ошибка добавления строки {}: {}", row_index + 2, e);
                                    skipped_rows += 1;
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️  Ошибка чтения строки {}: {}", row_index + 2, e);
                                skipped_rows += 1;
                            }
                        }
                    }

                    if skipped_rows > 0 {
                        eprintln!("⚠️  Всего пропущено строк: {} из {}", skipped_rows, total_rows);
                    }

                    // Выводим предупреждения о типизации
                    for warning in table.get_warnings() {
                        eprintln!("⚠️  {}", warning);
                    }

                    Ok(Value::Table(table))
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }

        "table_sort" => {
            if args.len() < 2 {
                return Err(DataCodeError::wrong_argument_count("table_sort", 2, args.len(), line));
            }
            match (&args[0], &args[1]) {
                (Value::Table(table), String(column_name)) => {
                    // Находим индекс колонки для сортировки
                    let col_index = table.column_names.iter()
                        .position(|name| name == column_name)
                        .ok_or_else(|| DataCodeError::runtime_error(
                            &format!("Колонка '{}' не найдена в таблице", column_name),
                            line
                        ))?;

                    // Определяем порядок сортировки (по умолчанию по возрастанию)
                    let ascending = if args.len() > 2 {
                        match &args[2] {
                            Bool(b) => *b,
                            String(s) => s.to_lowercase() == "asc" || s.to_lowercase() == "ascending",
                            _ => true,
                        }
                    } else {
                        true
                    };

                    let mut sorted_table = table.clone();

                    // Сортируем строки
                    sorted_table.rows.sort_by(|a, b| {
                        let val_a = a.get(col_index).unwrap_or(&Value::Null);
                        let val_b = b.get(col_index).unwrap_or(&Value::Null);

                        let cmp = compare_values(val_a, val_b);
                        if ascending { cmp } else { cmp.reverse() }
                    });

                    Ok(Value::Table(sorted_table))
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        // ========== ФУНКЦИИ ФИЛЬТРАЦИИ ДАННЫХ ==========

        // Фильтрация таблицы по условию
        "table_filter" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_filter", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Value::Table(table), String(condition)) => {
                    filter_table_by_condition(table, condition, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        // SQL-подобные запросы WHERE
        "table_where" => {
            if args.len() != 4 {
                return Err(DataCodeError::wrong_argument_count("table_where", 4, args.len(), line));
            }

            match (&args[0], &args[1], &args[2], &args[3]) {
                (Value::Table(table), String(column), String(operator), value) => {
                    filter_table_where(table, column, operator, value, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, String, Value", "other", line)),
            }
        }

        // Сложные запросы с парсингом выражений
        "table_query" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_query", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Value::Table(table), String(query)) => {
                    filter_table_by_query(table, query, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        // Уникальные значения в колонке
        "table_distinct" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_distinct", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Value::Table(table), String(column)) => {
                    get_distinct_values(table, column, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        // Случайная выборка строк
        "table_sample" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_sample", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Value::Table(table), Number(n)) => {
                    sample_table_rows(table, *n as usize, line)
                }
                _ => Err(DataCodeError::type_error("Table and Number", "other", line)),
            }
        }

        // Фильтрация по диапазону значений
        "table_between" => {
            if args.len() != 4 {
                return Err(DataCodeError::wrong_argument_count("table_between", 4, args.len(), line));
            }

            match (&args[0], &args[1], &args[2], &args[3]) {
                (Value::Table(table), String(column), min_val, max_val) => {
                    filter_table_between(table, column, min_val, max_val, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, Value, Value", "other", line)),
            }
        }

        // Фильтрация по списку значений (IN)
        "table_in" => {
            if args.len() != 3 {
                return Err(DataCodeError::wrong_argument_count("table_in", 3, args.len(), line));
            }

            match (&args[0], &args[1], &args[2]) {
                (Value::Table(table), String(column), Array(values)) => {
                    filter_table_in(table, column, values, line)
                }
                _ => Err(DataCodeError::type_error("Table, String, Array", "other", line)),
            }
        }

        // Фильтрация по null значениям
        "table_is_null" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_is_null", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Value::Table(table), String(column)) => {
                    filter_table_null(table, column, true, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        // Фильтрация по не-null значениям
        "table_not_null" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("table_not_null", 2, args.len(), line));
            }

            match (&args[0], &args[1]) {
                (Value::Table(table), String(column)) => {
                    filter_table_null(table, column, false, line)
                }
                _ => Err(DataCodeError::type_error("Table and String", "other", line)),
            }
        }

        // Функция для проверки типов данных
        "isinstance" => {
            if args.len() != 2 {
                return Err(DataCodeError::wrong_argument_count("isinstance", 2, args.len(), line));
            }

            let value = &args[0];
            let type_name = match &args[1] {
                String(s) => s.as_str(),
                _ => return Err(DataCodeError::type_error("String", "other", line)),
            };

            let is_instance = match type_name.to_lowercase().as_str() {
                "number" | "integer" | "int" | "float" => {
                    matches!(value, Number(_))
                }
                "string" | "str" => {
                    matches!(value, String(_))
                }
                "bool" | "boolean" => {
                    matches!(value, Bool(_))
                }
                "array" | "list" => {
                    matches!(value, Array(_))
                }
                "object" | "dict" | "map" => {
                    matches!(value, Object(_))
                }
                "table" => {
                    matches!(value, Table(_))
                }
                "currency" | "money" => {
                    matches!(value, Currency(_))
                }
                "null" | "none" => {
                    matches!(value, Null)
                }
                "path" => {
                    matches!(value, Path(_))
                }
                "pathpattern" | "pattern" => {
                    matches!(value, PathPattern(_))
                }
                _ => {
                    return Err(DataCodeError::runtime_error(
                        &format!("Unknown type name: '{}'. Valid types: number, string, bool, array, object, table, currency, null, path, pathpattern", type_name),
                        line
                    ));
                }
            };

            Ok(Bool(is_instance))
        }

        // Функция для создания перечисления с индексами
        "enum" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("enum", 1, args.len(), line));
            }

            match &args[0] {
                Array(arr) => {
                    let mut result = Vec::new();
                    for (index, value) in arr.iter().enumerate() {
                        // Создаем массив [индекс, значение]
                        let pair = Array(vec![
                            Number(index as f64),
                            value.clone()
                        ]);
                        result.push(pair);
                    }
                    Ok(Array(result))
                }
                String(s) => {
                    let mut result = Vec::new();
                    for (index, ch) in s.chars().enumerate() {
                        // Создаем массив [индекс, символ]
                        let pair = Array(vec![
                            Number(index as f64),
                            String(ch.to_string())
                        ]);
                        result.push(pair);
                    }
                    Ok(Array(result))
                }
                Table(table) => {
                    let mut result = Vec::new();
                    for (index, row) in table.rows.iter().enumerate() {
                        // Создаем массив [индекс, строка_таблицы]
                        let pair = Array(vec![
                            Number(index as f64),
                            Array(row.clone())
                        ]);
                        result.push(pair);
                    }
                    Ok(Array(result))
                }
                _ => Err(DataCodeError::type_error("Array, String, or Table", "other", line)),
            }
        }

        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

// Вспомогательная функция для красивого вывода таблиц
fn print_table_formatted(table: &Table, limit: Option<usize>) {
    if table.rows.is_empty() {
        println!("📋 Таблица пуста");
        return;
    }

    let rows_to_show = if let Some(n) = limit {
        std::cmp::min(n, table.rows.len())
    } else {
        table.rows.len()
    };

    // Вычисляем максимальную ширину для каждой колонки
    let mut col_widths: Vec<usize> = table.column_names.iter()
        .map(|name| name.len())
        .collect();

    for (_i, row) in table.rows.iter().take(rows_to_show).enumerate() {
        for (j, value) in row.iter().enumerate() {
            if j < col_widths.len() {
                let value_str = format_value_for_table(value);
                col_widths[j] = std::cmp::max(col_widths[j], value_str.len());
            }
        }
    }

    // Печатаем заголовок
    print!("┌");
    for (i, width) in col_widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < col_widths.len() - 1 {
            print!("┬");
        }
    }
    println!("┐");

    // Печатаем названия колонок
    print!("│");
    for (i, (name, width)) in table.column_names.iter().zip(&col_widths).enumerate() {
        print!(" {:width$} ", name, width = width);
        if i < col_widths.len() - 1 {
            print!("│");
        }
    }
    println!("│");

    // Печатаем разделитель
    print!("├");
    for (i, width) in col_widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < col_widths.len() - 1 {
            print!("┼");
        }
    }
    println!("┤");

    // Печатаем строки данных
    for row in table.rows.iter().take(rows_to_show) {
        print!("│");
        for (i, (value, width)) in row.iter().zip(&col_widths).enumerate() {
            let value_str = format_value_for_table(value);
            print!(" {:width$} ", value_str, width = width);
            if i < col_widths.len() - 1 {
                print!("│");
            }
        }
        println!("│");
    }

    // Печатаем нижнюю границу
    print!("└");
    for (i, width) in col_widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2));
        if i < col_widths.len() - 1 {
            print!("┴");
        }
    }
    println!("┘");

    // Показываем информацию о количестве строк
    if let Some(n) = limit {
        if table.rows.len() > n {
            println!("... показано {} из {} строк", n, table.rows.len());
        }
    }
}

fn format_value_for_table(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{:.2}", n)
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Currency(c) => c.clone(),
        Value::Null => "null".to_string(),
        Value::Path(p) => p.display().to_string(),
        Value::PathPattern(p) => format!("Pattern({})", p.display()),
        Value::Array(_) => "[Array]".to_string(),
        Value::Object(_) => "{Object}".to_string(),
        Value::Table(_) => "[Table]".to_string(),
    }
}

// Функция для сравнения значений при сортировке
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    use Value::*;

    match (a, b) {
        (Number(a), Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (String(a), String(b)) => a.cmp(b),
        (Bool(a), Bool(b)) => a.cmp(b),
        (Null, Null) => Ordering::Equal,
        (Null, _) => Ordering::Less,
        (_, Null) => Ordering::Greater,
        // Для разных типов сравниваем их строковые представления
        _ => format_value_for_table(a).cmp(&format_value_for_table(b)),
    }
}

// Оптимизированная функция для парсинга значений из CSV
fn parse_csv_value(s: &str) -> Value {
    let trimmed = s.trim();

    // Быстрая проверка пустых значений
    if trimmed.is_empty() {
        return Value::Null;
    }

    // Быстрая проверка специальных значений без создания lowercase строки
    if trimmed.len() <= 5 {
        match trimmed {
            "null" | "NULL" | "Null" | "na" | "NA" | "Na" => return Value::Null,
            "true" | "TRUE" | "True" | "yes" | "YES" | "Yes" => return Value::Bool(true),
            "false" | "FALSE" | "False" | "no" | "NO" | "No" => return Value::Bool(false),
            _ => {}
        }
    }

    // Быстрая проверка чисел - проверяем первый символ
    let first_char = trimmed.chars().next().unwrap();
    if first_char.is_ascii_digit() || first_char == '-' || first_char == '+' {
        // Сначала пытаемся парсить как число (целое)
        if let Ok(int_val) = trimmed.parse::<i64>() {
            return Value::Number(int_val as f64);
        }

        // Затем как число с плавающей точкой
        if let Ok(float_val) = trimmed.parse::<f64>() {
            return Value::Number(float_val);
        }
    }

    // Быстрая проверка валют - проверяем наличие валютных символов или цифр
    if trimmed.len() <= 50 && (
        trimmed.chars().any(|c| matches!(c, '$' | '€' | '₽' | '£' | '¥' | '₹' | '₩' | '₪')) ||
        (trimmed.chars().any(|c| c.is_ascii_digit()) &&
         trimmed.chars().any(|c| c.is_ascii_alphabetic()))
    ) {
        if crate::value::is_currency_string(trimmed) {
            return Value::Currency(trimmed.to_string());
        }
    }

    // По умолчанию - строка
    Value::String(trimmed.to_string())
}

// Функция для парсинга значений из Excel
fn parse_excel_value(cell: &calamine::Data) -> Value {
    match cell {
        calamine::Data::Empty => Value::Null,
        calamine::Data::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Value::Null
            } else {
                Value::String(trimmed.to_string())
            }
        }
        calamine::Data::Float(f) => Value::Number(*f),
        calamine::Data::Int(i) => Value::Number(*i as f64),
        calamine::Data::Bool(b) => Value::Bool(*b),
        calamine::Data::DateTime(dt) => Value::String(dt.to_string()),
        calamine::Data::DateTimeIso(dt) => Value::String(dt.clone()),
        calamine::Data::DurationIso(dur) => Value::String(dur.clone()),
        calamine::Data::Error(e) => Value::String(format!("ERROR: {:?}", e)),
    }
}

// ========== РЕАЛИЗАЦИЯ ФУНКЦИЙ ФИЛЬТРАЦИИ ==========

// Фильтрация таблицы по условию с парсингом выражения
fn filter_table_by_condition(table: &Table, condition: &str, line: usize) -> Result<Value> {
    let mut filtered_table = crate::value::Table::new(table.column_names.clone());

    for row in &table.rows {
        // Создаем контекст переменных для текущей строки
        let mut row_context = std::collections::HashMap::new();
        for (i, col_name) in table.column_names.iter().enumerate() {
            if let Some(value) = row.get(i) {
                row_context.insert(col_name.clone(), value.clone());
            }
        }



        // Парсим выражение
        let mut parser = crate::parser::Parser::new(condition);
        let expr = match parser.parse_expression() {
            Ok(expr) => expr,
            Err(e) => return Err(DataCodeError::runtime_error(&format!("Ошибка парсинга условия: {}", e), line)),
        };

        // Создаем evaluator с контекстом строки
        let evaluator = crate::evaluator::Evaluator::new(&row_context, line);

        // Вычисляем условие
        match evaluator.evaluate(&expr) {
            Ok(Value::Bool(true)) => {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
            Ok(Value::Bool(false)) => {
                // Строка не прошла фильтр
            }
            Ok(_) => {
                return Err(DataCodeError::runtime_error("Условие фильтрации должно возвращать boolean", line));
            }
            Err(e) => {
                return Err(DataCodeError::runtime_error(&format!("Ошибка в условии фильтрации: {}", e), line));
            }
        }
    }

    Ok(Value::Table(filtered_table))
}

// SQL-подобная фильтрация WHERE
fn filter_table_where(table: &Table, column: &str, operator: &str, value: &Value, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;

    let mut filtered_table = crate::value::Table::new(table.column_names.clone());

    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            let matches = match operator {
                "=" | "==" => values_equal(row_value, value),
                "!=" | "<>" => !values_equal(row_value, value),
                "<" => compare_values_for_filter(row_value, value) == std::cmp::Ordering::Less,
                ">" => compare_values_for_filter(row_value, value) == std::cmp::Ordering::Greater,
                "<=" => {
                    let cmp = compare_values_for_filter(row_value, value);
                    cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal
                }
                ">=" => {
                    let cmp = compare_values_for_filter(row_value, value);
                    cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal
                }
                "LIKE" => match (row_value, value) {
                    (Value::String(s1), Value::String(s2)) => {
                        // Простая реализация LIKE с поддержкой % и _
                        let pattern = s2.replace('%', ".*").replace('_', ".");
                        match regex::Regex::new(&pattern) {
                            Ok(re) => re.is_match(s1),
                            Err(_) => false,
                        }
                    }
                    _ => false,
                },
                "IN" => match value {
                    Value::Array(arr) => arr.iter().any(|v| values_equal(row_value, v)),
                    _ => false,
                },
                _ => return Err(DataCodeError::runtime_error(
                    &format!("Неподдерживаемый оператор: {}", operator),
                    line
                )),
            };

            if matches {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }

    Ok(Value::Table(filtered_table))
}

// Сложные запросы с парсингом выражений (алиас для filter_table_by_condition)
fn filter_table_by_query(table: &Table, query: &str, line: usize) -> Result<Value> {
    filter_table_by_condition(table, query, line)
}

// Получение уникальных значений в колонке
fn get_distinct_values(table: &Table, column: &str, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;

    let mut unique_values = std::collections::HashSet::new();
    let mut result_values = Vec::new();

    for row in &table.rows {
        if let Some(value) = row.get(col_index) {
            let value_str = format_value_for_table(value);
            if unique_values.insert(value_str) {
                result_values.push(value.clone());
            }
        }
    }

    Ok(Value::Array(result_values))
}

// Случайная выборка строк из таблицы
fn sample_table_rows(table: &Table, n: usize, line: usize) -> Result<Value> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    if n >= table.rows.len() {
        // Если запрашиваем больше строк чем есть, возвращаем всю таблицу
        return Ok(Value::Table(table.clone()));
    }

    let mut rng = thread_rng();
    let sampled_indices: Vec<usize> = (0..table.rows.len()).collect::<Vec<_>>()
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect();

    let mut sampled_table = crate::value::Table::new(table.column_names.clone());

    for &index in &sampled_indices {
        if let Some(row) = table.rows.get(index) {
            if let Err(e) = sampled_table.add_row(row.clone()) {
                return Err(DataCodeError::runtime_error(&e, line));
            }
        }
    }

    Ok(Value::Table(sampled_table))
}

// Фильтрация по диапазону значений (BETWEEN)
fn filter_table_between(table: &Table, column: &str, min_val: &Value, max_val: &Value, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;

    let mut filtered_table = crate::value::Table::new(table.column_names.clone());

    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            let cmp_min = compare_values_for_filter(row_value, min_val);
            let cmp_max = compare_values_for_filter(row_value, max_val);

            // Значение должно быть >= min_val и <= max_val
            if (cmp_min == std::cmp::Ordering::Greater || cmp_min == std::cmp::Ordering::Equal) &&
               (cmp_max == std::cmp::Ordering::Less || cmp_max == std::cmp::Ordering::Equal) {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }

    Ok(Value::Table(filtered_table))
}

// Фильтрация по списку значений (IN)
fn filter_table_in(table: &Table, column: &str, values: &[Value], line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;

    let mut filtered_table = crate::value::Table::new(table.column_names.clone());

    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            if values.iter().any(|v| values_equal(row_value, v)) {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }

    Ok(Value::Table(filtered_table))
}

// Фильтрация по null/not null значениям
fn filter_table_null(table: &Table, column: &str, is_null: bool, line: usize) -> Result<Value> {
    // Находим индекс колонки
    let col_index = table.column_names.iter()
        .position(|name| name == column)
        .ok_or_else(|| DataCodeError::runtime_error(
            &format!("Колонка '{}' не найдена в таблице", column),
            line
        ))?;

    let mut filtered_table = crate::value::Table::new(table.column_names.clone());

    for row in &table.rows {
        if let Some(row_value) = row.get(col_index) {
            let is_value_null = matches!(row_value, Value::Null);
            if is_value_null == is_null {
                if let Err(e) = filtered_table.add_row(row.clone()) {
                    return Err(DataCodeError::runtime_error(&e, line));
                }
            }
        }
    }

    Ok(Value::Table(filtered_table))
}

// ========== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ==========

// Функция для сравнения значений при фильтрации
fn compare_values_for_filter(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    use Value::*;

    match (a, b) {
        (Number(a), Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (String(a), String(b)) => a.cmp(b),
        (Currency(a), Currency(b)) => a.cmp(b),
        (Bool(a), Bool(b)) => a.cmp(b),
        (Null, Null) => Ordering::Equal,
        (Null, _) => Ordering::Less,
        (_, Null) => Ordering::Greater,
        // Для разных типов сравниваем их строковые представления
        _ => format_value_for_table(a).cmp(&format_value_for_table(b)),
    }
}

// Функция для проверки равенства значений
fn values_equal(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Number(a), Number(b)) => (a - b).abs() < f64::EPSILON,
        (String(a), String(b)) => a == b,
        (Currency(a), Currency(b)) => a == b,
        (Bool(a), Bool(b)) => a == b,
        (Null, Null) => true,
        (Array(a), Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        _ => false,
    }
}