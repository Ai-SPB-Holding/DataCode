# Quick WebSocket Server Test

## 1. Start the Server

```bash
datacode --websocket --host 0.0.0.0 --port 8899
```

## 2. Open Web Client

Open file `examples/08-websocket/websocket_client_example.html` in browser and connect to `ws://127.0.0.1:8899`

## 3. Or Use Ready Scripts

### Node.js (requires: npm install ws)
```bash
node examples/08-websocket/test_websocket.js
```

### Python (requires: pip install websockets)
```bash
python examples/08-websocket/test_websocket.py
```

### Bash (requires: cargo install websocat)
```bash
./examples/08-websocket/test_websocket.sh
```

## 4. Or Send Request Manually

### Via wscat:
```bash
npm install -g wscat
wscat -c ws://127.0.0.1:8899
# Then enter:
{"code": "print('Hello, World!')"}
```

### Via websocat:
```bash
cargo install websocat
echo '{"code": "print(\"Hello, World!\")"}' | websocat ws://127.0.0.1:8899
```

## Ready JSON Requests for Copying:

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

## Expected Response:

```json
{
  "success": true,
  "output": "Hello, World!\n",
  "error": null
}
```
