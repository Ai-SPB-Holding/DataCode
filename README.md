# 🧠 DataCode - Interactive Programming Language

**DataCode** is a simple, interactive programming language designed for fast data processing and easy learning. It features an intuitive syntax, powerful array support, built-in functions, and user-defined functions with local scope.

## 🚀 Features

- **Interactive REPL** with multiline support and command history
- **File execution** - write programs in `.dc` files
- **Array literals** - `[1, 2, 3]`, `['a', 'b']`, mixed types supported
- **Array indexing** - `arr[0]`, `nested[0][1]` with full nesting support
- **User-defined functions** with local scope, parameters and recursion
- **Conditional statements** - if/else/endif with nesting support
- **For loops** - iterate over arrays with `for...in`
- **Arithmetic and logical operations** with proper precedence
- **String manipulation** and concatenation
- **Table operations** - work with CSV/Excel files, automatic typing
- **40+ built-in functions** - math, string, array, file, and table operations
- **Path manipulation** for file system operations
- **Flexible data types** - numbers, strings, booleans, arrays, tables, paths
- **Improved error messages** with line numbers and context
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
# Create a simple DataCode file
echo 'print("Hello, DataCode!")' > hello.dc

# Create an array example
echo 'global arr = [1, 2, 3]
print("Array:", arr)
print("First element:", arr[0])' > arrays.dc

# Execute the files
datacode hello.dc          # (after global installation)
datacode arrays.dc
# or
cargo run hello.dc         # (development mode)
cargo run arrays.dc
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

### 🔹 Массивы
```DataCode
# Создание массивов любых типов
global numbers = [1, 2, 3, 4, 5]
global strings = ['hello', 'world', 'datacode']
global booleans = [true, false, true]
global mixed = [1, 'hello', true, 3.14]
global empty = []

# Вложенные массивы
global matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
global nested_mixed = [[1, 'a'], [true, 3.14]]

# Доступ к элементам (индексирование с 0)
print(numbers[0])        # 1
print(strings[1])        # world
print(mixed[2])          # true
print(matrix[0][1])      # 2
print(nested_mixed[1][0]) # true

# Trailing comma поддерживается
global trailing = [1, 2, 3,]

# Использование в циклах
for item in [1, 2, 3] do
    print('Item:', item)
forend
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
# Цикл по массиву переменных
for file in files do
    local path = basePath / 'data' / file
    local text = read_file(path)
    print('>>', file, 'length:', text)
forend

# Цикл по литералу массива
for number in [1, 2, 3, 4, 5] do
    print('Number:', number, 'Squared:', number * number)
forend

# Цикл по смешанному массиву
for item in ['hello', 42, true] do
    print('Item:', item)
forend

# Цикл по вложенному массиву
for row in [[1, 2], [3, 4], [5, 6]] do
    print('Row:', row, 'Sum:', sum(row))
forend
```
- `for x in array do ... forend` - итерация по массиву
- `x` — переменная, доступная внутри тела цикла
- Поддерживаются как переменные-массивы, так и литералы массивов

---

## 🔧 Встроенные функции (40+)

### 📁 Файловые операции
| Функция | Описание |
|---------|----------|
| `getcwd()` | Текущая директория |
| `path(string)` | Создание пути из строки |
| `read_file(path)` | Чтение файлов (.txt, .csv, .xlsx) |

### 🧮 Математические функции
| Функция | Описание |
|---------|----------|
| `abs(n)` | Абсолютное значение |
| `sqrt(n)` | Квадратный корень |
| `pow(base, exp)` | Возведение в степень |
| `min(...)` | Минимальное значение |
| `max(...)` | Максимальное значение |
| `round(n)` | Округление |

### 📝 Строковые функции
| Функция | Описание |
|---------|----------|
| `length(str)` | Длина строки |
| `upper(str)` | В верхний регистр |
| `lower(str)` | В нижний регистр |
| `trim(str)` | Удаление пробелов |
| `split(str, delim)` | Разделение строки |
| `join(array, delim)` | Объединение массива |
| `contains(str, substr)` | Проверка вхождения |

### 📊 Функции массивов
| Функция | Описание |
|---------|----------|
| `push(array, item)` | Добавить элемент |
| `pop(array)` | Удалить последний |
| `unique(array)` | Уникальные элементы |
| `reverse(array)` | Обратный порядок |
| `sort(array)` | Сортировка |
| `sum(array)` | Сумма чисел |
| `average(array)` | Среднее значение |
| `count(array)` | Количество элементов |

### 📋 Табличные функции
| Функция | Описание |
|---------|----------|
| `table(data, headers)` | Создание таблицы |
| `show_table(table)` | Вывод таблицы |
| `table_info(table)` | Информация о таблице |
| `table_head(table, n)` | Первые n строк |
| `table_tail(table, n)` | Последние n строк |
| `table_select(table, cols)` | Выбор колонок |
| `table_sort(table, col, asc)` | Сортировка таблицы |

### 🔧 Утилиты
| Функция | Описание |
|---------|----------|
| `print(...)` | Вывод значений |
| `now()` | Текущее время |


---

## 🧪 Пример программы
```DataCode
# Пользовательская функция для анализа массивов
global function analyze_array(arr) do
    local size = count(arr)
    local sum_val = sum(arr)
    local avg_val = average(arr)

    print('📊 Анализ массива:', arr)
    print('  Размер:', size)
    print('  Сумма:', sum_val)
    print('  Среднее:', avg_val)

    return [size, sum_val, avg_val]
endfunction

# Работа с массивами и файлами
global basePath = getcwd()
global dataPath = basePath / 'examples'

# Создаем массивы данных
global numbers = [10, 20, 30, 40, 50]
global mixed_data = [1, 'test', true, 3.14]
global matrix = [[1, 2], [3, 4], [5, 6]]

print('🧮 Анализ числовых данных')
global stats = analyze_array(numbers)

print('')
print('📋 Работа с файлами')
global files = ['sample.csv', 'data.txt']

for file in files do
    local fullPath = dataPath / file
    print('📄 Обрабатываем:', file)

    # Если это CSV файл, показываем таблицу
    if contains(file, '.csv') do
        local table = read_file(fullPath)
        print('📊 Содержимое таблицы:')
        table_head(table, 3)
    endif
forend

print('')
print('🔢 Работа с вложенными массивами')
for row in matrix do
    local row_sum = sum(row)
    print('Строка:', row, 'Сумма:', row_sum)
forend

print('✅ Анализ завершен!')
```

---

## 📦 Поддерживаемые типы

| Тип | Пример | Описание |
|-----|--------|----------|
| String | `'abc'`, `'hello.txt'` | Всегда в одинарных кавычках |
| Number | `42`, `3.14` | Целые и дробные числа |
| Bool | `true`, `false` | Логические значения |
| Array | `[1, 'hello', true]` | Массивы любых типов данных |
| Path | `base / 'file.csv'` | Строится через `/` |
| Table | `table(data, headers)` | Табличные данные |
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
REPL поддерживает многострочный ввод для циклов и массивов:
```
>>> global arr = [1, 2, 3, 4, 5]
✓ arr = Array([Number(1.0), Number(2.0), Number(3.0), Number(4.0), Number(5.0)])
>>> print(arr[0])
1
>>> global nested = [[1, 2], [3, 4]]
✓ nested = Array([Array([Number(1.0), Number(2.0)]), Array([Number(3.0), Number(4.0)])])
>>> print(nested[0][1])
2
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

## 📅 Статус реализации
### ✅ Полностью реализовано
- ✅ Улучшенная система ошибок с детальными сообщениями
- ✅ Мощный парсер выражений с приоритетом операторов
- ✅ **Литералы массивов** `[1, 2, 3]`, `['a', 'b']`, смешанные типы
- ✅ **Индексирование массивов** `arr[0]`, `nested[0][1]` с полной поддержкой вложенности
- ✅ Арифметические операции (+, -, *, /)
- ✅ Операторы сравнения (==, !=, <, >, <=, >=)
- ✅ Логические операции (and, or, not)
- ✅ Интерактивный REPL с многострочной поддержкой и историей команд
- ✅ Поддержка global / local переменных
- ✅ Условные конструкции if/else/endif (с поддержкой вложенности)
- ✅ Пользовательские функции с локальной областью видимости
- ✅ Рекурсивные функции
- ✅ Циклы for ... in (включая литералы массивов)
- ✅ 40+ встроенных функций (математические, строковые, файловые, табличные)
- ✅ Работа с таблицами и CSV/Excel файлами
- ✅ Автоматическая типизация данных с предупреждениями
- ✅ Поддержка путей файловой системы
- ✅ Выполнение .dc файлов

### 🔄 Известные ограничения
- ⚠️ Вложенные условия требуют осторожного использования

### 📋 Планируется в будущем
- 📋 Циклы while и do-while
- 📋 Объекты с методами `{key: value}`
- 📋 Импорт модулей
- 📋 Обработка исключений try/catch
- 📋 Деструктуризация массивов

---

## 🧑‍💻 Автор

Made by Igornet0.