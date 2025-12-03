// backend/DataCode/src/main.rs
mod value;
mod builtins;
mod interpreter;
mod error;
mod parser;
mod evaluator;
mod repl;
mod websocket;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Сначала проверяем наличие команды websocket (она должна обрабатываться отдельно)
        if args.iter().any(|a| a == "--websocket" || a == "--ws" || a == "websocket" || a == "ws") {
            let (host, port) = parse_websocket_args(&args);
            start_websocket_server(host, port);
            return;
        }

        // Проверяем наличие флага --debug или --verbose
        let debug_mode = args.contains(&"--debug".to_string()) || args.contains(&"--verbose".to_string());

        // Находим файл .dc или команду (исключая флаги)
        let mut file_or_command = None;
        for arg in &args[1..] {
            if !arg.starts_with("--") {
                file_or_command = Some(arg);
                break;
            }
        }

        if let Some(arg) = file_or_command {
            // Проверяем, является ли аргумент файлом .dc
            if arg.ends_with(".dc") {
                run_file(arg, debug_mode);
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
                        start_websocket_server(host, port);
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

fn run_file(file_path: &str, debug_mode: bool) {
    use interpreter::Interpreter;

    println!("🧠 DataCode File Executor");
    println!("========================");
    println!("📁 Executing file: {}", file_path);
    if debug_mode {
        println!("🔍 Debug mode: ON");
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
        Ok(content) => content,
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

fn start_websocket_server(host: String, port: u16) {
    let address = format!("{}:{}", host, port);
    
    println!("🚀 Запуск WebSocket сервера DataCode...");
    println!("📡 Адрес: ws://{}", address);
    println!("💡 Используйте --host и --port для изменения адреса");
    println!("💡 Или переменную окружения DATACODE_WS_ADDRESS");
    println!();
    
    // Создаем tokio runtime для асинхронного выполнения
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    if let Err(e) = rt.block_on(websocket::start_server(&address)) {
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
    println!("WebSocket Server:");
    println!("  • Start server: datacode --websocket");
    println!("  • Default address: ws://127.0.0.1:8080");
    println!("  • Custom host/port: datacode --websocket --host 0.0.0.0 --port 8899");
    println!("  • Or use env var: DATACODE_WS_ADDRESS=0.0.0.0:3000 datacode --websocket");
    println!("  • Send JSON: {{\"code\": \"print('Hello World')\"}}");
    println!("  • Receive JSON: {{\"success\": true, \"output\": \"Hello World\\n\", \"error\": null}}");
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
