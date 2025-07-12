use crate::interpreter::Interpreter;
use crate::error::DataCodeError;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::io::{self, Write};

pub struct Repl {
    interpreter: Interpreter,
    multiline_buffer: Vec<String>,
    in_multiline: bool,
    multiline_type: Option<MultilineType>,
    editor: DefaultEditor,
}

#[derive(Debug, Clone, PartialEq)]
enum MultilineType {
    ForLoop,
    IfStatement,
    Function,
}

impl Repl {
    pub fn new() -> RustylineResult<Self> {
        let mut editor = DefaultEditor::new()?;

        // Загружаем историю команд, если файл существует
        let _ = editor.load_history(".datacode_history");

        Ok(Self {
            interpreter: Interpreter::new(),
            multiline_buffer: Vec::new(),
            in_multiline: false,
            multiline_type: None,
            editor,
        })
    }

    pub fn run(&mut self) {
        println!("🧠 DataCode Interactive Interpreter");
        println!("Type 'help' for commands, 'exit' to quit");
        println!("Version 1.0 - Enhanced with improved parser and error handling");
        println!("💡 Use ↑/↓ arrows to navigate command history");
        println!();

        loop {
            let prompt = if self.in_multiline {
                match self.multiline_type {
                    Some(MultilineType::ForLoop) => "... ",
                    Some(MultilineType::IfStatement) => "if> ",
                    Some(MultilineType::Function) => "fn> ",
                    None => "... ",
                }
            } else {
                ">>> "
            };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    let line = line.trim().to_string();

                    if line.is_empty() {
                        continue;
                    }

                    // Добавляем команду в историю
                    self.editor.add_history_entry(&line).ok();

                    if !self.in_multiline && self.handle_special_commands(&line) {
                        continue;
                    }

                    self.process_line(line);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye! 👋");
                    break;
                }
                Err(err) => {
                    eprintln!("Error reading input: {}", err);
                    break;
                }
            }
        }

        // Сохраняем историю команд при выходе
        if let Err(err) = self.editor.save_history(".datacode_history") {
            eprintln!("Warning: Could not save command history: {}", err);
        }
    }

    fn handle_special_commands(&mut self, line: &str) -> bool {
        match line {
            "exit" | "quit" => {
                println!("Goodbye! 👋");
                std::process::exit(0);
            }
            "help" => {
                self.show_help();
                true
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                io::stdout().flush().unwrap();
                true
            }
            "vars" => {
                self.show_variables();
                true
            }
            "reset" => {
                self.interpreter = Interpreter::new();
                self.multiline_buffer.clear();
                self.in_multiline = false;
                self.multiline_type = None;
                println!("Interpreter reset! 🔄");
                true
            }
            _ => false,
        }
    }

    fn process_line(&mut self, line: String) {
        // Проверяем начало многострочных конструкций
        if !self.in_multiline {
            if line.trim_start().starts_with("for ") && line.trim_end().ends_with(" do") {
                self.start_multiline(line, MultilineType::ForLoop);
                return;
            } else if line.trim_start().starts_with("if ") && line.trim_end().ends_with(" then") {
                self.start_multiline(line, MultilineType::IfStatement);
                return;
            } else if (line.trim_start().starts_with("global function ") || line.trim_start().starts_with("local function ")) && line.trim_end().ends_with(" do") {
                self.start_multiline(line, MultilineType::Function);
                return;
            }
        }

        // Если мы в многострочном режиме
        if self.in_multiline {
            // Проверяем окончание многострочных конструкций
            let trimmed = line.trim();
            let should_end = match self.multiline_type {
                Some(MultilineType::ForLoop) => trimmed == "forend",
                Some(MultilineType::IfStatement) => trimmed == "endif",
                Some(MultilineType::Function) => trimmed == "endfunction",
                None => false,
            };

            if should_end {
                self.multiline_buffer.push(line);
                self.execute_multiline();
            } else {
                self.multiline_buffer.push(line);
            }
        } else {
            // Выполняем однострочную команду
            self.execute_single_line(line);
        }
    }

    fn start_multiline(&mut self, line: String, multiline_type: MultilineType) {
        self.in_multiline = true;
        self.multiline_type = Some(multiline_type);
        self.multiline_buffer.clear();
        self.multiline_buffer.push(line);
    }

    fn execute_multiline(&mut self) {
        let code = self.multiline_buffer.join("\n");
        self.execute_code(&code);
        
        // Сбрасываем многострочный режим
        self.in_multiline = false;
        self.multiline_type = None;
        self.multiline_buffer.clear();
    }

    fn execute_single_line(&mut self, line: String) {
        self.execute_code(&line);
    }

    fn execute_code(&mut self, code: &str) {
        match self.interpreter.exec(code) {
            Ok(()) => {
                // Команда выполнена успешно
                // Если это было присваивание, показываем переменную
                if code.trim().starts_with("global ") || code.trim().starts_with("local ") {
                    if let Some(var_name) = self.extract_variable_name(code) {
                        if let Some(value) = self.interpreter.get_variable(&var_name) {
                            println!("✓ {} = {:?}", var_name, value);
                        }
                    }
                }
            }
            Err(error) => {
                self.print_error(&error);
            }
        }
    }

    fn extract_variable_name(&self, code: &str) -> Option<String> {
        let code = code.trim();
        if let Some(rest) = code.strip_prefix("global ").or_else(|| code.strip_prefix("local ")) {
            if let Some(eq_pos) = rest.find('=') {
                let var_name = rest[..eq_pos].trim();
                return Some(var_name.to_string());
            }
        }
        None
    }

    fn print_error(&self, error: &DataCodeError) {
        println!("❌ {}", error);
        
        // Дополнительные подсказки для частых ошибок
        match error {
            DataCodeError::VariableError { name, .. } => {
                println!("💡 Hint: Variable '{}' is not defined. Use 'global {} = value' to define it.", name, name);
            }
            DataCodeError::FunctionError { name, .. } => {
                println!("💡 Hint: Function '{}' is not available. Type 'help' to see available functions.", name);
            }
            DataCodeError::SyntaxError { .. } => {
                println!("💡 Hint: Check your syntax. Type 'help' for examples.");
            }
            _ => {}
        }
    }

    fn show_help(&self) {
        println!("📚 DataCode Help");
        println!();
        println!("🔧 Special Commands:");
        println!("  help     - Show this help");
        println!("  exit     - Exit the interpreter");
        println!("  clear    - Clear the screen");
        println!("  vars     - Show all variables");
        println!("  reset    - Reset the interpreter");
        println!();
        println!("📝 Basic Syntax:");
        println!("  global x = 10              # Define global variable");
        println!("  local y = 'hello'          # Define local variable");
        println!("  global result = x + y      # Arithmetic operations");
        println!();
        println!("🔢 Operators:");
        println!("  +, -, *, /                 # Arithmetic");
        println!("  ==, !=, <, >, <=, >=       # Comparison");
        println!("  and, or, not               # Logical");
        println!("  ()                         # Grouping");
        println!();
        println!("🔄 For Loops:");
        println!("  for item in array do");
        println!("      print(item)");
        println!("  forend");
        println!();
        println!("🔧 User Functions:");
        println!("  global function add(a, b) do");
        println!("      return a + b");
        println!("  endfunction");
        println!("  ");
        println!("  local function greet(name) do");
        println!("      return 'Hello, ' + name + '!'");
        println!("  endfunction");
        println!();
        println!("🏗️ Built-in Functions:");
        println!("  now()                      # Current time");
        println!("  getcwd()                   # Current directory");
        println!("  path('string')             # Create path");
        println!("  list_files(path)           # List files in directory");
        println!("  read_file(path)            # Read file content");
        println!("  print(...)                 # Print values");
        println!();
        println!("📁 Path Operations:");
        println!("  global base = getcwd()");
        println!("  global full = base / 'subdir' / 'file.txt'");
        println!();
        println!("💡 Examples:");
        println!("  global x = 10");
        println!("  global y = 20");
        println!("  global sum = x + y");
        println!("  global condition = (x > 5) and (y < 30)");
        println!("  print('Result:', sum, 'Condition:', condition)");
        println!();
        println!("  # Define and call a function");
        println!("  global function multiply(a, b) do");
        println!("      return a * b");
        println!("  endfunction");
        println!("  global result = multiply(5, 3)");
    }

    fn show_variables(&self) {
        println!("📊 Current Variables:");
        if self.interpreter.variables.is_empty() {
            println!("  (no variables defined)");
        } else {
            for (name, value) in &self.interpreter.variables {
                println!("  {} = {:?}", name, value);
            }
        }
        println!();
    }
}

// Функция для запуска REPL
pub fn start_repl() {
    match Repl::new() {
        Ok(mut repl) => {
            repl.run();
        }
        Err(err) => {
            eprintln!("Failed to initialize REPL: {}", err);
            std::process::exit(1);
        }
    }
}
