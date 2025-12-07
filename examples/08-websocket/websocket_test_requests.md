# Test Requests for DataCode WebSocket Server

## Starting the Server

```bash
datacode --websocket --host 0.0.0.0 --port 8899
```

## Test Request Examples

### 1. Simple Request via wscat

```bash
# Install wscat: npm install -g wscat
wscat -c ws://127.0.0.1:8899

# Then send:
{"code": "print('Hello, World!')"}
```

### 2. Test via curl (if websocat is installed)

```bash
# Install websocat: cargo install websocat
echo '{"code": "print(\"Hello from curl!\")"}' | websocat ws://127.0.0.1:8899
```

### 3. JavaScript/Node.js Test

Create file `test_websocket.js`:

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:8899');

ws.on('open', function open() {
    console.log('✅ Connected to server');
    
    // Test 1: Simple output
    const test1 = {
        code: "print('Hello, World!')"
    };
    console.log('\n📤 Sending test 1:', JSON.stringify(test1));
    ws.send(JSON.stringify(test1));
});

ws.on('message', function message(data) {
    const response = JSON.parse(data);
    console.log('\n📥 Received response:');
    console.log('  Success:', response.success);
    console.log('  Output:', response.output);
    if (response.error) {
        console.log('  Error:', response.error);
    }
    
    // Test 2: Variables
    if (!ws.test2Sent) {
        ws.test2Sent = true;
        const test2 = {
            code: "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"
        };
        console.log('\n📤 Sending test 2:', JSON.stringify(test2));
        ws.send(JSON.stringify(test2));
    } else if (!ws.test3Sent) {
        // Test 3: Loop
        ws.test3Sent = true;
        const test3 = {
            code: "for i in [1, 2, 3] do\n    print('Number:', i)\nnext i"
        };
        console.log('\n📤 Sending test 3:', JSON.stringify(test3));
        ws.send(JSON.stringify(test3));
    } else {
        ws.close();
    }
});

ws.on('error', function error(err) {
    console.error('❌ Error:', err.message);
});

ws.on('close', function close() {
    console.log('\n🔌 Connection closed');
});
```

Run:
```bash
node test_websocket.js
```

### 4. Python Test

Create file `test_websocket.py`:

```python
import asyncio
import websockets
import json

async def test_websocket():
    uri = "ws://127.0.0.1:8899"
    
    async with websockets.connect(uri) as websocket:
        print("✅ Connected to server")
        
        # Test 1: Simple output
        test1 = {
            "code": "print('Hello, World!')"
        }
        print(f"\n📤 Sending test 1: {json.dumps(test1)}")
        await websocket.send(json.dumps(test1))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Received response:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        if result.get('error'):
            print(f"  Error: {result['error']}")
        
        # Test 2: Variables
        test2 = {
            "code": "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"
        }
        print(f"\n📤 Sending test 2: {json.dumps(test2)}")
        await websocket.send(json.dumps(test2))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Received response:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        
        # Test 3: Function
        test3 = {
            "code": "global function greet(name) do\n    return 'Hello, ' + name + '!'\nendfunction\nprint(greet('DataCode'))"
        }
        print(f"\n📤 Sending test 3: {json.dumps(test3)}")
        await websocket.send(json.dumps(test3))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Received response:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        
        # Test 4: Error (to check error handling)
        test4 = {
            "code": "print(undefined_variable)"
        }
        print(f"\n📤 Sending test 4 (expecting error): {json.dumps(test4)}")
        await websocket.send(json.dumps(test4))
        
        response = await websocket.recv()
        result = json.loads(response)
        print(f"\n📥 Received response:")
        print(f"  Success: {result['success']}")
        print(f"  Output: {result['output']}")
        if result.get('error'):
            print(f"  Error: {result['error']}")

if __name__ == "__main__":
    asyncio.run(test_websocket())
```

Run:
```bash
pip install websockets
python test_websocket.py
```

### 5. Bash Script Using websocat

Create file `test_websocket.sh`:

```bash
#!/bin/bash

SERVER="ws://127.0.0.1:8899"

echo "🧪 Testing DataCode WebSocket Server"
echo "=========================================="
echo ""

# Test 1: Simple output
echo "📤 Test 1: Simple output"
echo '{"code": "print(\"Hello, World!\")"}' | websocat $SERVER
echo ""

# Test 2: Variables
echo "📤 Test 2: Variables"
echo '{"code": "global x = 10\nglobal y = 20\nprint(\"Sum:\", x + y)"}' | websocat $SERVER
echo ""

# Test 3: Loop
echo "📤 Test 3: Loop"
echo '{"code": "for i in [1, 2, 3] do\n    print(\"Number:\", i)\nnext i"}' | websocat $SERVER
echo ""

echo "✅ Testing completed"
```

Run:
```bash
chmod +x test_websocket.sh
./test_websocket.sh
```

### 6. Simple JSON Requests for Copying

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

## Expected Responses

### Successful Execution:
```json
{
  "success": true,
  "output": "Hello, World!\n",
  "error": null
}
```

### Execution Error:
```json
{
  "success": false,
  "output": "",
  "error": "Error: variable 'undefined_variable' is not defined"
}
```
