use crate::value::{Value, Table as TableStruct};
use crate::error::{DataCodeError, Result};
use std::fs;
use std::path::{PathBuf, Path};
use glob::glob;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

// Thread-local storage для SmbManager
thread_local! {
    static SMB_MANAGER: RefCell<Option<Arc<Mutex<crate::websocket::smb::SmbManager>>>> = RefCell::new(None);
}

/// Установить SmbManager для текущего потока
pub fn set_smb_manager(manager: Arc<Mutex<crate::websocket::smb::SmbManager>>) {
    SMB_MANAGER.with(|m| *m.borrow_mut() = Some(manager));
}

/// Очистить SmbManager для текущего потока
pub fn clear_smb_manager() {
    SMB_MANAGER.with(|m| *m.borrow_mut() = None);
}

/// Разрешить путь для режима --use-ve
/// Если путь относительный и включен режим use_ve, разрешает его относительно папки сессии
/// БЕЗОПАСНОСТЬ: Блокирует попытки выйти за пределы папки сессии с помощью ".."
fn resolve_path_for_use_ve(path: &Path, line: usize) -> Result<PathBuf> {
    use crate::websocket::{get_use_ve, get_user_session_path};
    use crate::error::DataCodeError;
    
    // Если режим use_ve не включен, возвращаем путь как есть
    if !get_use_ve() {
        return Ok(path.to_path_buf());
    }
    
    // Если путь абсолютный, блокируем доступ вне папки сессии
    if path.is_absolute() {
        if let Some(session_path) = get_user_session_path() {
            // Проверяем, что абсолютный путь находится внутри папки сессии
            if path.strip_prefix(&session_path).is_ok() {
                // Путь находится внутри папки сессии, разрешаем
                return Ok(path.to_path_buf());
            } else {
                // Путь вне папки сессии - блокируем
                return Err(DataCodeError::runtime_error(
                    "Доступ запрещен: путь находится вне папки сессии",
                    line
                ));
            }
        }
        // Если нет папки сессии, блокируем абсолютные пути
        return Err(DataCodeError::runtime_error(
            "Доступ запрещен: абсолютные пути не разрешены в режиме use_ve",
            line
        ));
    }
    
    // Если путь относительный и есть папка сессии, разрешаем относительно неё
    if let Some(session_path) = get_user_session_path() {
        // Проверяем, что путь не содержит компоненты ".." в начале
        // Это простейшая проверка на path traversal
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            // Более строгая проверка: блокируем если путь начинается с ".." или содержит "/../"
            let normalized = path_str.replace("\\", "/");
            if normalized.starts_with("../") || normalized.starts_with("..\\") || 
               normalized.contains("/../") || normalized.contains("\\..\\") ||
               normalized == ".." {
                return Err(DataCodeError::runtime_error(
                    "Доступ запрещен: использование '..' для выхода из папки сессии не разрешено",
                    line
                ));
            }
        }
        
        let resolved = session_path.join(path);
        
        // Дополнительная проверка безопасности: убеждаемся, что результат всё ещё внутри папки сессии
        // Нормализуем путь (canonicalize не используем, так как файл может не существовать)
        // Вместо этого просто проверяем, что resolved начинается с session_path
        if let Ok(canonical_session) = std::fs::canonicalize(&session_path) {
            if let Ok(canonical_resolved) = std::fs::canonicalize(&resolved) {
                if !canonical_resolved.starts_with(&canonical_session) {
                    return Err(DataCodeError::runtime_error(
                        "Доступ запрещен: путь выходит за пределы папки сессии",
                        line
                    ));
                }
            }
        } else {
            // Если canonicalize не работает, используем простую проверку через strip_prefix
            if resolved.strip_prefix(&session_path).is_err() {
                return Err(DataCodeError::runtime_error(
                    "Доступ запрещен: путь выходит за пределы папки сессии",
                    line
                ));
            }
        }
        
        return Ok(resolved);
    }
    
    // Если папка сессии не установлена, возвращаем ошибку для безопасности
    Err(DataCodeError::runtime_error(
        "Доступ запрещен: папка сессии не установлена",
        line
    ))
}

/// Вычислить относительный путь от папки сессии до файла
/// В режиме --use-ve возвращает путь относительно папки сессии
/// Вне режима --use-ve возвращает полный путь
fn make_relative_to_session(absolute_path: &Path, _line: usize) -> Result<PathBuf> {
    use crate::websocket::{get_use_ve, get_user_session_path};
    
    if get_use_ve() {
        if let Some(session_path) = get_user_session_path() {
            // Пытаемся вычислить относительный путь от папки сессии
            if let Ok(rel_path) = absolute_path.strip_prefix(&session_path) {
                return Ok(rel_path.to_path_buf());
            }
            // Если не получается (например, файл вне папки сессии), возвращаем полный путь
            return Ok(absolute_path.to_path_buf());
        }
    }
    
    // Вне режима use_ve или если нет папки сессии, возвращаем полный путь
    Ok(absolute_path.to_path_buf())
}

/// Парсинг lib:// пути
/// Возвращает (share_name, path) или None если это не lib:// путь
fn parse_lib_path(path_str: &str) -> Option<(String, String)> {
    if path_str.starts_with("lib://") {
        let rest = &path_str[6..]; // Убираем "lib://"
        if let Some(slash_pos) = rest.find('/') {
            let share_name = rest[..slash_pos].to_string();
            let path = rest[slash_pos + 1..].to_string();
            Some((share_name, path))
        } else {
            // Только имя шары, без пути
            Some((rest.to_string(), String::new()))
        }
    } else {
        None
    }
}

/// File operations functions
pub fn call_file_function(name: &str, args: Vec<Value>, line: usize) -> Result<Value> {
    use Value::*;

    match name {
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
            
            // Внутренняя функция для обработки list_files с Path
            fn list_files_internal(path: &std::path::Path, line: usize) -> Result<Vec<Value>> {
                let path_str = path.to_string_lossy();
                
                // Проверяем, является ли это lib:// путем
                if let Some((share_name, smb_path)) = parse_lib_path(&path_str) {
                    // Работаем с SMB
                    let files = SMB_MANAGER.with(|m| {
                        if let Some(manager) = m.borrow().as_ref() {
                            manager.lock().unwrap().list_files(&share_name, &smb_path)
                                .map_err(|e| DataCodeError::runtime_error(&e, line))
                        } else {
                            Err(DataCodeError::runtime_error(
                                &format!("SMB manager not available. Share '{}' may not be connected.", share_name),
                                line
                            ))
                        }
                    })?;
                    
                    // Для SMB возвращаем полные пути как Path
                    let mut path_objects = vec![];
                    for file_name in files {
                        // Строим полный путь: lib://share_name/smb_path/file_name
                        let full_smb_path = if smb_path.is_empty() || smb_path == "/" {
                            format!("lib://{}/{}", share_name, file_name)
                        } else {
                            let clean_path = if smb_path.ends_with('/') {
                                &smb_path[..smb_path.len()-1]
                            } else {
                                &smb_path
                            };
                            format!("lib://{}/{}/{}", share_name, clean_path, file_name)
                        };
                        path_objects.push(Value::Path(PathBuf::from(full_smb_path)));
                    }
                    return Ok(path_objects);
                }
                
                // Обычная файловая система
                // В режиме --use-ve для относительных путей используем папку сессии
                let resolved_path = resolve_path_for_use_ve(path, line)?;
                
                let entries = fs::read_dir(&resolved_path).map_err(|e|
                    DataCodeError::runtime_error(&format!("Failed to read dir: {}", e), line))?;
                let mut files = vec![];
                for entry in entries {
                    let entry = entry.map_err(|e| DataCodeError::runtime_error(&e.to_string(), line))?;
                    if let Ok(file_type) = entry.file_type() {
                        // Возвращаем и файлы, и директории
                        if file_type.is_file() || file_type.is_dir() {
                            let absolute_path = entry.path();
                            // Вычисляем относительный путь относительно папки сессии
                            let relative_path = make_relative_to_session(&absolute_path, line)?;
                            files.push(Value::Path(relative_path));
                        }
                    }
                }
                Ok(files)
            }
            
            match &args[0] {
                Path(p) => {
                    Ok(Array(list_files_internal(p, line)?))
                }
                Value::String(s) => {
                    // Поддержка строковых аргументов (для ".", пустых строк и относительных путей)
                    let path = if s.is_empty() || s == "." {
                        // Пустая строка или "." означает текущую директорию
                        PathBuf::from("")
                    } else {
                        PathBuf::from(s)
                    };
                    Ok(Array(list_files_internal(&path, line)?))
                }
                Value::PathPattern(pattern) => {
                    let pattern_str = pattern.to_string_lossy();
                    
                    // Проверяем lib:// паттерн
                    if let Some((share_name, smb_path)) = parse_lib_path(&pattern_str) {
                        // Для SMB пока не поддерживаем паттерны, только простой список
                        let files = SMB_MANAGER.with(|m| {
                            if let Some(manager) = m.borrow().as_ref() {
                                manager.lock().unwrap().list_files(&share_name, &smb_path)
                                    .map_err(|e| DataCodeError::runtime_error(&e, line))
                            } else {
                                Err(DataCodeError::runtime_error(
                                    &format!("SMB manager not available. Share '{}' may not be connected.", share_name),
                                    line
                                ))
                            }
                        })?;
                        
                        // Для SMB паттернов возвращаем полные пути как Path
                        let mut path_objects = vec![];
                        for file_name in files {
                            // Строим полный путь: lib://share_name/smb_path/file_name
                            let full_smb_path = if smb_path.is_empty() || smb_path == "/" {
                                format!("lib://{}/{}", share_name, file_name)
                            } else {
                                let clean_path = if smb_path.ends_with('/') {
                                    &smb_path[..smb_path.len()-1]
                                } else {
                                    &smb_path
                                };
                                format!("lib://{}/{}/{}", share_name, clean_path, file_name)
                            };
                            path_objects.push(Value::Path(PathBuf::from(full_smb_path)));
                        }
                        Ok(Array(path_objects))
                    } else {
                        let mut files = vec![];
                        
                        for entry in glob(&pattern_str).map_err(|e| 
                            DataCodeError::runtime_error(&format!("Invalid glob pattern: {}", e), line))? {
                            match entry {
                                Ok(path) => {
                                    // Возвращаем и файлы, и директории
                                    if path.is_file() || path.is_dir() {
                                        // Вычисляем относительный путь относительно папки сессии
                                        let relative_path = make_relative_to_session(&path, line)?;
                                        files.push(Value::Path(relative_path));
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Warning: Error reading file: {}", e);
                                }
                            }
                        }
                        Ok(Array(files))
                    }
                }
                _ => Err(DataCodeError::type_error("Path, PathPattern or String", "other", line)),
            }
        }
        
        "read_file" => {
            if args.is_empty() || args.len() > 3 {
                return Err(DataCodeError::runtime_error(
                    &format!("read_file expects 1-3 arguments, got {}", args.len()),
                    line
                ));
            }

            // Парсим аргументы
            let path = match &args[0] {
                Value::Path(p) => p,
                _ => return Err(DataCodeError::type_error("Path", "other", line)),
            };

            // Парсим опциональные аргументы
            let mut header_row: Option<usize> = None;
            let mut sheet_name: Option<std::string::String> = None;

            if args.len() == 2 {
                // Два аргумента: path и либо header_row (Number), либо sheet_name (String)
                match &args[1] {
                    Value::Number(n) => {
                        if n < &0.0 || n.fract() != 0.0 {
                            return Err(DataCodeError::runtime_error(
                                "header_row must be a non-negative integer",
                                line
                            ));
                        }
                        header_row = Some(*n as usize);
                    }
                    Value::String(s) => {
                        sheet_name = Some(s.clone());
                    }
                    _ => return Err(DataCodeError::runtime_error(
                        "Second argument must be either a number (header_row) or a string (sheet_name)",
                        line
                    )),
                }
            } else if args.len() == 3 {
                // Три аргумента: path, header_row (Number), sheet_name (String)
                match (&args[1], &args[2]) {
                    (Value::Number(n), Value::String(s)) => {
                        if n < &0.0 || n.fract() != 0.0 {
                            return Err(DataCodeError::runtime_error(
                                "header_row must be a non-negative integer",
                                line
                            ));
                        }
                        header_row = Some(*n as usize);
                        sheet_name = Some(s.clone());
                    }
                    _ => return Err(DataCodeError::runtime_error(
                        "Expected (path, header_row: Number, sheet_name: String)",
                        line
                    )),
                }
            }

            let path_str = path.to_string_lossy();
            
            // Проверяем, является ли это lib:// путем
            if let Some((share_name, smb_path)) = parse_lib_path(&path_str) {
                // Работаем с SMB
                let file_content = SMB_MANAGER.with(|m| {
                    if let Some(manager) = m.borrow().as_ref() {
                        manager.lock().unwrap().read_file(&share_name, &smb_path)
                            .map_err(|e| DataCodeError::runtime_error(&e, line))
                    } else {
                        Err(DataCodeError::runtime_error(
                            &format!("SMB manager not available. Share '{}' may not be connected.", share_name),
                            line
                        ))
                    }
                })?;
                
                // Определяем тип файла по расширению
                let ext = if let Some(dot_pos) = smb_path.rfind('.') {
                    &smb_path[dot_pos + 1..]
                } else {
                    ""
                }.to_lowercase();
                
                match ext.as_str() {
                    "txt" => {
                        let contents = std::string::String::from_utf8(file_content)
                            .map_err(|e| DataCodeError::runtime_error(&format!("Failed to decode file content: {}", e), line))?;
                        Ok(Value::String(contents))
                    }
                    "csv" => {
                        // Читаем CSV из памяти
                        read_csv_from_bytes(&file_content, header_row.unwrap_or(0), line)
                    }
                    "xlsx" => {
                        // Читаем XLSX из памяти
                        read_xlsx_from_bytes(&file_content, header_row, sheet_name.as_deref(), line)
                    }
                    _ => {
                        // Пытаемся прочитать как текст
                        match std::string::String::from_utf8(file_content) {
                            Ok(contents) => Ok(Value::String(contents)),
                            Err(_) => Err(DataCodeError::runtime_error(
                                &format!("Unsupported file extension: {}. Cannot read binary file.", ext),
                                line
                            )),
                        }
                    }
                }
            } else {
                // Обычная файловая система
                // В режиме --use-ve для относительных путей используем папку сессии
                let resolved_path = resolve_path_for_use_ve(path, line)?;
                
                let ext = resolved_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                match ext.as_str() {
                    "txt" => {
                        let contents = std::fs::read_to_string(&resolved_path)
                            .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read file: {}", e), line))?;
                        Ok(Value::String(contents))
                    }
                    "csv" => {
                        read_csv_file(&resolved_path, header_row.unwrap_or(0), line)
                    }
                    "xlsx" => {
                        read_xlsx_file(&resolved_path, header_row, sheet_name.as_deref(), line)
                    }
                    _ => Err(DataCodeError::runtime_error(&format!("Unsupported file extension: {}", ext), line)),
                }
            }
        }
        
        "analyze_csv" => {
            if args.len() != 1 {
                return Err(DataCodeError::wrong_argument_count("analyze_csv", 1, args.len(), line));
            }
            match &args[0] {
                Value::Path(p) => {
                    let resolved_path = resolve_path_for_use_ve(p, line)?;
                    analyze_csv_file(&resolved_path, line)
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
                    read_csv_safe_file(p, line)
                }
                _ => Err(DataCodeError::type_error("Path", "other", line)),
            }
        }
        
        _ => Err(DataCodeError::function_not_found(name, line)),
    }
}

/// Check if a function name belongs to file functions
pub fn is_file_function(name: &str) -> bool {
    matches!(name, "path" | "list_files" | "read_file" | "analyze_csv" | "read_csv_safe")
}

// Helper functions for file operations
fn read_csv_file(p: &std::path::Path, header_row: usize, line: usize) -> Result<Value> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false) // Мы сами управляем заголовками
        .from_path(p)
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read CSV: {}", e), line))?;

    let mut records = rdr.records().enumerate();
    
    // Пропускаем строки до header_row
    let headers: Vec<String> = if let Some((row_idx, result)) = records.next() {
        if row_idx < header_row {
            // Пропускаем строки до нужной
            let mut last_record = result;
            for _ in (row_idx + 1)..=header_row {
                if let Some((_, result)) = records.next() {
                    last_record = result;
                } else {
                    return Err(DataCodeError::runtime_error(
                        &format!("File has fewer than {} rows (header_row = {})", header_row + 1, header_row),
                        line
                    ));
                }
            }
            let record = last_record.map_err(|e| 
                DataCodeError::runtime_error(&format!("Failed to read header row: {}", e), line))?;
            record.iter().map(|h| h.trim().to_string()).collect()
        } else if row_idx == header_row {
            // Это уже нужная строка
            let record = result.map_err(|e| 
                DataCodeError::runtime_error(&format!("Failed to read header row: {}", e), line))?;
            record.iter().map(|h| h.trim().to_string()).collect()
        } else {
            return Err(DataCodeError::runtime_error(
                &format!("Internal error: row index {} != header_row {}", row_idx, header_row),
                line
            ));
        }
    } else {
        return Err(DataCodeError::runtime_error("Empty CSV file", line));
    };

    if headers.is_empty() {
        return Err(DataCodeError::runtime_error("Header row is empty", line));
    }

    let mut table = TableStruct::new(headers);
    let mut warnings = Vec::new();

    // Читаем оставшиеся строки данных
    for (row_index, result) in records {
        let record = result.map_err(|e| DataCodeError::runtime_error(&format!("Failed to read row {}: {}", row_index + 1, e), line))?;
        
        let mut row_values = Vec::new();
        for (_col_index, field) in record.iter().enumerate() {
            let value = parse_csv_value(field.trim());
            row_values.push(value);
        }
        
        if let Err(e) = table.add_row(row_values) {
            warnings.push(format!("Row {}: {}", row_index + 1, e));
        }
    }

    for warning in warnings {
        eprintln!("⚠️  {}", warning);
    }

    Ok(Value::table(table))
}

fn read_xlsx_file(p: &std::path::Path, header_row: Option<usize>, sheet_name: Option<&str>, line: usize) -> Result<Value> {
    use calamine::{Reader, open_workbook, Xlsx};
    let mut workbook: Xlsx<_> = open_workbook(p)
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to open xlsx: {}", e), line))?;
    
    // Выбираем лист по имени или по индексу
    let range = if let Some(name) = sheet_name {
        workbook.worksheet_range(name)
            .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read sheet '{}': {}", name, e), line))?
    } else {
        workbook.worksheet_range_at(0)
            .ok_or_else(|| DataCodeError::runtime_error("No sheets found", line))?
            .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read sheet: {}", e), line))?
    };

    let header_row_index = header_row.unwrap_or(0);
    let mut rows = range.rows();
    
    // Пропускаем строки до header_row
    let headers: Vec<String> = if let Some(header_row_data) = rows.nth(header_row_index) {
        header_row_data.iter().map(|cell| {
            match cell {
                calamine::Data::String(s) => s.clone(),
                calamine::Data::Float(f) => f.to_string(),
                calamine::Data::Int(i) => i.to_string(),
                _ => "Column".to_string(),
            }
        }).collect()
    } else {
        return Err(DataCodeError::runtime_error(
            &format!("File has fewer than {} rows (header_row = {})", header_row_index + 1, header_row_index),
            line
        ));
    };

    if headers.is_empty() {
        return Err(DataCodeError::runtime_error("Header row is empty", line));
    }

    let mut table = TableStruct::new(headers);
    
    for row in rows {
        let row_values: Vec<Value> = row.iter()
            .map(|cell| parse_excel_value(cell))
            .collect();
        
        if let Err(e) = table.add_row(row_values) {
            eprintln!("Warning: {}", e);
        }
    }

    Ok(Value::table(table))
}

fn analyze_csv_file(_p: &std::path::Path, _line: usize) -> Result<Value> {
    // Implementation for CSV analysis
    // This is a placeholder - you can implement detailed CSV analysis here
    Ok(Value::String("CSV analysis not yet implemented".to_string()))
}

fn read_csv_safe_file(p: &std::path::Path, line: usize) -> Result<Value> {
    // Implementation for safe CSV reading
    // This is a placeholder - you can implement safe CSV reading here
    read_csv_file(p, 0, line) // По умолчанию заголовок в первой строке
}

/// Читать CSV из байтов (для SMB файлов)
fn read_csv_from_bytes(bytes: &[u8], header_row: usize, line: usize) -> Result<Value> {
    use std::io::Cursor;
    
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false) // Мы сами управляем заголовками
        .from_reader(Cursor::new(bytes));

    let mut records = rdr.records().enumerate();
    
    // Пропускаем строки до header_row
    let headers: Vec<String> = if let Some((row_idx, result)) = records.next() {
        if row_idx < header_row {
            // Пропускаем строки до нужной
            let mut last_record = result;
            for _ in (row_idx + 1)..=header_row {
                if let Some((_, result)) = records.next() {
                    last_record = result;
                } else {
                    return Err(DataCodeError::runtime_error(
                        &format!("File has fewer than {} rows (header_row = {})", header_row + 1, header_row),
                        line
                    ));
                }
            }
            let record = last_record.map_err(|e| 
                DataCodeError::runtime_error(&format!("Failed to read header row: {}", e), line))?;
            record.iter().map(|h| h.trim().to_string()).collect()
        } else if row_idx == header_row {
            // Это уже нужная строка
            let record = result.map_err(|e| 
                DataCodeError::runtime_error(&format!("Failed to read header row: {}", e), line))?;
            record.iter().map(|h| h.trim().to_string()).collect()
        } else {
            return Err(DataCodeError::runtime_error(
                &format!("Internal error: row index {} != header_row {}", row_idx, header_row),
                line
            ));
        }
    } else {
        return Err(DataCodeError::runtime_error("Empty CSV file", line));
    };

    if headers.is_empty() {
        return Err(DataCodeError::runtime_error("Header row is empty", line));
    }

    let mut table = TableStruct::new(headers);
    let mut warnings = Vec::new();

    // Читаем оставшиеся строки данных
    for (row_index, result) in records {
        let record = result.map_err(|e| DataCodeError::runtime_error(&format!("Failed to read row {}: {}", row_index + 1, e), line))?;
        
        let mut row_values = Vec::new();
        for (_col_index, field) in record.iter().enumerate() {
            let value = parse_csv_value(field.trim());
            row_values.push(value);
        }
        
        if let Err(e) = table.add_row(row_values) {
            warnings.push(format!("Row {}: {}", row_index + 1, e));
        }
    }

    for warning in warnings {
        eprintln!("⚠️  {}", warning);
    }

    Ok(Value::table(table))
}

/// Читать XLSX из байтов (для SMB файлов)
fn read_xlsx_from_bytes(bytes: &[u8], header_row: Option<usize>, sheet_name: Option<&str>, line: usize) -> Result<Value> {
    use calamine::{Reader, open_workbook_from_rs, Xlsx};
    use std::io::Cursor;
    
    let mut workbook: Xlsx<_> = open_workbook_from_rs(Cursor::new(bytes))
        .map_err(|e| DataCodeError::runtime_error(&format!("Failed to open xlsx: {}", e), line))?;
    
    // Выбираем лист по имени или по индексу
    let range = if let Some(name) = sheet_name {
        workbook.worksheet_range(name)
            .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read sheet '{}': {}", name, e), line))?
    } else {
        workbook.worksheet_range_at(0)
            .ok_or_else(|| DataCodeError::runtime_error("No sheets found", line))?
            .map_err(|e| DataCodeError::runtime_error(&format!("Failed to read sheet: {}", e), line))?
    };

    let header_row_index = header_row.unwrap_or(0);
    let mut rows = range.rows();
    
    // Пропускаем строки до header_row
    let headers: Vec<String> = if let Some(header_row_data) = rows.nth(header_row_index) {
        header_row_data.iter().map(|cell| {
            match cell {
                calamine::Data::String(s) => s.clone(),
                calamine::Data::Float(f) => f.to_string(),
                calamine::Data::Int(i) => i.to_string(),
                _ => "Column".to_string(),
            }
        }).collect()
    } else {
        return Err(DataCodeError::runtime_error(
            &format!("File has fewer than {} rows (header_row = {})", header_row_index + 1, header_row_index),
            line
        ));
    };

    if headers.is_empty() {
        return Err(DataCodeError::runtime_error("Header row is empty", line));
    }

    let mut table = TableStruct::new(headers);
    
    for row in rows {
        let row_values: Vec<Value> = row.iter()
            .map(|cell| parse_excel_value(cell))
            .collect();
        
        if let Err(e) = table.add_row(row_values) {
            eprintln!("Warning: {}", e);
        }
    }

    Ok(Value::table(table))
}

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

fn parse_csv_value(s: &str) -> Value {
    let trimmed = s.trim();

    // Try to parse as number
    if let Ok(n) = trimmed.parse::<f64>() {
        return Value::Number(n);
    }

    // Try to parse as boolean
    match trimmed.to_lowercase().as_str() {
        "true" | "yes" | "1" => return Value::Bool(true),
        "false" | "no" | "0" => return Value::Bool(false),
        _ => {}
    }

    // Check for currency
    if trimmed.starts_with('$') || trimmed.starts_with('€') || trimmed.starts_with('£') {
        return Value::Currency(trimmed.to_string());
    }

    // Empty or null values
    if trimmed.is_empty() || trimmed.to_lowercase() == "null" || trimmed.to_lowercase() == "na" {
        return Value::Null;
    }

    // Default to string
    Value::String(trimmed.to_string())
}
