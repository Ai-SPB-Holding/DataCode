// backend/DataCode/src/main.rs
mod value;
mod builtins;
mod interpreter;
mod error;
mod parser;
mod evaluator;
mod repl;
mod websocket;
mod cache;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Сначала проверяем наличие команды websocket (она должна обрабатываться отдельно)
        if args.iter().any(|a| a == "--websocket" || a == "--ws" || a == "websocket" || a == "ws") {
            let (host, port) = parse_websocket_args(&args);
            let use_ve = args.contains(&"--use-ve".to_string());
            start_websocket_server(host, port, use_ve);
            return;
        }

        // Проверяем наличие флага --debug или --verbose
        let debug_mode = args.contains(&"--debug".to_string()) || args.contains(&"--verbose".to_string());
        
        // Проверяем наличие флага --build_model
        let build_model = args.contains(&"--build_model".to_string());
        
        // Определяем имя выходного файла для SQLite (если указан --build_model)
        let mut sqlite_output = None;
        if build_model {
            // Ищем аргумент после --build_model
            for i in 0..args.len() {
                if args[i] == "--build_model" && i + 1 < args.len() && !args[i + 1].starts_with("--") && !args[i + 1].ends_with(".dc") {
                    sqlite_output = Some(args[i + 1].clone());
                    break;
                }
            }
            // Если не указан, проверяем переменную окружения
            if sqlite_output.is_none() {
                if let Ok(env_path) = std::env::var("DATACODE_SQLITE_OUTPUT") {
                    sqlite_output = Some(env_path);
                }
            }
        }

        // Находим файл .dc или команду (исключая флаги и аргументы после --build_model)
        let mut file_or_command = None;
        for arg in &args[1..] {
            if !arg.starts_with("--") && arg != sqlite_output.as_ref().unwrap_or(&String::new()) {
                file_or_command = Some(arg);
                break;
            }
        }

        if let Some(arg) = file_or_command {
            // Проверяем, является ли аргумент файлом .dc
            if arg.ends_with(".dc") {
                run_file(arg, debug_mode, build_model, sqlite_output);
            } else {
                match arg.as_str() {
                    "repl" | "-i" => {
                        repl::start_repl();
                    }
                    "demo" => {
                        run_demo();
                    }
                    "websocket" | "ws" => {
                        let (host, port) = parse_websocket_args(&args);
                        let use_ve = args.contains(&"--use-ve".to_string());
                        start_websocket_server(host, port, use_ve);
                    }
                    "help" | "-h" => {
                        show_help();
                    }
                    _ => {
                        println!("❌ Unknown argument: {}", arg);
                        println!("💡 Tip: Use .dc extension for DataCode files");
                        show_help();
                    }
                }
            }
        } else {
            // Проверяем флаги без файла
            let first_arg = &args[1];
            match first_arg.as_str() {
                "--repl" | "-i" => {
                    repl::start_repl();
                }
                "--demo" => {
                    run_demo();
                }
                "--help" | "-h" => {
                    show_help();
                }
                _ => {
                    println!("❌ Unknown argument: {}", first_arg);
                    println!("💡 Tip: Use .dc extension for DataCode files");
                    show_help();
                }
            }
        }
    } else {
        // По умолчанию запускаем REPL
        repl::start_repl();
    }
}

fn run_file(file_path: &str, debug_mode: bool, build_model: bool, sqlite_output: Option<String>) {
    use interpreter::Interpreter;

    println!("🧠 DataCode File Executor");
    println!("========================");
    println!("📁 Executing file: {}", file_path);
    if debug_mode {
        println!("🔍 Debug mode: ON");
    }
    if build_model {
        println!("🗄️  SQLite export: ON");
    }
    println!();

    // Проверяем существование файла
    if !Path::new(file_path).exists() {
        println!("❌ Error: File '{}' not found", file_path);
        println!("💡 Make sure the file exists and the path is correct");
        std::process::exit(1);
    }

    // Читаем содержимое файла
    let content = match fs::read_to_string(file_path) {
        Ok(content) => {
            // Debug: проверяем размер файла
            if debug_mode {
                println!("🔍 File size: {} bytes", content.len());
            }
            content
        },
        Err(e) => {
            println!("❌ Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };

    // Проверяем, что файл не пустой
    if content.trim().is_empty() {
        println!("⚠️  Warning: File '{}' is empty", file_path);
        println!("✅ Execution completed (nothing to execute)");
        return;
    }

    // Удаляем BOM (Byte Order Mark) если присутствует
    let content = if content.starts_with('\u{feff}') {
        if debug_mode {
            println!("🔍 Removing UTF-8 BOM");
        }
        content.trim_start_matches('\u{feff}').to_string()
    } else {
        content
    };

    // Создаем интерпретатор
    let mut interpreter = Interpreter::new();

    // Выполняем код
    println!("🚀 Starting execution...");
    println!();

    match interpreter.exec(&content) {
        Ok(()) => {
            println!();
            println!("✅ Execution completed successfully!");

            // Показываем финальные переменные если они есть
            let vars = interpreter.get_all_variables();
            if !vars.is_empty() && debug_mode {
                println!();
                println!("📊 Final Variables:");
                for (name, value) in vars {
                    println!("  {} = {:?}", name, value);
                }
            }

            // Экспортируем в SQLite если указан флаг --build_model
            if build_model {
                println!();
                println!("🗄️  Exporting to SQLite...");
                
                // Очищаем локальные области видимости перед экспортом
                // Это гарантирует, что в экспорт попадут только глобальные переменные
                while interpreter.variable_manager.loop_depth() > 0 {
                    interpreter.exit_loop_scope();
                }
                while interpreter.variable_manager.function_depth() > 0 {
                    interpreter.exit_function_scope();
                }
                
                // Определяем имя выходного файла
                let output_path = sqlite_output.unwrap_or_else(|| {
                    // По умолчанию: {имя_скрипта}.db
                    let file_stem = Path::new(file_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("output");
                    format!("{}.db", file_stem)
                });

                match crate::builtins::sqlite_export::export_tables_to_sqlite(&interpreter, &output_path) {
                    Ok(()) => {
                        println!("✅ SQLite database created successfully: {}", output_path);
                    }
                    Err(e) => {
                        println!("❌ Failed to export to SQLite: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            println!();
            println!("❌ Execution failed with error:");
            println!("   {}", e);
            std::process::exit(1);
        }
    }
}

fn run_demo() {
    use interpreter::Interpreter;

    println!("🧠 DataCode Demo");
    println!("================");

    let mut interp = Interpreter::new();

    // Пример кода DataCode:
    let examples = vec![
        ("Setting up variables", "global x = 10"),
        ("String variable", "global name = 'DataCode'"),
        ("Arithmetic", "global result = x * 2 + 5"),
        ("String concatenation", "global greeting = 'Hello, ' + name + '!'"),
        ("Comparison", "global is_big = x > 5"),
        ("Logical operation", "global condition = is_big and (result < 100)"),
        ("Current directory", "global cwd = getcwd()"),
        ("Current time", "global time = now()"),
    ];

    for (description, code) in examples {
        println!("\n📝 {}: {}", description, code);
        match interp.exec(code) {
            Ok(()) => {
                if let Some(var_name) = extract_variable_name(code) {
                    if let Some(value) = interp.get_variable(&var_name) {
                        println!("   ✓ {} = {:?}", var_name, value);
                    }
                } else {
                    println!("   ✓ Executed successfully");
                }
            }
            Err(e) => println!("   ❌ Error: {}", e),
        }
    }

    println!("\n🔄 For loop example:");
    interp.exec("global numbers = [1, 2, 3]").ok(); // Это пока не работает, но покажем структуру

    let for_loop = "for i in [1, 2, 3] do
    print('Number:', i)
next i";

    println!("Code:\n{}", for_loop);
    // match interp.exec(for_loop) {
    //     Ok(_) => println!("✓ Loop executed successfully"),
    //     Err(e) => println!("❌ Error: {}", e),
    // }

    println!("\n🚀 To start interactive mode, run: cargo run --repl");
}

fn extract_variable_name(code: &str) -> Option<String> {
    let code = code.trim();
    if let Some(rest) = code.strip_prefix("global ").or_else(|| code.strip_prefix("local ")) {
        if let Some(eq_pos) = rest.find('=') {
            let var_name = rest[..eq_pos].trim();
            return Some(var_name.to_string());
        }
    }
    None
}

/// Парсить аргументы командной строки для WebSocket сервера
fn parse_websocket_args(args: &[String]) -> (String, u16) {
    use std::env;
    
    let mut host = None;
    let mut port = None;
    
    // Сначала проверяем переменные окружения
    if let Ok(addr) = env::var("DATACODE_WS_ADDRESS") {
        if let Some((h, p)) = parse_address(&addr) {
            host = Some(h);
            port = Some(p);
        }
    }
    
    // Затем парсим аргументы командной строки (они имеют приоритет)
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                if i + 1 < args.len() {
                    host = Some(args[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = Some(p);
                    } else {
                        eprintln!("⚠️  Неверный порт: {}, используем значение по умолчанию", args[i + 1]);
                    }
                    i += 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    // Используем значения по умолчанию, если не указаны
    let final_host = host.unwrap_or_else(|| "127.0.0.1".to_string());
    let final_port = port.unwrap_or(8080);
    
    (final_host, final_port)
}

/// Парсить адрес в формате "host:port"
fn parse_address(addr: &str) -> Option<(String, u16)> {
    if let Some(colon_pos) = addr.rfind(':') {
        let h = addr[..colon_pos].to_string();
        if let Ok(p) = addr[colon_pos + 1..].parse::<u16>() {
            return Some((h, p));
        }
    }
    None
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
    
    if let Err(e) = rt.block_on(websocket::start_server(&address, use_ve)) {
        eprintln!("❌ Ошибка запуска WebSocket сервера: {}", e);
        std::process::exit(1);
    }
}

fn show_help() {
    println!("🧠 DataCode - Interactive Programming Language");
    println!();
    println!("Usage:");
    println!("  datacode                   # Start interactive REPL (default)");
    println!("  datacode main.dc           # Execute DataCode file");
    println!("  datacode main.dc --debug   # Execute with debug info (shows variable types)");
    println!("  datacode main.dc --build_model  # Execute and export tables to SQLite");
    println!("  datacode main.dc --build_model output.db  # Export to specific file");
    println!("  datacode --repl            # Start interactive REPL");
    println!("  datacode --demo            # Run demonstration");
    println!("  datacode --websocket       # Start WebSocket server for remote code execution");
    println!("  datacode --help            # Show this help");
    println!();
    println!("File Execution:");
    println!("  • Create files with .dc extension");
    println!("  • Write DataCode programs in files");
    println!("  • Execute with: datacode filename.dc");
    println!("  • Use --debug flag to see detailed variable information");
    println!();
    println!("Debug Mode:");
    println!("  • Shows final variables with full type information");
    println!("  • Example: departments = Array([String(\"Engineering\"), String(\"Marketing\")])");
    println!("  • Useful for development and debugging");
    println!("  • Flags: --debug or --verbose");
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
    println!("  global function greet(name) do");
    println!("      return 'Hello, ' + name + '!'");
    println!("  endfunction");
    println!("  ");
    println!("  global message = greet('DataCode')");
    println!("  print(message)");
    println!();
    println!("Run with: datacode example.dc");
    println!("Debug run: datacode example.dc --debug");
}
