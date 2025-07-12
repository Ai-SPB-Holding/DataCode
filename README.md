# 🧠 DataCode - Interactive Programming Language

**DataCode** is a simple, interactive programming language designed for fast data processing and easy learning. It features an intuitive syntax, built-in functions, and support for user-defined functions with local scope.

## 🚀 Features

- **Interactive REPL** with multiline support and command history
- **File execution** - write programs in `.dc` files
- **User-defined functions** with local scope and parameters
- **Arithmetic and logical operations**
- **String manipulation** and concatenation
- **For loops** for iteration
- **Built-in functions** for common operations
- **Path manipulation** for file system operations
- **Improved error messages** with line numbers
- **Comment support** with `#`

## 📦 Installation

### Option 1: Global Installation (Recommended)
Install DataCode as a global command:

```bash
# Clone and install
git clone https://github.com/igornet0/DataCode.git
cd DataCode

# Install globally
make install
# or
./install.sh

# Now you can use datacode from anywhere!
datacode --help
```

### Option 2: Development Mode
Run directly with Cargo:

```bash
git clone https://github.com/igornet0/DataCode.git
cd DataCode
cargo run
```

## 🎯 Usage

### After Global Installation
```bash
datacode                   # Start interactive REPL
datacode filename.dc       # Execute DataCode file
datacode --repl            # Start interactive REPL
datacode --demo            # Run demonstration
datacode --help            # Show help
```

### Development Mode
```bash
cargo run                  # Start interactive REPL
cargo run filename.dc      # Execute DataCode file
cargo run -- --help       # Show help

# Or use Makefile
make run                   # Start REPL
make examples              # Run all examples
make test                  # Run tests
```

### Quick Examples
```bash
# Create a DataCode file
echo 'print("Hello, DataCode!")' > hello.dc

# Execute the file
datacode hello.dc          # (after global installation)
# or
cargo run hello.dc         # (development mode)
```

### Программное использование
```rust
use data_code::interpreter::Interpreter;

fn main() {
    let mut interp = Interpreter::new();
    interp.exec("global basePath = getcwd()").unwrap();
    interp.exec("global files = list_files(basePath / 'data')").unwrap();
}
```
---

## 📄 Синтаксис языка

### 🔹 Переменные
```DataCode
global path = getcwd()
local subdir = 'data'
```
• `global` — сохраняет переменную глобально
• `local` — ограничена текущим контекстом (например, циклом)

### 🔹 Арифметические операции
```DataCode
global x = 10
global y = 20
global sum = x + y          # Сложение
global diff = x - y         # Вычитание
global prod = x * y         # Умножение
global quot = x / y         # Деление
global complex = (x + y) * 2 - 5  # Сложные выражения
```

### 🔹 Операторы сравнения
```DataCode
global eq = x == y          # Равенство
global ne = x != y          # Неравенство
global gt = x > y           # Больше
global lt = x < y           # Меньше
global ge = x >= y          # Больше или равно
global le = x <= y          # Меньше или равно
```

### 🔹 Логические операции
```DataCode
global flag1 = true
global flag2 = false
global and_result = flag1 and flag2    # Логическое И
global or_result = flag1 or flag2      # Логическое ИЛИ
global not_result = not flag1          # Логическое НЕ
global complex_logic = (x > 5) and (y < 30) or flag1
```

### 🔹 Конкатенация путей
```DataCode
global dir = basePath / 'data' / 'images'
```
• `/` используется для Path + String (контекстно определяется)

### 🔹 Сложение строк
```DataCode
global name = 'image' + '001.jpg'
global greeting = 'Hello, ' + name + '!'
```
• `+` объединяет строки

---

## 🔁 Циклы
```DataCode
for file in files do
    local path = basePath / 'data' / file
    local text = read_file(path)
    print('>>', file, 'length:', text)
forend
```
- for x in array do ... forend
- file — переменная, доступная внутри тела цикла

---

## 🔧 Встроенные функции

| Имя | Аргументы | Возвращает | Описание |
|-----|-----------|------------|----------|
| `getcwd()` | — | Path | Текущая директория |
| `list_files(path)` | Path | Array<String> | Список файлов в папке |
| `read_file(path)` | Path | String/Array | Чтение .txt, .csv, .xlsx |
| `now()` | — | String (RFC3339) | Текущее время |
| `print(...)` | any... | null | Печать значений |


---

## 🧪 Пример программы
```DataCode
global basePath = getcwd()
local SearchPath = basePath / 'data'
global files = list_files(SearchPath)

for file in files do
    print('-', file)
    local content = read_file(SearchPath / file)
    print(content)
forend
```

---

## 📦 Поддерживаемые типы

| Тип | Пример | Описание |
|-----|--------|----------|
| String | `'abc'`, `'hello.txt'` | Всегда в одинарных кавычках |
| Path | `base / 'file.csv'` | Строится через `/` |
| Array | `['a', 'b']` (в будущем) | Пока возвращается из `list_files` |
| Number | `42`, `3.14` | Поддержка в будущем |
| Null | — | Возвращается `print(...)` |


---

## ⚠️ Ошибки

Типичные сообщения об ошибках:
- Unknown variable: foo
- Invalid / expression
- Unsupported expression
- read_file() expects a path

---

## 📚 Расширение

Проект легко расширяется:
- Добавить функции в builtins.rs
- Добавить типы в value.rs
- Добавить синтаксис в interpreter.rs

---

## 🧪 Тестирование

Выполните:
```bash
cargo test
```
Тесты проверяют:
- Объявление переменных
- Конкатенацию путей
- Вызов встроенных функций
- Исполнение for-циклов

---

## 🛠 Пример вызова из CLI
```bash
cargo run
```

---

## 🎯 Интерактивный REPL

### Запуск
```bash
cargo run
```

### Специальные команды REPL
- `help` — показать справку
- `exit` или `quit` — выйти из интерпретатора
- `clear` — очистить экран
- `vars` — показать все переменные
- `reset` — сбросить интерпретатор

### Пример сессии
```
🧠 DataCode Interactive Interpreter
>>> global x = 10
✓ x = Number(10.0)
>>> global y = 20
✓ y = Number(20.0)
>>> global result = (x + y) * 2
✓ result = Number(60.0)
>>> print('Result is:', result)
Result is: 60
>>> vars
📊 Current Variables:
  x = Number(10.0)
  y = Number(20.0)
  result = Number(60.0)
>>> exit
Goodbye! �
```

### Многострочные конструкции
REPL поддерживает многострочный ввод для циклов:
```
>>> for i in [1, 2, 3] do
...     print('Number:', i)
...     global doubled = i * 2
...     print('Doubled:', doubled)
... forend
Number: 1
Doubled: 2
Number: 2
Doubled: 4
Number: 3
Doubled: 6
```

## 📅 Планы развития
### ✅ Реализовано
- ✅ Улучшенная система ошибок с детальными сообщениями
- ✅ Мощный парсер выражений с приоритетом операторов
- ✅ Арифметические операции (+, -, *, /)
- ✅ Операторы сравнения (==, !=, <, >, <=, >=)
- ✅ Логические операции (and, or, not)
- ✅ Интерактивный REPL с многострочной поддержкой
- ✅ Поддержка global / local переменных
- ✅ Встроенные функции для работы с файлами
- ✅ Циклы for ... in

### 🔄 В разработке
- 🔄 Условные конструкции if/else/endif
- 🔄 Пользовательские функции
- 🔄 Расширенные встроенные функции

### 📋 Планируется
- 📋 Объекты (Object) и методы
- 📋 Массивы с индексацией
- 📋 Импорт модулей
- 📋 Обработка исключений

---

## 🧑‍💻 Автор

Made by Igornet0.