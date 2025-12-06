# WebSocket Примеры для DataCode

Эта папка содержит примеры использования WebSocket сервера DataCode.

## Файлы

- **test_websocket.js** - Node.js клиент для тестирования WebSocket сервера
- **test_websocket.py** - Python клиент для тестирования WebSocket сервера  
- **test_websocket.sh** - Bash скрипт для тестирования (требуется websocat)
- **websocket_client_example.html** - HTML клиент с веб-интерфейсом
- **websocket_requests.json** - JSON файл с примерами запросов
- **websocket_test_requests.md** - Документация по тестированию WebSocket
- **test_smb_connection.py** - Python клиент для тестирования SMB подключения через WebSocket
- **test_smb_load_data.dc** - Пример DataCode скрипта для работы с файлами на SMB шаре

## Запуск сервера

```bash
datacode --websocket --host 0.0.0.0 --port 8899
```

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

### Bash
```bash
cd examples/08-websocket
cargo install websocat  # если еще не установлен
bash test_websocket.sh
```

### HTML клиент
Откройте `websocket_client_example.html` в браузере.

## Формат запросов

Все запросы отправляются в формате JSON:

```json
{
  "code": "print('Hello, World!')"
}
```

## Формат ответов

```json
{
  "success": true,
  "output": "Hello, World!\n",
  "error": null
}
```

## Примеры кода

### Простой вывод
```json
{"code": "print('Hello, World!')"}
```

### Переменные
```json
{"code": "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"}
```

### Цикл
```json
{"code": "for i in [1, 2, 3] do\n    print('Number:', i)\nnext i"}
```

### Функция
```json
{"code": "global function greet(name) do\n    return 'Hello, ' + name + '!'\nendfunction\nprint(greet('DataCode'))"}
```

## SMB Connection (Подключение к SMB шаре)

WebSocket сервер поддерживает подключение к SMB (Samba/CIFS) шаре для работы с файлами на удаленных серверах.

### Требования

**Для Linux/Mac:**
```bash
# Установка smbclient (Samba)
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
  "error": "Ошибка подключения к SMB: ..."
}
```

### Использование lib:// протокола в DataCode

После успешного подключения к SMB шаре, вы можете использовать специальный протокол `lib://` в DataCode скриптах для доступа к файлам на шаре.

**Формат пути:**
```
lib://share_name/path/to/file
```

Где:
- `share_name` - имя подключенной SMB шары
- `path/to/file` - путь к файлу или директории на шаре

### Пример использования

#### 1. Подключение и выполнение скрипта (Python)

Используйте `test_smb_connection.py` для подключения к SMB и выполнения DataCode скрипта:

```bash
cd examples/08-websocket
python3 test_smb_connection.py test_smb_load_data.dc
```

Скрипт автоматически:
1. Подключается к WebSocket серверу
2. Отправляет запрос на подключение к SMB шаре
3. Выполняет DataCode скрипт из указанного файла

**Настройка параметров подключения:**

Отредактируйте переменные в начале `test_smb_connection.py`:
```python
username = "your_username"
password = "your_password"
smb_server = "192.168.1.100"  # IP или имя сервера
smb_share = "share_name"      # Имя SMB шары
domain = "WORKGROUP"          # Домен (может быть пустой строкой)
```

#### 2. Пример DataCode скрипта для работы с SMB

Пример из `test_smb_load_data.dc`:

```datacode
local path = path("lib://Stream/my_dir")

for path_dir in list_files(path) do
    for file in list_files(path / path_dir) do
        if contains(file, 'data') do
            print(file)
            global data = read_file(path / path_dir / file, 11, ' data')
            print(len(data))
            table_info(data)
        endif
    next file
next path_dir
```

**Что делает скрипт:**
1. Создает путь к директории на SMB шаре: `lib://Stream/my_dir`
2. Перебирает все файлы в директории
3. Для каждого подкаталога перебирает файлы
4. Если имя файла содержит 'data', читает его с помощью `read_file`
5. Выводит информацию о загруженной таблице

### Поддерживаемые операции с SMB

После подключения к SMB шаре через WebSocket, в DataCode скриптах доступны следующие операции:

#### list_files
Получить список файлов в директории на SMB шаре:
```datacode
local files = list_files(path("lib://share_name/directory"))
```

#### read_file
Прочитать файл с SMB шары:
```datacode
local data = read_file(path("lib://share_name/path/to/file.csv"))
```

Поддерживаемые форматы файлов:
- **CSV** - автоматически парсится в таблицу
- **XLSX** - автоматически парсится в таблицу (поддержка листов)
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
local base_path = path("lib://share_name")
local file_path = base_path / "subdirectory" / "file.csv"
local data = read_file(file_path)
```

### Важные замечания

1. **Подключение сохраняется на время сессии** - SMB подключение активно до отключения клиента от WebSocket сервера
2. **Отдельное подключение для каждого клиента** - каждый WebSocket клиент имеет свой собственный набор SMB подключений
3. **Безопасность** - пароли передаются в открытом виде в JSON запросах, используйте защищенное соединение (WSS) в production
4. **Производительность** - операции с SMB медленнее, чем с локальной файловой системой, учитывайте это при работе с большими файлами

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
        
        # 2. Выполнение DataCode скрипта
        code = """
        local files = list_files(path("lib://data/reports"))
        for file in files do
            print("File:", file)
        next file
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

