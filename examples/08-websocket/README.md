# WebSocket Примеры для DataCode

Эта папка содержит примеры использования WebSocket сервера DataCode.

## Файлы

- **test_websocket.js** - Node.js клиент для тестирования WebSocket сервера
- **test_websocket.py** - Python клиент для тестирования WebSocket сервера  
- **test_websocket.sh** - Bash скрипт для тестирования (требуется websocat)
- **websocket_client_example.html** - HTML клиент с веб-интерфейсом
- **websocket_requests.json** - JSON файл с примерами запросов
- **websocket_test_requests.md** - Документация по тестированию WebSocket

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

