// Main entry point для DataCode интерпретатора

use data_code::{run, run_with_vm};
use data_code::sqlite_export;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");


fn print_help() {
    println!("🧠 DataCode - Interactive Programming Language");
    println!();
    println!("Usage:");
    println!("  datacode                   # Start interactive REPL (default)");
    println!("  datacode main.dc           # Execute DataCode file");
    println!("  datacode main.dc --build_model  # Execute and export tables to SQLite");
    println!("  datacode main.dc --build_model output.db  # Export to specific file");
    println!("  datacode --websocket       # Start WebSocket server for remote code execution");
    println!("  datacode --help            # Show this help");
    println!();
    println!("File Execution:");
    println!("  • Create files with .dc extension");
    println!("  • Write DataCode programs in files");
    println!("  • Execute with: datacode filename.dc");

    println!();
    println!("SQLite Export (--build_model):");
    println!("  • Exports all tables from global variables to SQLite database");
    println!("  • Automatically detects foreign key relationships");
    println!("  • Creates metadata table _datacode_variables with all variable info");
    println!("  • Default output: <script_name>.db");
    println!("  • Custom output: --build_model output.db");
    println!("  • Environment variable: DATACODE_SQLITE_OUTPUT=path.db");
    println!();
    println!("WebSocket Server:");
    println!("  • Start server: datacode --websocket");
    println!("  • Default address: ws://127.0.0.1:8080");
    println!("  • Custom host/port: datacode --websocket --host 0.0.0.0 --port 8899");
    println!("  • Or use env var: DATACODE_WS_ADDRESS=0.0.0.0:3000 datacode --websocket");
    println!("  • Virtual environment mode: datacode --websocket --use-ve");
    println!("    - Creates isolated session folders in src/temp_sessions");
    println!("    - getcwd() returns empty string");
    println!("    - Supports file uploads via upload_file request");
    println!("    - Session folder is deleted on disconnect");
    println!("  • Send JSON: {{\"code\": \"print('Hello World')\"}}");
    println!("  • Receive JSON: {{\"success\": true, \"output\": \"Hello World\\n\", \"error\": null}}");
    println!("  • Upload file: {{\"type\": \"upload_file\", \"filename\": \"test.txt\", \"content\": \"...\"}}");
    println!();
    println!("Features:");
    println!("  • Interactive REPL with multiline support");
    println!("  • User-defined functions with local scope");
    println!("  • Arithmetic and logical operations");
    println!("  • File system operations");
    println!("  • For loops and control structures");
    println!("  • Improved error messages with line numbers");
    println!("  • Path manipulation");
    println!("  • Functional programming methods (map, filter, reduce)");
    println!("  • WebSocket server for remote code execution");
    println!();
    println!("Example DataCode file (example.dc):");
    println!("  # Simple DataCode program");
    println!("  fn greet(name) {{");
    println!("      return 'Hello, ' + name + '!'");
    println!("  }}");
    println!("  ");
    println!("  global message = greet('DataCode')");
    println!("  print(message)");
    println!();
    println!("Run with: datacode example.dc");
    println!("Debug run: datacode example.dc --debug");
}


fn print_version() {
    println!("DataCode v{}", VERSION);
}

fn start_websocket_server(host: String, port: u16, use_ve: bool) {
    let address = format!("{}:{}", host, port);
    
    println!("🚀 Запуск WebSocket сервера DataCode...");
    println!("📡 Адрес: ws://{}", address);
    if use_ve {
        println!("📁 Режим виртуальной среды: включен (--use-ve)");
    }
    println!("💡 Используйте --host и --port для изменения адреса");
    println!("💡 Или переменную окружения DATACODE_WS_ADDRESS");
    println!();
    
    // Создаем tokio runtime для асинхронного выполнения
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    if let Err(e) = rt.block_on(data_code::websocket::start_server(&address, use_ve)) {
        eprintln!("❌ Ошибка запуска WebSocket сервера: {}", e);
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Обработка аргументов командной строки
    if args.len() > 1 {
        let arg = &args[1];
        
        // Проверка на опции
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                print_version();
                return;
            }
            "--websocket" => {
                // Парсим аргументы для WebSocket сервера
                let mut host = "127.0.0.1".to_string();
                let mut port = 8080u16;
                let mut use_ve = false;
                
                // Проверяем переменную окружения
                if let Ok(ws_address) = env::var("DATACODE_WS_ADDRESS") {
                    if let Some(colon_pos) = ws_address.find(':') {
                        host = ws_address[..colon_pos].to_string();
                        if let Ok(p) = ws_address[colon_pos + 1..].parse::<u16>() {
                            port = p;
                        }
                    } else {
                        host = ws_address;
                    }
                }
                
                // Парсим аргументы командной строки
                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--host" => {
                            if i + 1 < args.len() {
                                host = args[i + 1].clone();
                                i += 2;
                            } else {
                                eprintln!("Ошибка: --host требует значение");
                                std::process::exit(1);
                            }
                        }
                        "--port" => {
                            if i + 1 < args.len() {
                                if let Ok(p) = args[i + 1].parse::<u16>() {
                                    port = p;
                                    i += 2;
                                } else {
                                    eprintln!("Ошибка: неверный номер порта");
                                    std::process::exit(1);
                                }
                            } else {
                                eprintln!("Ошибка: --port требует значение");
                                std::process::exit(1);
                            }
                        }
                        "--use-ve" => {
                            use_ve = true;
                            i += 1;
                        }
                        _ => {
                            eprintln!("Неизвестный аргумент: {}", args[i]);
                            std::process::exit(1);
                        }
                    }
                }
                
                start_websocket_server(host, port, use_ve);
                return;
            }
            _ => {
                // Проверка, что это не опция (начинается с -)
                if arg.starts_with('-') {
                    eprintln!("Неизвестная опция: {}", arg);
                    eprintln!("Используйте --help для справки");
                    std::process::exit(1);
                }
            }
        }
        
        // Запуск файла
        let filename = arg;
        
        // Проверка существования файла
        if !Path::new(filename).exists() {
            eprintln!("Ошибка: файл '{}' не найден", filename);
            std::process::exit(1);
        }
        
        // Проверка расширения файла (опционально, но полезно)
        if !filename.ends_with(".dc") {
            eprintln!("Предупреждение: файл '{}' не имеет расширения .dc", filename);
        }
        
        // Проверяем наличие флага --build_model
        let mut build_model = false;
        let mut output_db: Option<String> = None;
        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--build_model" | "--build-model" => {
                    build_model = true;
                    // Проверяем следующий аргумент - может быть имя файла
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        output_db = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        // Определяем имя выходного файла для SQLite
        if build_model {
            let db_filename = if let Some(db) = output_db {
                db
            } else if let Ok(env_db) = env::var("DATACODE_SQLITE_OUTPUT") {
                env_db
            } else {
                // По умолчанию: имя скрипта с расширением .db
                let path = PathBuf::from(filename);
                let stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                format!("{}.db", stem)
            };
            
            // Чтение и выполнение файла с экспортом
            match fs::read_to_string(filename) {
                Ok(source) => {
                    match run_with_vm(&source) {
                        Ok((_, vm)) => {
                            // Экспортируем таблицы в SQLite
                            match sqlite_export::export_to_sqlite(&vm, &db_filename) {
                                Ok(_) => {
                                    println!("✅ База данных создана: {}", db_filename);
                                }
                                Err(e) => {
                                    eprintln!("❌ Ошибка экспорта в SQLite: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Ошибка выполнения: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка чтения файла '{}': {}", filename, e);
                    std::process::exit(1);
                }
            }
        } else {
            // Обычное выполнение без экспорта
            match fs::read_to_string(filename) {
                Ok(source) => {
                    match run(&source) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Ошибка выполнения: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка чтения файла '{}': {}", filename, e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        // REPL режим (интерактивный)
        println!("ДатаКод v{} - Bytecode VM", VERSION);
        println!("Введите код (Ctrl+D или 'exit' для выхода):");
        println!();
        
        let mut input = String::new();
        loop {
            use std::io::{self, Write};
            
            // Показываем приглашение
            print!("datacode> ");
            io::stdout().flush().unwrap();
            
            match io::stdin().read_line(&mut input) {
                Ok(0) => {
                    // EOF (Ctrl+D)
                    println!("\nДо свидания!");
                    break;
                }
                Ok(_) => {
                    let trimmed = input.trim();
                    
                    // Проверка на команду выхода
                    if trimmed == "exit" || trimmed == "quit" {
                        println!("До свидания!");
                        break;
                    }
                    
                    if trimmed.is_empty() {
                        input.clear();
                        continue;
                    }
                    
                    // Выполнение кода
                    match run(trimmed) {
                        Ok(value) => {
                            // Если есть результат, показываем его
                            if !matches!(value, data_code::Value::Null) {
                                println!("=> {:?}", value);
                            }
                        }
                        Err(e) => {
                            eprintln!("Ошибка: {}", e);
                        }
                    }
                    input.clear();
                }
                Err(e) => {
                    eprintln!("Ошибка чтения: {}", e);
                    break;
                }
            }
        }
    }
}
