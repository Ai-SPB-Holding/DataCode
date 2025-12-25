# Примеры WebSocket для DataCode

Эта папка содержит примеры использования WebSocket сервера DataCode.

## Файлы

- **test_websocket.js** - Node.js клиент для тестирования WebSocket сервера
- **test_websocket.py** - Python клиент для тестирования WebSocket сервера  
- **test_websocket.sh** - Bash скрипт для тестирования (требует websocat)
- **websocket_client_example.html** - HTML клиент с веб-интерфейсом
- **websocket_requests.json** - JSON файл с примерами запросов
- **websocket_test_requests.md** - Документация по тестированию WebSocket
- **test_smb_connection.py** - Python клиент для тестирования SMB подключения через WebSocket
- **test_smb_load_data.dc** - Пример скрипта DataCode для работы с файлами на SMB шаре
- **test_file_upload.py** - Python клиент для тестирования загрузки файлов через WebSocket (требует режим --use-ve)

## Запуск сервера

**Стандартный режим:**
```bash
datacode --websocket --host 0.0.0.0 --port 8899
```

**Режим виртуальной среды (для загрузки файлов):**
```bash
datacode --websocket --host 0.0.0.0 --port 8899 --use-ve
```

Флаг `--use-ve` включает:
- Изолированные папки сессий в `src/temp_sessions`
- `getcwd()` возвращает пустую строку
- Поддержка загрузки файлов через WebSocket
- Автоматическая очистка папок сессий при отключении

## Использование

### Node.js
```bash
cd examples/08-websocket
node test_websocket.js
```

### Python
```bash
cd examples/08-websocket
pip install websockets
python3 test_websocket.py
```

### Загрузка файлов (Python, требует --use-ve)
```bash
cd examples/08-websocket
pip install websockets
# Убедитесь, что сервер запущен с флагом --use-ve
python3 test_file_upload.py
```

### Bash
```bash
cd examples/08-websocket
cargo install websocat  # если ещё не установлен
bash test_websocket.sh
```

### HTML клиент
Откройте `websocket_client_example.html` в браузере.

## Формат запроса

Все запросы отправляются в формате JSON:

```json
{
  "code": "print('Привет, Мир!')"
}
```

## Формат ответа

```json
{
  "success": true,
  "output": "Привет, Мир!\n",
  "error": null
}
```

## Примеры кода

### Простой вывод
```json
{"code": "print('Привет, Мир!')"}
```

### Переменные
```json
{"code": "global x = 10\nglobal y = 20\nprint('Сумма:', x + y)"}
```

### Цикл
```json
{"code": "for i in [1, 2, 3] {\n    print('Число:', i)\n}"}
```

### Функция
```json
{"code": "fn greet(name) {\n    return 'Привет, ' + name + '!'\n}\nprint(greet('DataCode'))"}
```

## Подключение к SMB шаре

WebSocket сервер поддерживает подключение к SMB (Samba/CIFS) шаре для работы с файлами на удалённых серверах.

### Требования

**Для Linux/Mac:**
```bash
# Установить smbclient (Samba)
brew install samba  # macOS
# или
sudo apt-get install samba-client  # Ubuntu/Debian
```

**Для Windows:**
SMB клиент встроен в систему, дополнительная установка не требуется.

### Подключение к SMB шаре

Для подключения к SMB шаре отправьте запрос с типом `smb_connect`:

```json
{
  "type": "smb_connect",
  "ip": "192.168.1.100",
  "login": "username",
  "password": "password",
  "domain": "WORKGROUP",
  "share_name": "share_name"
}
```

**Параметры:**
- `ip` - IP адрес или имя SMB сервера
- `login` - имя пользователя для подключения
- `password` - пароль пользователя
- `domain` - домен (обычно `WORKGROUP` для Windows или имя домена, может быть пустой строкой)
- `share_name` - имя SMB шары для подключения

**Ответ сервера:**
```json
{
  "success": true,
  "message": "Успешно подключено к SMB шаре 'share_name'",
  "error": null
}
```

При ошибке:
```json
{
  "success": false,
  "message": "",
  "error": "Ошибка подключения SMB: ..."
}
```

### Использование протокола lib:// в DataCode

После успешного подключения к SMB шаре вы можете использовать специальный протокол `lib://` в скриптах DataCode для доступа к файлам на шаре.

**Формат пути:**
```
lib://share_name/path/to/file
```

Где:
- `share_name` - имя подключённой SMB шары
- `path/to/file` - путь к файлу или директории на шаре

### Пример использования

#### 1. Подключение и выполнение скрипта (Python)

Используйте `test_smb_connection.py` для подключения к SMB и выполнения скрипта DataCode:

```bash
cd examples/08-websocket
python3 test_smb_connection.py test_smb_load_data.dc
```

Скрипт автоматически:
1. Подключается к WebSocket серверу
2. Отправляет запрос на подключение к SMB шаре
3. Выполняет скрипт DataCode из указанного файла

**Настройка параметров подключения:**

Отредактируйте переменные в начале `test_smb_connection.py`:
```python
username = "your_username"
password = "your_password"
smb_server = "192.168.1.100"  # IP или имя сервера
smb_share = "share_name"      # Имя SMB шары
domain = "WORKGROUP"          # Домен (может быть пустой строкой)
```

#### 2. Пример скрипта DataCode для работы с SMB

Пример из `test_smb_load_data.dc`:

```datacode
let path = path("lib://Stream/my_dir")

for path_dir in list_files(path) {
    for file in list_files(path / path_dir) {
        if contains(file, 'data') {
            print(file)
            global data = read_file(path / path_dir / file, 11, ' data')
            print(len(data))
            table_info(data)
        }
    }
}
```

**Что делает скрипт:**
1. Создаёт путь к директории на SMB шаре: `lib://Stream/my_dir`
2. Итерируется по всем файлам в директории
3. Для каждой поддиректории итерируется по файлам
4. Если имя файла содержит 'data', читает его с помощью `read_file`
5. Выводит информацию о загруженной таблице

### Поддерживаемые операции SMB

После подключения к SMB шаре через WebSocket, в скриптах DataCode доступны следующие операции:

#### list_files
Получить список файлов в директории на SMB шаре:
```datacode
let files = list_files(path("lib://share_name/directory"))
```

#### read_file
Прочитать файл с SMB шары:
```datacode
let data = read_file(path("lib://share_name/path/to/file.csv"))
```

Поддерживаемые форматы файлов:
- **CSV** - автоматически парсится в таблицу
- **XLSX** - автоматически парсится в таблицу (с поддержкой листов)
- **TXT** - читается как текстовая строка

**Параметры read_file для CSV/XLSX:**
```datacode
# CSV с указанием строки заголовка
read_file(path("lib://share/file.csv"), 0)

# XLSX с указанием листа
read_file(path("lib://share/file.xlsx"), 0, "Sheet1")
```

### Работа с путями

Пути на SMB шаре можно комбинировать с помощью оператора `/`:

```datacode
let base_path = path("lib://share_name")
let file_path = base_path / "subdirectory" / "file.csv"
let data = read_file(file_path)
```

### Важные замечания

1. **Подключение сохраняется для сессии** - SMB подключение активно до отключения клиента от WebSocket сервера
2. **Отдельное подключение для каждого клиента** - каждый WebSocket клиент имеет свой набор SMB подключений
3. **Безопасность** - пароли передаются в открытом виде в JSON запросах, используйте защищённое соединение (WSS) в продакшене
4. **Производительность** - операции SMB медленнее локальной файловой системы, учитывайте это при работе с большими файлами

### Полный пример работы с SMB

```python
import asyncio
import websockets
import json

async def smb_example():
    async with websockets.connect("ws://localhost:8899") as websocket:
        # 1. Подключение к SMB
        connect_request = {
            "type": "smb_connect",
            "ip": "192.168.1.100",
            "login": "user",
            "password": "pass",
            "domain": "WORKGROUP",
            "share_name": "data"
        }
        await websocket.send(json.dumps(connect_request))
        response = await websocket.recv()
        print("SMB Connect:", json.loads(response))
        
        # 2. Выполнение скрипта DataCode
        code = """
        let files = list_files(path("lib://data/reports"))
        for file in files {
            print("Файл:", file)
        }
        """
        
        execute_request = {
            "type": "execute",
            "code": code
        }
        await websocket.send(json.dumps(execute_request))
        response = await websocket.recv()
        print("Execute:", json.loads(response))

asyncio.run(smb_example())
```

## Загрузка файлов через WebSocket

WebSocket сервер поддерживает загрузку файлов при запуске с флагом `--use-ve`. Каждое подключение клиента получает изолированную папку сессии в `src/temp_sessions`.

### Требования

1. Запустите сервер с флагом `--use-ve`:
```bash
datacode --websocket --host 0.0.0.0 --port 8899 --use-ve
```

2. Установите зависимости Python:
```bash
pip install websockets
```

### Загрузка файлов

#### Текстовые файлы

```python
import asyncio
import websockets
import json

async def upload_text_file():
    async with websockets.connect("ws://localhost:8899") as websocket:
        upload_request = {
            "type": "upload_file",
            "filename": "test.txt",
            "content": "Привет, DataCode!\nЭто тестовый файл."
        }
        await websocket.send(json.dumps(upload_request))
        response = await websocket.recv()
        result = json.loads(response)
        print(result)

asyncio.run(upload_text_file())
```

#### Бинарные файлы (Base64)

```python
import asyncio
import websockets
import json
import base64

async def upload_binary_file():
    async with websockets.connect("ws://localhost:8899") as websocket:
        # Прочитать бинарный файл и закодировать в base64
        with open("image.png", "rb") as f:
            binary_data = f.read()
        base64_data = base64.b64encode(binary_data).decode('utf-8')
        
        upload_request = {
            "type": "upload_file",
            "filename": "image.png",
            "content": f"base64:{base64_data}"
        }
        await websocket.send(json.dumps(upload_request))
        response = await websocket.recv()
        result = json.loads(response)
        print(result)

asyncio.run(upload_binary_file())
```

#### Файлы в поддиректориях

```python
upload_request = {
    "type": "upload_file",
    "filename": "subdir/nested_file.txt",
    "content": "Содержимое вложенного файла"
}
```

### Ответ сервера

**Успех:**
```json
{
  "success": true,
  "message": "Файл test.txt успешно загружен",
  "error": null
}
```

**Ошибка:**
```json
{
  "success": false,
  "message": "",
  "error": "Ошибка записи файла: ..."
}
```

### Работа с загруженными файлами

После загрузки файлов вы можете работать с ними в скриптах DataCode:

```python
# Загрузить CSV файл
upload_csv = {
    "type": "upload_file",
    "filename": "data.csv",
    "content": "name,age\nAlice,30\nBob,25"
}
await websocket.send(json.dumps(upload_csv))

# Прочитать и обработать CSV
code = """
global data = read_file(path("data.csv"), 0)
print("Загружено строк:", len(data))
table_info(data)
"""
execute_request = {
    "type": "execute",
    "code": code
}
await websocket.send(json.dumps(execute_request))
```

### Важные замечания

1. **Изоляция сессий** - Каждое WebSocket подключение получает свою папку сессии
2. **Автоматическая очистка** - Папки сессий автоматически удаляются при отключении клиента
3. **Поведение getcwd()** - В режиме `--use-ve` `getcwd()` возвращает пустую строку
4. **Пути к файлам** - Используйте относительные пути в скриптах DataCode (например, `path("data.csv")`)
5. **Кодирование Base64** - Бинарные файлы должны иметь префикс `base64:` в поле content

### Полный пример

См. `test_file_upload.py` для полного примера с несколькими тестовыми случаями:
```bash
python3 test_file_upload.py
```
