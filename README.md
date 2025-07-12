# 🧠 DataCode Language

**DataCode** — это интерпретируемый язык, написанный на Rust, предназначенный для работы с файловой системой и данными. Он прост, декларативен и легко расширяем.

---

## 🚀 Быстрый старт

```rust
use DataCode::interpreter::Interpreter;

fn main() {
    let mut interp = Interpreter::new();
    interp.exec("global basePath = getcwd()").unwrap();
    interp.exec("global files = list_files(basePath / 'data')").unwrap();
}

```
---

## 📄 Синтаксис языка

🔹 Переменные
```DataCode
global path = getcwd()
local subdir = 'data'
```
•	global — сохраняет переменную глобально
•	local — ограничена текущим контекстом (например, циклом)

🔹 Конкатенация путей
```DataCode
global dir = basePath / 'data' / 'images'
```
•	/ используется для Path + String

🔹 Сложение строк
```DataCode
global name = 'image' + '001.jpg'
```
•	+ объединяет строки

---

## 🔁 Циклы
```DataCode
for file in files do
    local path = basePath / 'data' / file
    local text = read_file(path)
    print('>>', file, 'length:', text)
forend
```
•	for x in array do ... forend
•	file — переменная, доступная внутри тела цикла

---

## 🔧 Встроенные функции

Имя	Аргументы	Возвращает	Описание
getcwd()	—	Path	Текущая директория
list_files(path)	Path	Array<String>	Список файлов в папке
read_file(path)	Path	String/Array	Чтение .txt, .csv, .xlsx
now()	—	String (RFC3339)	Текущее время
print(...)	any...	null	Печать значений


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

Тип	Пример	Описание
String	'abc', 'hello.txt'	Всегда в одинарных кавычках
Path	base / 'file.csv'	Строится через /
Array	['a', 'b'] (в будущем)	Пока возвращается из list_files
Number	42, 3.14	Поддержка в будущем
Null	—	Возвращается print(...)


---

## ⚠️ Ошибки

Типичные сообщения об ошибках:
•	Unknown variable: foo
•	Invalid / expression
•	Unsupported expression
•	read_file() expects a path

---

## 📚 Расширение

Проект легко расширяется:
•	Добавить функции в builtins.rs
•	Добавить типы в value.rs
•	Добавить синтаксис в interpreter.rs

---

## 🧪 Тестирование

Выполните:
```bash
cargo test
```
Тесты проверяют:
•	Объявление переменных
•	Конкатенацию путей
•	Вызов встроенных функций
•	Исполнение for-циклов

---

## 🛠 Пример вызова из CLI
```bash
cargo run
```

---

## 📅 Планы
•	Поддержка global / local
•	Встроенные функции
•	Циклы for ... in
•	Ветвление if/else
•	Пользовательские функции
•	Объекты (Object)
•	Строки в двойных кавычках

---

## 🧑‍💻 Автор

Made by Igornet0.