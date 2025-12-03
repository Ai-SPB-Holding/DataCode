# Тестовые запросы для WebSocket сервера DataCode

## Запуск сервера

```bash
datacode --websocket --host 0.0.0.0 --port 8899
```

## Примеры тестовых запросов

### 1. Простой запрос через wscat

```bash
# Установите wscat: npm install -g wscat
wscat -c ws://127.0.0.1:8899

# Затем отправьте:
{"code": "print('Hello, World!')"}
```

### 2. Тест через curl (если установлен websocat)

```bash
# Установите websocat: cargo install websocat
echo '{"code": "print(\"Hello from curl!\")"}' | websocat ws://127.0.0.1:8899
```

### 3. JavaScript/Node.js тест

Создайте файл `test_websocket.js`:

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:8899');

ws.on('open', function open() {
    console.log('✅ Подключено к серверу');
    
    // Тест 1: Простой вывод
    const test1 = {
        code: "print('Hello, World!')"
    };
    console.log('\n📤 Отправка теста 1:', JSON.stringify(test1));
    ws.send(JSON.stringify(test1));
});

ws.on('message', function message(data) {
    const response = JSON.parse(data);
    console.log('\n📥 Получен ответ:');
    console.log('  Success:', response.success);
    console.log('  Output:', response.output);
    if (response.error) {
        console.log('  Error:', response.error);
    }
    
    // Тест 2: Переменные
    if (!ws.test2Sent) {
        ws.test2Sent = true;
        const test2 = {
            code: "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"
        };
        console.log('\n📤 Отправка теста 2:', JSON.stringify(test2));
        ws.send(JSON.stringify(test2));
    } else if (!ws.test3Sent) {
        // Тест 3: Цикл
        ws.test3Sent = true;
        const test3 = {
            code: "for i in [1, 2, 3] do\n    print('Number:', i)\nnext i"
        };
        console.log('\n📤 Отправка теста 3:', JSON.stringify(test3));
        ws.send(JSON.stringify(test3));
    } else {
        ws.close();
    }
});

ws.on('error', function error(err) {
    console.error('❌ Ошибка:', err.message);
});

ws.on('close', function close() {
    console.log('\n🔌 Соединение закрыто');
});
```

Запуск:
```bash
node test_websocket.js
```

### 4. Python тест

Создайте файл `test_websocket.py`:

```python
import asyncio
import websockets
import json

async def test_websocket():
    uri = "ws://127.0.0.1:8899"
    
    async with websockets.connect(uri) as websocket:
        print("✅ Подключено к серверу")
        
        # Тест 1: Простой вывод
        test1 = {
            "code": "print('Hello, World!')"
        }
        print(f"\n📤 Отправка теста 1: {json.dumps(test1)}")
        await websocket.send(json.dumps(test1))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Получен ответ:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        if result.get('error'):
            print(f"  Error: {result['error']}")
        
        # Тест 2: Переменные
        test2 = {
            "code": "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"
        }
        print(f"\n📤 Отправка теста 2: {json.dumps(test2)}")
        await websocket.send(json.dumps(test2))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Получен ответ:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        
        # Тест 3: Функция
        test3 = {
            "code": "global function greet(name) do\n    return 'Hello, ' + name + '!'\nendfunction\nprint(greet('DataCode'))"
        }
        print(f"\n📤 Отправка теста 3: {json.dumps(test3)}")
        await websocket.send(json.dumps(test3))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Получен ответ:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        
        # Тест 4: Ошибка (для проверки обработки ошибок)
        test4 = {
            "code": "print(undefined_variable)"
        }
        print(f"\n📤 Отправка теста 4 (ожидаем ошибку): {json.dumps(test4)}")
        await websocket.send(json.dumps(test4))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Получен ответ:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        if result.get('error'):
            print(f"  Error: {result['error']}")

if __name__ == "__main__":
    asyncio.run(test_websocket())
```

Запуск:
```bash
pip install websockets
python test_websocket.py
```

### 5. Bash скрипт с использованием websocat

Создайте файл `test_websocket.sh`:

```bash
#!/bin/bash

SERVER="ws://127.0.0.1:8899"

echo "🧪 Тестирование WebSocket сервера DataCode"
echo "=========================================="
echo ""

# Тест 1: Простой вывод
echo "📤 Тест 1: Простой вывод"
echo '{"code": "print(\"Hello, World!\")"}' | websocat $SERVER
echo ""

# Тест 2: Переменные
echo "📤 Тест 2: Переменные"
echo '{"code": "global x = 10\nglobal y = 20\nprint(\"Sum:\", x + y)"}' | websocat $SERVER
echo ""

# Тест 3: Цикл
echo "📤 Тест 3: Цикл"
echo '{"code": "for i in [1, 2, 3] do\n    print(\"Number:\", i)\nnext i"}' | websocat $SERVER
echo ""

echo "✅ Тестирование завершено"
```

Запуск:
```bash
chmod +x test_websocket.sh
./test_websocket.sh
```

### 6. Простые JSON запросы для копирования

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

```json
{"code": "print(undefined_variable)"}
```

## Ожидаемые ответы

### Успешное выполнение:
```json
{
  "success": true,
  "output": "Hello, World!\n",
  "error": null
}
```

### Ошибка выполнения:
```json
{
  "success": false,
  "output": "",
  "error": "Ошибка: переменная 'undefined_variable' не определена"
}
```

