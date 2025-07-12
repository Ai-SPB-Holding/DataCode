// backend/DataCode/src/main.rs
mod value;
mod builtins;
mod interpreter;
mod error;
mod parser;
mod evaluator;
mod repl;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
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
                println!("Unknown argument: {}", args[1]);
                show_help();
            }
        }
    } else {
        // По умолчанию запускаем REPL
        repl::start_repl();
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
            Ok(result) => {
                match result {
                    Some(value) => {
                        if let Some(var_name) = extract_variable_name(code) {
                            println!("   ✓ {} = {:?}", var_name, value);
                        } else {
                            println!("   ✓ Result: {:?}", value);
                        }
                    }
                    None => {
                        println!("   ✓ Executed successfully");
                    }
                }
            }
            Err(e) => println!("   ❌ Error: {}", e),
        }
    }

    println!("\n🔄 For loop example:");
    interp.exec("global numbers = [1, 2, 3]").ok(); // Это пока не работает, но покажем структуру

    let for_loop = "for i in [1, 2, 3] do
    print('Number:', i)
forend";

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

fn show_help() {
    println!("🧠 DataCode - Interactive Programming Language");
    println!();
    println!("Usage:");
    println!("  cargo run                 # Start interactive REPL (default)");
    println!("  cargo run -- --repl       # Start interactive REPL");
    println!("  cargo run -- --demo       # Run demonstration");
    println!("  cargo run -- --help       # Show this help");
    println!();
    println!("Features:");
    println!("  • Interactive REPL with multiline support");
    println!("  • Arithmetic and logical operations");
    println!("  • File system operations");
    println!("  • For loops");
    println!("  • Improved error messages");
    println!("  • Path manipulation");
    println!();
    println!("Example session:");
    println!("  >>> global x = 10");
    println!("  ✓ x = Number(10.0)");
    println!("  >>> global result = x * 2 + 5");
    println!("  ✓ result = Number(25.0)");
    println!("  >>> print('Result is:', result)");
    println!("  Result is: Number(25.0)");
}
