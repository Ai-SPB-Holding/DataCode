# Быстрый тест WebSocket сервера

## 1. Запустите сервер

```bash
datacode --websocket --host 0.0.0.0 --port 8899
```

## 2. Откройте веб-клиент

Откройте файл `examples/08-websocket/websocket_client_example.html` в браузере и подключитесь к `ws://127.0.0.1:8899`

## 3. Или используйте готовые скрипты

### Node.js (требуется: npm install ws)
```bash
node examples/08-websocket/test_websocket.js
```

### Python (требуется: pip install websockets)
```bash
python examples/08-websocket/test_websocket.py
```

### Bash (требуется: cargo install websocat)
```bash
./examples/08-websocket/test_websocket.sh
```

## 4. Или отправьте запрос вручную

### Через wscat:
```bash
npm install -g wscat
wscat -c ws://127.0.0.1:8899
# Затем введите:
{"code": "print('Hello, World!')"}
```

### Через websocat:
```bash
cargo install websocat
echo '{"code": "print(\"Hello, World!\")"}' | websocat ws://127.0.0.1:8899
```

## Готовые JSON запросы для копирования:

```json
{"code": "print('Hello, World!')"}
```

```json
{"code": "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"}
```

```json
{"code": "for i in [1, 2, 3] do\n    print('Number:', i)\nnext i"}
```

```json
{"code": "global function greet(name) do\n    return 'Hello, ' + name + '!'\nendfunction\nprint(greet('DataCode'))"}
```

## Ожидаемый ответ:

```json
{
  "success": true,
  "output": "Hello, World!\n",
  "error": null
}
```

