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

Отправьте JSON сообщение с полем `code`:

```json
{
  "code": "print('Hello, World!')"
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
{"code": "print('Hello!')"}
```

## Особенности

1. **Изоляция сессий**: Каждый клиент получает свой собственный интерпретатор. Переменные и функции, определенные одним клиентом, не видны другим.

2. **Перехват вывода**: Все вызовы `print()` перехватываются и отправляются клиенту в поле `output`.

3. **Обработка ошибок**: Ошибки выполнения возвращаются в поле `error`, а `success` устанавливается в `false`.

4. **Многострочный код**: Поддерживается выполнение многострочного кода:

```json
{
  "code": "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"
}
```

## Веб-клиент

Откройте файл `examples/websocket_client_example.html` в браузере для интерактивного тестирования WebSocket сервера.

## Безопасность

⚠️ **Внимание**: Текущая реализация не включает аутентификацию или ограничения доступа. Не используйте на публичных серверах без дополнительной защиты!

## Ограничения

- Интерпретатор не является потокобезопасным (`Send`), поэтому каждый клиент обрабатывается в отдельной локальной задаче
- Переменные и функции не сохраняются между запросами от одного клиента (каждый запрос выполняется в том же интерпретаторе, но состояние может быть изменено)

