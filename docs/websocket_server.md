# DataCode WebSocket Server

WebSocket сервер для удаленного выполнения кода DataCode.

## Запуск сервера

```bash
# Запуск на адресе по умолчанию (127.0.0.1:8080)
datacode --websocket

# Запуск с указанием хоста и порта через флаги
datacode --websocket --host 0.0.0.0 --port 8899

# Запуск на кастомном адресе через переменную окружения
DATACODE_WS_ADDRESS=0.0.0.0:3000 datacode --websocket

# Комбинация: флаги имеют приоритет над переменной окружения
DATACODE_WS_ADDRESS=127.0.0.1:8080 datacode --websocket --host 0.0.0.0 --port 8899
# Результат: сервер запустится на 0.0.0.0:8899
```

## Протокол

### Подключение

Подключитесь к WebSocket серверу по адресу `ws://127.0.0.1:8080` (или указанному адресу).

### Формат запроса

WebSocket сервер поддерживает несколько типов запросов. Все запросы должны содержать поле `type` для указания типа операции.

#### Выполнение кода

Отправьте JSON сообщение с типом `execute` и полем `code`:

```json
{
  "type": "execute",
  "code": "print('Hello, World!')"
}
```

**Обратная совместимость:** Старый формат без поля `type` также поддерживается:

```json
{
  "code": "print('Hello, World!')"
}
```

#### Подключение к SMB шаре

Для подключения к SMB (Samba/CIFS) шаре используйте тип `smb_connect`:

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
- `login` - имя пользователя
- `password` - пароль пользователя
- `domain` - домен (обычно `WORKGROUP` или имя домена, может быть пустой строкой)
- `share_name` - имя SMB шары

**Ответ:**
```json
{
  "success": true,
  "message": "Успешно подключено к SMB шаре 'share_name'",
  "error": null
}
```

### Формат ответа

Сервер вернет JSON с результатом выполнения:

**Успешное выполнение:**
```json
{
  "success": true,
  "output": "Hello, World!\n",
  "error": null
}
```

**Ошибка выполнения:**
```json
{
  "success": false,
  "output": "",
  "error": "Ошибка: переменная 'x' не определена"
}
```

## Примеры использования

### JavaScript/Node.js

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:8080');

ws.on('open', function open() {
    const request = {
        type: "execute",
        code: "print('Hello from WebSocket!')"
    };
    ws.send(JSON.stringify(request));
});

ws.on('message', function message(data) {
    const response = JSON.parse(data);
    console.log('Output:', response.output);
    if (response.error) {
        console.error('Error:', response.error);
    }
});
```

### Python

```python
import asyncio
import websockets
import json

async def execute_code():
    uri = "ws://127.0.0.1:8080"
    async with websockets.connect(uri) as websocket:
        request = {
            "type": "execute",
            "code": "print('Hello from Python!')"
        }
        await websocket.send(json.dumps(request))
        response = json.loads(await websocket.recv())
        print("Output:", response["output"])
        if response["error"]:
            print("Error:", response["error"])

asyncio.run(execute_code())
```

### cURL (через wscat)

```bash
# Установите wscat: npm install -g wscat
wscat -c ws://127.0.0.1:8080
# Затем отправьте:
{"type": "execute", "code": "print('Hello!')"}
```

## SMB Connection (Подключение к SMB шаре)

WebSocket сервер поддерживает подключение к SMB (Samba/CIFS) шаре для работы с файлами на удаленных серверах.

### Требования

**Для Linux/Mac:**
```bash
brew install samba  # macOS
# или
sudo apt-get install samba-client  # Ubuntu/Debian
```

**Для Windows:** SMB клиент встроен в систему.

### Использование lib:// протокола

После успешного подключения к SMB шаре через запрос `smb_connect`, вы можете использовать специальный протокол `lib://` в DataCode скриптах:

```
lib://share_name/path/to/file
```

Где `share_name` - имя подключенной SMB шары, а `path/to/file` - путь к файлу на шаре.

### Пример работы с SMB

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
        response = json.loads(await websocket.recv())
        print("SMB Connect:", response)
        
        # 2. Выполнение DataCode скрипта с использованием SMB
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
        response = json.loads(await websocket.recv())
        print("Execute:", response)

asyncio.run(smb_example())
```

### Поддерживаемые операции

После подключения к SMB шаре доступны следующие операции в DataCode:

- **list_files(path("lib://share_name/dir"))** - получить список файлов
- **read_file(path("lib://share_name/file.csv"))** - прочитать файл (поддерживаются CSV, XLSX, TXT)

Подробнее см. `examples/08-websocket/README.md`.

## Особенности

1. **Изоляция сессий**: Каждый клиент получает свой собственный интерпретатор. Переменные и функции, определенные одним клиентом, не видны другим.

2. **Перехват вывода**: Все вызовы `print()` перехватываются и отправляются клиенту в поле `output`.

3. **Обработка ошибок**: Ошибки выполнения возвращаются в поле `error`, а `success` устанавливается в `false`.

4. **Многострочный код**: Поддерживается выполнение многострочного кода:

```json
{
  "type": "execute",
  "code": "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"
}
```

5. **SMB подключения**: Каждый клиент имеет свой набор SMB подключений, которые автоматически закрываются при отключении клиента.

## Веб-клиент

Откройте файл `examples/websocket_client_example.html` в браузере для интерактивного тестирования WebSocket сервера.

## Безопасность

⚠️ **Внимание**: Текущая реализация не включает аутентификацию или ограничения доступа. Не используйте на публичных серверах без дополнительной защиты!

## Ограничения

- Интерпретатор не является потокобезопасным (`Send`), поэтому каждый клиент обрабатывается в отдельной локальной задаче
- Переменные и функции не сохраняются между запросами от одного клиента (каждый запрос выполняется в том же интерпретаторе, но состояние может быть изменено)

